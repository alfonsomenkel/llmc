# Changelog

All notable changes to this project are documented here.

This project follows contract-semantic versioning:
- Contracts define meaning
- Versions reflect semantic changes

---

## [Unreleased]

- No changes yet.

---

## [0.1.1] - 2026-02-12

### Changed
- Renamed project/package references from `llm-contracts` / `llm_contracts` to `llmc`.
- Bumped project version to `0.1.1` for release tag `v0.1.1`.
- Updated repository links and usage docs to match the `llmc` name.

---

## [0.1.0] - 2026-02-12

### Added
- Rust CLI verifier (`llm_contracts`) using `clap`, `serde`, and `serde_json`.
- Contract DSL parsing for:
  - `contract` (optional metadata)
  - `version` (optional metadata)
  - `inputs` (required metadata field)
  - `output_type`
  - `rules`
- Rule support:
  - `required_field`
  - `field_type`
  - `allowed_values`
  - `regex`
  - `min_items`
  - `no_empty_rows`
- Contract-level regex validation (invalid regex patterns are treated as invalid contracts).
- Structured JSON verdict output on stdout with:
  - `status` (`pass` / `fail`)
  - `violations` array
  - violation fields including `rule`, `field`, `message`, plus `expected`/`actual` when present.
- Deterministic exit codes:
  - `0` pass
  - `1` contract violations
  - `2` invalid contract
  - `3` runtime/IO error
- Integration tests for:
  - CLI exit-code behavior
  - stdout verdict JSON schema
  - rule failure paths (`allowed_values`, `regex`, `min_items`)
- Example contracts and fixtures:
  - `examples/contract.v1.json` through `examples/contract.v4.json`
  - shared non-versioned outputs: `examples/output_pass.json`, `examples/output_fail.json`
- Documentation files:
  - `README.md`
  - `VERSIONING.md`
  - `contract_versioning.md`
  - `LICENSE` (MIT)
