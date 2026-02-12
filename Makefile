.PHONY: build release test fmt clippy run-pass run-fail

BIN := llmc

build:
	cargo build --bin $(BIN)

release:
	cargo build --release --bin $(BIN)

test:
	cargo test

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

run-pass:
	cargo run --bin $(BIN) -- --contract examples/contract.v4.json --output examples/output_pass.json

run-fail:
	cargo run --bin $(BIN) -- --contract examples/contract.v4.json --output examples/output_fail.json
