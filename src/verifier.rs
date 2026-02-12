use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use crate::contract::{Contract, OutputType, Rule, ValueType};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VerdictStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Violation {
    pub rule_name: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<Value>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Verdict {
    pub status: VerdictStatus,
    pub violations: Vec<Violation>,
}

#[derive(Debug)]
pub enum RunError {
    Io(io::Error),
    InvalidContract(serde_json::Error),
    InvalidContractRegex(regex::Error),
    InvalidOutput(serde_json::Error),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Io(err) => write!(f, "I/O error: {err}"),
            RunError::InvalidContract(err) => write!(f, "Invalid contract JSON: {err}"),
            RunError::InvalidContractRegex(err) => write!(f, "Invalid contract regex: {err}"),
            RunError::InvalidOutput(err) => write!(f, "Invalid output JSON: {err}"),
        }
    }
}

impl Error for RunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RunError::Io(err) => Some(err),
            RunError::InvalidContract(err) => Some(err),
            RunError::InvalidContractRegex(err) => Some(err),
            RunError::InvalidOutput(err) => Some(err),
        }
    }
}

pub fn run(contract_path: &Path, output_path: &Path) -> Result<Verdict, RunError> {
    let contract_contents = fs::read_to_string(contract_path).map_err(RunError::Io)?;
    let output_contents = fs::read_to_string(output_path).map_err(RunError::Io)?;

    let contract: Contract =
        serde_json::from_str(&contract_contents).map_err(RunError::InvalidContract)?;
    let output: Value = serde_json::from_str(&output_contents).map_err(RunError::InvalidOutput)?;
    validate_contract(&contract)?;

    Ok(verify(&contract, &output))
}

pub fn verify(contract: &Contract, output: &Value) -> Verdict {
    let mut violations = Vec::new();

    match contract.output_type {
        OutputType::Object if !output.is_object() => violations.push(simple_violation(
            "OutputType",
            "Expected top-level JSON object.".to_string(),
        )),
        OutputType::Array if !output.is_array() => violations.push(simple_violation(
            "OutputType",
            "Expected top-level JSON array.".to_string(),
        )),
        _ => {}
    }

    for rule in &contract.rules {
        check_rule(rule, output, &mut violations);
    }

    let status = if violations.is_empty() {
        VerdictStatus::Pass
    } else {
        VerdictStatus::Fail
    };

    Verdict { status, violations }
}

fn simple_violation(rule_name: &str, detail: String) -> Violation {
    Violation {
        rule_name: rule_name.to_string(),
        detail,
        field: None,
        rule: None,
        expected: None,
        actual: None,
    }
}

fn allowed_values_violation(
    field: &str,
    expected: &[Value],
    actual: &Value,
    detail: String,
) -> Violation {
    Violation {
        rule_name: "AllowedValues".to_string(),
        detail,
        field: Some(field.to_string()),
        rule: Some("allowed_values".to_string()),
        expected: Some(Value::Array(expected.to_vec())),
        actual: Some(actual.clone()),
    }
}

fn regex_violation(field: &str, pattern: &str, actual: &Value, detail: String) -> Violation {
    Violation {
        rule_name: "Regex".to_string(),
        detail,
        field: Some(field.to_string()),
        rule: Some("regex".to_string()),
        expected: Some(Value::String(pattern.to_string())),
        actual: Some(actual.clone()),
    }
}

fn min_items_violation(value: u64, actual: Value, detail: String) -> Violation {
    Violation {
        rule_name: "MinItems".to_string(),
        detail,
        field: Some("$".to_string()),
        rule: Some("min_items".to_string()),
        expected: Some(Value::from(value)),
        actual: Some(actual),
    }
}

fn validate_contract(contract: &Contract) -> Result<(), RunError> {
    for rule in &contract.rules {
        if let Rule::Regex { pattern, .. } = rule {
            Regex::new(pattern).map_err(RunError::InvalidContractRegex)?;
        }
    }
    Ok(())
}

