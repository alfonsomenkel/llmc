use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{json, Value};
use tempfile::tempdir;

fn write_json(path: &Path, value: &Value) {
    let payload = serde_json::to_string_pretty(value).expect("serialize fixture json");
    fs::write(path, payload).expect("write fixture json");
}

fn run_cli(contract_path: &Path, output_path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_llm_contracts"))
        .arg("--contract")
        .arg(contract_path)
        .arg("--output")
        .arg(output_path)
        .output()
        .expect("run llm_contracts binary")
}

fn assert_exit_code(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "expected exit {expected}, got {:?}; stdout: {}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_stdout_verdict_schema(output: &Output, expected_status: &str) {
    let parsed: Value = serde_json::from_slice(&output.stdout).expect("stdout is valid json");
    let root = parsed.as_object().expect("stdout root must be object");

    let status = root
        .get("status")
        .and_then(Value::as_str)
        .expect("status must be a string");
    assert!(status == "pass" || status == "fail");
    assert_eq!(status, expected_status);

    let violations = root
        .get("violations")
        .and_then(Value::as_array)
        .expect("violations must be an array");

    if status == "fail" {
        assert!(
            !violations.is_empty(),
            "fail verdict must include violations"
        );
        for violation in violations {
            let v = violation
                .as_object()
                .expect("each violation must be an object");
            assert!(
                v.get("rule").and_then(Value::as_str).is_some(),
                "violation.rule must be a string"
            );
            assert!(
                v.get("field").and_then(Value::as_str).is_some(),
                "violation.field must be a string"
            );
            assert!(
                v.get("message").and_then(Value::as_str).is_some(),
                "violation.message must be a string"
            );
        }
    }
}

#[test]
fn exits_zero_when_contract_passes() {
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

    let result = run_cli(&contract_path, &output_path);
    assert_exit_code(&result, 0);
    assert_stdout_verdict_schema(&result, "pass");
}

#[test]
fn exits_one_when_contract_has_violations() {
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

    let result = run_cli(&contract_path, &output_path);
    assert_exit_code(&result, 1);
    assert_stdout_verdict_schema(&result, "fail");
}

#[test]
fn exits_one_when_allowed_values_rule_fails() {
    let dir = tempdir().expect("create temp dir");
    let contract_path = dir.path().join("contract.json");
    let output_path = dir.path().join("output.json");

    let contract = json!({
        "inputs": ["prompt"],
        "output_type": "array",
        "rules": [
            {"rule": "allowed_values", "field": "status", "values": ["ok", "accepted"]}
        ]
    });
    let output = json!([
        {"status": "rejected"}
    ]);

    write_json(&contract_path, &contract);
    write_json(&output_path, &output);

    let result = run_cli(&contract_path, &output_path);
    assert_exit_code(&result, 1);
    assert_stdout_verdict_schema(&result, "fail");
}

#[test]
fn exits_two_when_contract_is_invalid() {
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

    let result = run_cli(&contract_path, &output_path);
    assert_exit_code(&result, 2);
    assert_stdout_verdict_schema(&result, "fail");
}

#[test]
fn exits_three_when_output_json_is_invalid() {
    let dir = tempdir().expect("create temp dir");
    let contract_path = dir.path().join("contract.json");
    let output_path = dir.path().join("output.json");

    let contract = json!({
        "inputs": ["prompt"],
        "output_type": "array",
        "rules": []
    });

    write_json(&contract_path, &contract);
    fs::write(&output_path, "{not valid json").expect("write invalid output json");

    let result = run_cli(&contract_path, &output_path);
    assert_exit_code(&result, 3);
    assert_stdout_verdict_schema(&result, "fail");
}

#[test]
fn exits_three_when_output_file_is_missing() {
    let dir = tempdir().expect("create temp dir");
    let contract_path = dir.path().join("contract.json");
    let missing_output_path = dir.path().join("missing_output.json");

    let contract = json!({
        "inputs": ["prompt"],
        "output_type": "array",
        "rules": []
    });

    write_json(&contract_path, &contract);

    let result = run_cli(&contract_path, &missing_output_path);
    assert_exit_code(&result, 3);
    assert_stdout_verdict_schema(&result, "fail");
}
