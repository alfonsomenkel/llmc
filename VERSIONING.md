# Contract Versioning Policy

This document defines how **contracts** are versioned in `llmc`.

Contracts define semantic truth.  
Correct versioning is mandatory for deterministic behavior.

---

## 1. What is versioned

Only **contracts** are versioned.

- Contracts define validation semantics
- Outputs are not versioned
- Implementation details are not versioned
- Tests and fixtures are not versioned independently

---

## 2. Version format

Contracts use a **monotonically increasing integer** version.

Example:

```json
{
  "contract": "user_list",
  "version": 2
}
```
