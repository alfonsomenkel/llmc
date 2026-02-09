# Contract Versioning

This project versions contract schemas by file name in `/Volumes/Ext/dev/LLM Contracts/examples`.

## Versions

### v1
- File: `contract.v1.json`
- Rules:
  - `required_field` on `id`
  - `field_type` on `id` (`number`)
  - `no_empty_rows`
- Does not include `allowed_values`.

### v2
- File: `contract.v2.json`
- Rules:
  - `required_field` on `id`
  - `field_type` on `id` (`number`)
  - `allowed_values` on `status` with allowed values: `"ok"`, `"accepted"`
  - `no_empty_rows`

## Compatibility

- `v2` adds validation semantics (`allowed_values`) and is stricter than `v1`.
- Outputs valid under `v1` may fail under `v2` if `status` is present and not in the allowed set.

## Run Commands

From project root:

```bash
cd "/Volumes/Ext/dev/LLM Contracts"
```

Run v1 contract:

```bash
cargo run -- --contract "/Volumes/Ext/dev/LLM Contracts/examples/contract.v1.json" --output "/Volumes/Ext/dev/LLM Contracts/examples/output_pass.json"
```

Run v2 contract (pass sample):

```bash
cargo run -- --contract "/Volumes/Ext/dev/LLM Contracts/examples/contract.v2.json" --output "/Volumes/Ext/dev/LLM Contracts/examples/output_pass.json"
```

Run v2 contract (fail sample):

```bash
cargo run -- --contract "/Volumes/Ext/dev/LLM Contracts/examples/contract.v2.json" --output "/Volumes/Ext/dev/LLM Contracts/examples/output_fail.json"
```

## Exit Codes

- `0`: contract passed
- `1`: contract failed (violations)
- `2`: invalid contract JSON
- `3`: runtime/IO error (including invalid output JSON)
