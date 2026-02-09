mod contract;
mod verifier;

use std::path::PathBuf;

use clap::Parser;
use serde_json::json;

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
        Err(RunError::InvalidOutput(err)) => (
            failure_verdict("Runtime", format!("Invalid output JSON: {err}")),
            EXIT_RUNTIME_IO,
        ),
        Err(RunError::Io(err)) => (
            failure_verdict("Runtime", format!("I/O error: {err}")),
            EXIT_RUNTIME_IO,
        ),
    };

    let serialized = match serde_json::to_string_pretty(&verdict) {
        Ok(serialized) => serialized,
        Err(err) => {
            exit_code = EXIT_RUNTIME_IO;
            serde_json::to_string_pretty(&json!({
                "status": "fail",
                "violations": [
                    {
                        "rule_name": "Runtime",
                        "detail": format!("Failed to serialize verdict: {err}")
                    }
                ]
            }))
            .expect("failed to serialize fallback verdict")
        }
    };

    println!("{serialized}");
    std::process::exit(exit_code);
}

fn failure_verdict(rule_name: &str, detail: String) -> Verdict {
    Verdict {
        status: VerdictStatus::Fail,
        violations: vec![Violation {
            rule_name: rule_name.to_string(),
            detail,
        }],
    }
}
