# json-multi-merge

## General info

**json-multi-merge** is a Core Tech Python package, written in Rust with PyO3, that deep-merges
any number of JSON-compatible dicts. Keys can carry suffix modifiers (`--`, `!`, `-history`) that
change how each value is merged. It is published to public PyPI as `json-multi-merge`.

[json-merge-tree](https://github.com/Media-Platforms/json-merge-tree) is the main consumer. It
calls `merge()` once per merged resource (for example on every config-ds `/merged` request), so
per-call cost and memory behaviour matter.

## Primary API

- **`merge(*objs: dict) -> dict`**: merge dicts left to right. With no arguments it returns `{}`.
  It raises `TypeError` if any argument is not a dict or holds an unsupported value type.

## Repository layout

| Path | Role |
| --- | --- |
| `src/lib.rs` | Python bindings (`merge`, `py_object_to_json`, `json_to_py_object`), merge logic (`merge_json_objects`, `process_key`), and Rust unit tests |
| `tests/test_merge.py` | pytest behaviour tests and leak regression tests |
| `Cargo.toml` | Crate metadata, PyO3 version and abi3 floor |
| `pyproject.toml` | Python package metadata, maturin build backend |
| `.github/workflows/test.yaml` | PR and `main` CI |
| `.github/workflows/release.yaml` | Wheel builds and PyPI upload on GitHub release |

## Expectations for contributors

- Keep the version in **`Cargo.toml` and `pyproject.toml` identical**. Bump both for any change
  that should ship.
- Keep merge behaviour unchanged unless the ticket says otherwise. Both the Rust tests and
  `tests/test_merge.py` pin it.
- Merging and releasing are for maintainers only (see `AGENTS.md`).
- Update **`memory_bank/`** when behaviour, dependencies, or agent workflow changes.
