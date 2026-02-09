use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

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
    InvalidOutput(serde_json::Error),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Io(err) => write!(f, "I/O error: {err}"),
            RunError::InvalidContract(err) => write!(f, "Invalid contract JSON: {err}"),
            RunError::InvalidOutput(err) => write!(f, "Invalid output JSON: {err}"),
        }
    }
}

impl Error for RunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RunError::Io(err) => Some(err),
            RunError::InvalidContract(err) => Some(err),
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

    Ok(verify(&contract, &output))
}

pub fn verify(contract: &Contract, output: &Value) -> Verdict {
    let mut violations = Vec::new();

    match contract.output_type {
        OutputType::Object if !output.is_object() => violations.push(Violation {
            rule_name: "OutputType".to_string(),
            detail: "Expected top-level JSON object.".to_string(),
        }),
        OutputType::Array if !output.is_array() => violations.push(Violation {
            rule_name: "OutputType".to_string(),
            detail: "Expected top-level JSON array.".to_string(),
        }),
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

fn check_rule(rule: &Rule, output: &Value, violations: &mut Vec<Violation>) {
    match rule {
        Rule::RequiredField { field } => check_required_field(field, output, violations),
        Rule::FieldType { field, expected } => {
            check_field_type(field, expected, output, violations)
        }
        Rule::NoEmptyRows => check_no_empty_rows(output, violations),
    }
}

fn check_required_field(field: &str, output: &Value, violations: &mut Vec<Violation>) {
    match output {
        Value::Object(map) => {
            if !map.contains_key(field) {
                violations.push(Violation {
                    rule_name: "RequiredField".to_string(),
                    detail: format!("Missing required field '{field}'."),
                });
            }
        }
        Value::Array(rows) => {
            for (idx, row) in rows.iter().enumerate() {
                match row {
                    Value::Object(map) => {
                        if !map.contains_key(field) {
                            violations.push(Violation {
                                rule_name: "RequiredField".to_string(),
                                detail: format!("Row {idx} is missing required field '{field}'."),
                            });
                        }
                    }
                    _ => violations.push(Violation {
                        rule_name: "RequiredField".to_string(),
                        detail: format!("Row {idx} is not an object."),
                    }),
                }
            }
        }
        _ => violations.push(Violation {
            rule_name: "RequiredField".to_string(),
            detail: "Output must be an object or an array of objects.".to_string(),
        }),
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
                    _ => violations.push(Violation {
                        rule_name: "FieldType".to_string(),
                        detail: format!("Row {idx} is not an object."),
                    }),
                }
            }
        }
        _ => violations.push(Violation {
            rule_name: "FieldType".to_string(),
            detail: "Output must be an object or an array of objects.".to_string(),
        }),
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
                violations.push(Violation {
                    rule_name: "FieldType".to_string(),
                    detail: format!(
                        "{location} expected type '{}', got '{}'.",
                        value_type_label(expected),
                        detected_value_type(value)
                    ),
                });
            }
        }
        None => {
            let location = row_index
                .map(|i| format!("Row {i}"))
                .unwrap_or_else(|| "Object".to_string());
            violations.push(Violation {
                rule_name: "FieldType".to_string(),
                detail: format!("{location} is missing field '{field}' for type check."),
            });
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
                            violations.push(Violation {
                                rule_name: "NoEmptyRows".to_string(),
                                detail: format!("Row {idx} is empty."),
                            });
                        }
                    }
                    _ => violations.push(Violation {
                        rule_name: "NoEmptyRows".to_string(),
                        detail: format!("Row {idx} is not an object."),
                    }),
                }
            }
        }
        _ => violations.push(Violation {
            rule_name: "NoEmptyRows".to_string(),
            detail: "NoEmptyRows requires top-level array output.".to_string(),
        }),
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
