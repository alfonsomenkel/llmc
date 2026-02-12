# Contract Versioning

This project versions contract schemas by file name in `./examples`.

Outputs are shared and not versioned:
- `output_pass.json`
- `output_fail.json`

## Versions

### v1
- File: `contract.v1.json`
- Rules:
  - `required_field` on `id`
  - `field_type` on `id` (`number`)
  - `no_empty_rows`
- Does not include `allowed_values`, `regex`, or `min_items`.

### v2
- File: `contract.v2.json`
- Rules:
  - `required_field` on `id`
  - `field_type` on `id` (`number`)
  - `allowed_values` on `status` with allowed values: `"ok"`, `"accepted"`
  - `no_empty_rows`
- Does not include `regex` or `min_items`.

### v3
- File: `contract.v3.json`
- Rules:
  - `required_field` on `id`
  - `field_type` on `id` (`number`)
  - `allowed_values` on `status` with allowed values: `"ok"`, `"accepted"`
  - `regex` on `code` with pattern: `^[A-Z]{3}$`
  - `no_empty_rows`
- Does not include `min_items`.

### v4
- File: `contract.v4.json`
- Rules:
  - `required_field` on `id`
  - `field_type` on `id` (`number`)
  - `allowed_values` on `status` with allowed values: `"ok"`, `"accepted"`
  - `regex` on `code` with pattern: `^[A-Z]{3}$`
  - `min_items` at root with value: `3`
  - `no_empty_rows`

## Compatibility

- `v2` adds `allowed_values` and is stricter than `v1`.
- `v3` adds `regex` and is stricter than `v2`.
- `v4` adds `min_items` and is stricter than `v3`.
- `output_pass.json` is valid for all versions.
- `output_fail.json` fails validation for v2, v3, and v4.

## Run Commands

From project root:

Run v1 contract (pass sample):

```bash
cargo run -- --contract ./examples/contract.v1.json --output ./examples/output_pass.json
```

Run v2 contract (pass sample):

```bash
cargo run -- --contract ./examples/contract.v2.json --output ./examples/output_pass.json
```

Run v2 contract (fail sample):

```bash
cargo run -- --contract ./examples/contract.v2.json --output ./examples/output_fail.json
```

Run v3 contract (pass sample):

```bash
cargo run -- --contract ./examples/contract.v3.json --output ./examples/output_pass.json
```

Run v3 contract (fail sample):

```bash
cargo run -- --contract ./examples/contract.v3.json --output ./examples/output_fail.json
```

Run v4 contract (pass sample):

```bash
cargo run -- --contract ./examples/contract.v4.json --output ./examples/output_pass.json
```

Run v4 contract (fail sample):

```bash
cargo run -- --contract ./examples/contract.v4.json --output ./examples/output_fail.json
```

## Exit Codes

- `0`: contract passed
- `1`: contract failed (violations)
- `2`: invalid contract JSON
- `3`: runtime/IO error (including invalid output JSON)