fn check_rule(rule: &Rule, output: &Value, violations: &mut Vec<Violation>) {
    match rule {
        Rule::RequiredField { field } => check_required_field(field, output, violations),
        Rule::FieldType { field, expected } => {
            check_field_type(field, expected, output, violations)
        }
        Rule::AllowedValues { field, values } => {
            check_allowed_values(field, values, output, violations)
        }
        Rule::Regex { field, pattern } => check_regex(field, pattern, output, violations),
        Rule::MinItems { value } => check_min_items(*value, output, violations),
        Rule::NoEmptyRows => check_no_empty_rows(output, violations),
    }
}

fn check_required_field(field: &str, output: &Value, violations: &mut Vec<Violation>) {
    match output {
        Value::Object(map) => {
            if !map.contains_key(field) {
                violations.push(simple_violation(
                    "RequiredField",
                    format!("Missing required field '{field}'."),
                ));
            }
        }
        Value::Array(rows) => {
            for (idx, row) in rows.iter().enumerate() {
                match row {
                    Value::Object(map) => {
                        if !map.contains_key(field) {
                            violations.push(simple_violation(
                                "RequiredField",
                                format!("Row {idx} is missing required field '{field}'."),
                            ));
                        }
                    }
                    _ => violations.push(simple_violation(
                        "RequiredField",
                        format!("Row {idx} is not an object."),
                    )),
                }
            }
        }
        _ => violations.push(simple_violation(
            "RequiredField",
            "Output must be an object or an array of objects.".to_string(),
        )),
    }
}

fn check_field_type(
    field: &str,
    expected: &ValueType,
    output: &Value,
    violations: &mut Vec<Violation>,
) {
    match output {
        Value::Object(map) => check_field_type_in_map(field, expected, map, None, violations),
        Value::Array(rows) => {
            for (idx, row) in rows.iter().enumerate() {
                match row {
                    Value::Object(map) => {
                        check_field_type_in_map(field, expected, map, Some(idx), violations)
                    }
                    _ => violations.push(simple_violation(
                        "FieldType",
                        format!("Row {idx} is not an object."),
                    )),
                }
            }
        }
        _ => violations.push(simple_violation(
            "FieldType",
            "Output must be an object or an array of objects.".to_string(),
        )),
    }
}

fn check_field_type_in_map(
    field: &str,
    expected: &ValueType,
    map: &serde_json::Map<String, Value>,
    row_index: Option<usize>,
    violations: &mut Vec<Violation>,
) {
    match map.get(field) {
        Some(value) => {
            if !matches_value_type(value, expected) {
                let location = row_index
                    .map(|i| format!("Row {i} field '{field}'"))
                    .unwrap_or_else(|| format!("Field '{field}'"));
                violations.push(simple_violation(
                    "FieldType",
                    format!(
                        "{location} expected type '{}', got '{}'.",
                        value_type_label(expected),
                        detected_value_type(value)
                    ),
                ));
            }
        }
        None => {
            let location = row_index
                .map(|i| format!("Row {i}"))
                .unwrap_or_else(|| "Object".to_string());
            violations.push(simple_violation(
                "FieldType",
                format!("{location} is missing field '{field}' for type check."),
            ));
        }
    }
}

fn check_no_empty_rows(output: &Value, violations: &mut Vec<Violation>) {
    match output {
        Value::Array(rows) => {
            for (idx, row) in rows.iter().enumerate() {
                match row {
                    Value::Object(map) => {
                        if map.is_empty() || map.values().all(is_empty_value) {
                            violations.push(simple_violation(
                                "NoEmptyRows",
                                format!("Row {idx} is empty."),
                            ));
                        }
                    }
                    _ => violations.push(simple_violation(
                        "NoEmptyRows",
                        format!("Row {idx} is not an object."),
                    )),
                }
            }
        }
        _ => violations.push(simple_violation(
            "NoEmptyRows",
            "NoEmptyRows requires top-level array output.".to_string(),
        )),
    }
}

