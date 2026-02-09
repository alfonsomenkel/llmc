#[path = "../src/contract.rs"]
mod contract;
#[path = "../src/verifier.rs"]
mod verifier;

use std::fs;
use std::path::Path;

use serde_json::{json, Value};
use tempfile::tempdir;

use verifier::{run, RunError, VerdictStatus};

fn write_json(path: &Path, value: &Value) {
    let payload = serde_json::to_string_pretty(value).expect("serialize json fixture");
    fs::write(path, payload).expect("write json fixture");
}

#[test]
fn validates_contract_successfully() {
    let dir = tempdir().expect("create temp dir");
    let contract_path = dir.path().join("contract.json");
    let output_path = dir.path().join("output.json");

    let contract = json!({
        "inputs": ["prompt"],
        "output_type": "array",
        "rules": [
            {"rule": "required_field", "field": "id"},
            {"rule": "field_type", "field": "id", "expected": "number"},
            {"rule": "no_empty_rows"}
        ]
    });

    let output = json!([
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ]);

    write_json(&contract_path, &contract);
    write_json(&output_path, &output);

    let verdict = run(&contract_path, &output_path).expect("verifier should run");

    assert_eq!(verdict.status, VerdictStatus::Pass);
    assert!(verdict.violations.is_empty());
}

#[test]
fn reports_missing_required_field_violation() {
    let dir = tempdir().expect("create temp dir");
    let contract_path = dir.path().join("contract.json");
    let output_path = dir.path().join("output.json");

    let contract = json!({
        "inputs": ["prompt"],
        "output_type": "array",
        "rules": [
            {"rule": "required_field", "field": "id"}
        ]
    });

    let output = json!([
        {"name": "Alice"}
    ]);

    write_json(&contract_path, &contract);
    write_json(&output_path, &output);

    let verdict = run(&contract_path, &output_path).expect("verifier should run");

    assert_eq!(verdict.status, VerdictStatus::Fail);
    assert!(verdict
        .violations
        .iter()
        .any(|v| v.rule_name == "RequiredField"));
}

#[test]
fn reports_empty_row_violation() {
    let dir = tempdir().expect("create temp dir");
    let contract_path = dir.path().join("contract.json");
    let output_path = dir.path().join("output.json");

    let contract = json!({
        "inputs": ["prompt"],
        "output_type": "array",
        "rules": [
            {"rule": "no_empty_rows"}
        ]
    });

    let output = json!([
        {"id": 1},
        {}
    ]);

    write_json(&contract_path, &contract);
    write_json(&output_path, &output);

    let verdict = run(&contract_path, &output_path).expect("verifier should run");

    assert_eq!(verdict.status, VerdictStatus::Fail);
    assert!(verdict
        .violations
        .iter()
        .any(|v| v.rule_name == "NoEmptyRows"));
}

#[test]
fn returns_invalid_contract_error_for_bad_contract_json() {
    let dir = tempdir().expect("create temp dir");
    let contract_path = dir.path().join("contract.json");
    let output_path = dir.path().join("output.json");

    let contract = json!({
        "inputs": ["prompt"],
        "output_type": "array"
    });
    let output = json!([]);

    write_json(&contract_path, &contract);
    write_json(&output_path, &output);

    let err = run(&contract_path, &output_path).expect_err("contract should be invalid");
    assert!(matches!(err, RunError::InvalidContract(_)));
}

#[test]
fn returns_invalid_output_error_for_bad_output_json() {
    let dir = tempdir().expect("create temp dir");
    let contract_path = dir.path().join("contract.json");
    let output_path = dir.path().join("output.json");

    let contract = json!({
        "inputs": ["prompt"],
        "output_type": "array",
        "rules": []
    });

    write_json(&contract_path, &contract);
    fs::write(&output_path, "{this is not valid json").expect("write invalid output json");

    let err = run(&contract_path, &output_path).expect_err("output should be invalid json");
    assert!(matches!(err, RunError::InvalidOutput(_)));
}
