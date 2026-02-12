# llmc

[![CI](https://github.com/alfonsomenkel/llm-contacts/actions/workflows/ci.yml/badge.svg)](https://github.com/alfonsomenkel/llm-contacts/actions/workflows/ci.yml)

`llm contracts` validates tool/debugger/LLM-generated facts (JSON) against developer-defined contracts and returns a deterministic, external PASS/FAIL verdict.

## Core flow

LLM -> tool/debugger -> facts (JSON) -> llmc -> PASS / FAIL

## Usage

Example contract (`contract.json`):

```json
{
  "contract": "user_list",
  "version": 1,
  "inputs": ["prompt"],
  "output_type": "array",
  "rules": [
    { "rule": "required_field", "field": "id" },
    { "rule": "field_type", "field": "id", "expected": "number" },
    { "rule": "min_items", "value": 2 }
  ]
}
```

Notes:
- `inputs` is parsed but not validated or enforced.
- Validation is applied to `output_type` and `rules`.

Example facts/output (`output.json`):

```json
[
  { "id": 1, "name": "Alice" },
  { "id": 2, "name": "Bob" }
]
```

Run:

```bash
llmc --contract ./contract.json --output ./output.json
```

## Build

Build debug binary:

```bash
cargo build
```

Build release binary:

```bash
cargo build --release
```

Run tests:

```bash
cargo test
```

Optional shortcuts with `make`:

```bash
make build
make release
make test
make run-pass
make run-fail
```

PASS verdict:

```json
{ "status": "pass", "violations": [] }
```

FAIL verdict (example):

```json
{
  "status": "fail",
  "violations": [
    { "rule": "RequiredField", "field": "", "message": "Missing required field 'id'." }
  ]
}
```

## File paths

Use relative paths for `--contract` and `--output` when possible. This improves portability across environments, makes CI configuration simpler, and supports reproducible runs from repository roots. Absolute paths are supported by the CLI but are discouraged.

## Exit codes

- `0`: pass
- `1`: contract violations
- `2`: invalid contract
- `3`: runtime / IO error

## Supported rules

- `required_field`
- `field_type`
- `allowed_values`
- `regex`
- `min_items`
- `no_empty_rows`

## Contract versioning

Contracts are versioned. Bump the contract version when contract semantics change. Facts/outputs are not versioned.

## What this tool is not

- Not a linter
- Not a debugger
- Not an LLM evaluator
- It enforces invariants only

## License

MIT. See `LICENSE`.
