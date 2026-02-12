mod contract;
mod verifier;

use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::Parser;
use serde_json::{json, Value};

use verifier::{run, RunError, Verdict, VerdictStatus, Violation};

const EXIT_PASS: i32 = 0;
const EXIT_CONTRACT_FAILED: i32 = 1;
const EXIT_INVALID_CONTRACT: i32 = 2;
const EXIT_RUNTIME_IO: i32 = 3;

#[derive(Debug, Parser)]
#[command(name = "llm_contracts")]
#[command(about = "Verify LLM outputs against a JSON contract")]
struct Cli {
    #[arg(short, long)]
    contract: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let (verdict, mut exit_code) = match run(&cli.contract, &cli.output) {
        Ok(verdict) => {
            let exit_code = if matches!(verdict.status, VerdictStatus::Pass) {
                EXIT_PASS
            } else {
                EXIT_CONTRACT_FAILED
            };
            (verdict, exit_code)
        }
        Err(RunError::InvalidContract(err)) => (
            failure_verdict("InvalidContract", err.to_string()),
            EXIT_INVALID_CONTRACT,
        ),
        Err(RunError::InvalidContractRegex(err)) => (
            failure_verdict("InvalidContract", err.to_string()),
            EXIT_INVALID_CONTRACT,
        ),
        Err(RunError::InvalidOutput(err)) => (
            failure_verdict("Runtime", format!("Invalid output JSON: {err}")),
            EXIT_RUNTIME_IO,
        ),
        Err(RunError::Io(err)) => (
            failure_verdict("Runtime", format!("I/O error: {err}")),
            EXIT_RUNTIME_IO,
        ),
    };

    let public_verdict = to_public_verdict(&verdict);
    let serialized = match serde_json::to_string_pretty(&public_verdict) {
        Ok(serialized) => serialized,
        Err(err) => {
            exit_code = EXIT_RUNTIME_IO;
            serde_json::to_string_pretty(&json!({
                "status": "fail",
                "violations": [
                    {
                        "rule": "runtime",
                        "field": "",
                        "message": format!("Failed to serialize verdict: {err}")
                    }
                ]
            }))
            .expect("failed to serialize fallback verdict")
        }
    };

    println!("{serialized}");
    std::process::exit(exit_code);
}

fn to_public_verdict(verdict: &Verdict) -> Value {
    let status = if matches!(verdict.status, VerdictStatus::Pass) {
        "pass"
    } else {
        "fail"
    };
    let violations: Vec<Value> = verdict.violations.iter().map(to_public_violation).collect();
    json!({
        "status": status,
        "violations": violations
    })
}

fn to_public_violation(violation: &Violation) -> Value {
    let mut obj = BTreeMap::new();
    obj.insert(
        "rule",
        Value::String(
            violation
                .rule
                .clone()
                .unwrap_or_else(|| violation.rule_name.clone()),
        ),
    );
    obj.insert(
        "field",
        Value::String(violation.field.clone().unwrap_or_default()),
    );
    obj.insert("message", Value::String(violation.detail.clone()));
    if let Some(expected) = &violation.expected {
        obj.insert("expected", expected.clone());
    }
    if let Some(actual) = &violation.actual {
        obj.insert("actual", actual.clone());
    }
    serde_json::to_value(obj).expect("serialize public violation")
}

fn failure_verdict(rule_name: &str, detail: String) -> Verdict {
    Verdict {
        status: VerdictStatus::Fail,
        violations: vec![Violation {
            rule_name: rule_name.to_string(),
            detail,
            field: None,
            rule: None,
            expected: None,
            actual: None,
        }],
    }
}