fn check_allowed_values(
    field: &str,
    values: &[Value],
    output: &Value,
    violations: &mut Vec<Violation>,
) {
    match output {
        Value::Object(map) => {
            if let Some(actual) = map.get(field) {
                if !values.iter().any(|allowed| allowed == actual) {
                    violations.push(allowed_values_violation(
                        field,
                        values,
                        actual,
                        format!("Field '{field}' has a disallowed value."),
                    ));
                }
            }
        }
        Value::Array(rows) => {
            for (idx, row) in rows.iter().enumerate() {
                match row {
                    Value::Object(map) => {
                        if let Some(actual) = map.get(field) {
                            if !values.iter().any(|allowed| allowed == actual) {
                                violations.push(allowed_values_violation(
                                    field,
                                    values,
                                    actual,
                                    format!("Row {idx} field '{field}' has a disallowed value."),
                                ));
                            }
                        }
                    }
                    _ => violations.push(simple_violation(
                        "AllowedValues",
                        format!("Row {idx} is not an object."),
                    )),
                }
            }
        }
        _ => violations.push(simple_violation(
            "AllowedValues",
            "Output must be an object or an array of objects.".to_string(),
        )),
    }
}

fn check_regex(field: &str, pattern: &str, output: &Value, violations: &mut Vec<Violation>) {
    let regex = Regex::new(pattern).expect("regex patterns validated in run()");
    match output {
        Value::Object(map) => check_regex_in_map(field, pattern, &regex, map, None, violations),
        Value::Array(rows) => {
            for (idx, row) in rows.iter().enumerate() {
                match row {
                    Value::Object(map) => {
                        check_regex_in_map(field, pattern, &regex, map, Some(idx), violations)
                    }
                    _ => violations.push(simple_violation(
                        "Regex",
                        format!("Row {idx} is not an object."),
                    )),
                }
            }
        }
        _ => violations.push(simple_violation(
            "Regex",
            "Output must be an object or an array of objects.".to_string(),
        )),
    }
}

fn check_regex_in_map(
    field: &str,
    pattern: &str,
    regex: &Regex,
    map: &serde_json::Map<String, Value>,
    row_index: Option<usize>,
    violations: &mut Vec<Violation>,
) {
    let Some(actual) = map.get(field) else {
        return;
    };

    match actual {
        Value::String(s) => {
            if !regex.is_match(s) {
                let detail = row_index
                    .map(|idx| format!("Row {idx} field '{field}' does not match regex pattern."))
                    .unwrap_or_else(|| format!("Field '{field}' does not match regex pattern."));
                violations.push(regex_violation(field, pattern, actual, detail));
            }
        }
        _ => {
            let detail = row_index
                .map(|idx| format!("Row {idx} field '{field}' must be a string for regex rule."))
                .unwrap_or_else(|| format!("Field '{field}' must be a string for regex rule."));
            violations.push(regex_violation(field, pattern, actual, detail));
        }
    }
}

fn check_min_items(value: u64, output: &Value, violations: &mut Vec<Violation>) {
    match output {
        Value::Array(items) => {
            let actual_len = items.len() as u64;
            if actual_len < value {
                violations.push(min_items_violation(
                    value,
                    Value::from(actual_len),
                    format!(
                        "Top-level array must contain at least {value} items, found {actual_len}."
                    ),
                ));
            }
        }
        _ => {
            violations.push(min_items_violation(
                value,
                Value::String(detected_value_type(output).to_string()),
                "MinItems requires top-level array output.".to_string(),
            ));
        }
    }
}

fn matches_value_type(value: &Value, expected: &ValueType) -> bool {
    match expected {
        ValueType::String => value.is_string(),
        ValueType::Number => value.is_number(),
        ValueType::Boolean => value.is_boolean(),
        ValueType::Object => value.is_object(),
        ValueType::Array => value.is_array(),
        ValueType::Null => value.is_null(),
    }
}

fn is_empty_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.trim().is_empty(),
        Value::Array(v) => v.is_empty(),
        Value::Object(m) => m.is_empty(),
        _ => false,
    }
}

fn value_type_label(value_type: &ValueType) -> &'static str {
    match value_type {
        ValueType::String => "string",
        ValueType::Number => "number",
        ValueType::Boolean => "boolean",
        ValueType::Object => "object",
        ValueType::Array => "array",
        ValueType::Null => "null",
    }
}

fn detected_value_type(value: &Value) -> &'static str {
    if value.is_string() {
        "string"
    } else if value.is_number() {
        "number"
    } else if value.is_boolean() {
        "boolean"
    } else if value.is_object() {
        "object"
    } else if value.is_array() {
        "array"
    } else {
        "null"
    }
}
