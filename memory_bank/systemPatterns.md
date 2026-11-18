# System Patterns

## Architecture

json-multi-merge is a **stateless extension module**. It runs in the caller's process; there is
no service, I/O, or global state.

```
  Python caller            json_multi_merge (Rust, PyO3)
  ┌────────────┐   dicts   ┌──────────────────────────────────────────┐
  │ merge(...) │──────────▶│ py_object_to_json ─▶ merge_json_objects  │
  │            │◀──────────│ json_to_py_object ◀─ (serde_json::Value) │
  └────────────┘ new dict  └──────────────────────────────────────────┘
```

## Core components

1. **`merge`** (`#[pyfunction]`, variadic): checks that every argument is a `PyDict`, converts
   each one, and folds them into the result with `merge_json_objects`, starting from `{}`.
2. **`py_object_to_json`**: dict, list, str, bool, int (i64), float, and None map to
   `serde_json::Value`. Anything else raises `TypeError("Unsupported type")`. Dict keys must be
   `str`. `bool` is checked before `int`.
3. **`json_to_py_object`**: builds new Python objects from a `Value`.
4. **`merge_json_objects`** / **`process_key`**: the pure-Rust merge rules, with no Python
   dependency, unit-tested in `src/lib.rs`.

## Merge rules

| Key form | Action |
| --- | --- |
| `key--` | Remove `key` from the result |
| `key!` | Replace `key` with the new value (merged against null, so nested modifiers still apply) |
| `key-history` | If both values are lists, concatenate them. If the old value is missing, use the new list. Otherwise merge normally. The key keeps its `-history` suffix. |
| `key` | Deep merge: objects merge recursively, while lists and scalars are replaced by the newer value |

## Design constraints

- **Bindings use the PyO3 `Bound<'py, T>` API.** Do not reintroduce GIL Refs (`&PyAny`,
  `&PyTuple`). PyO3 0.20 GIL Refs caused the Python 3.14 leak fixed in 0.3.2.
- **abi3 wheels**: one `cp39-abi3` wheel per platform covers every Python version from 3.9 up.
  pip installs it on newer interpreters without warning, so CI must test each new Python release
  (the `test.yaml` matrix and the `test-wheels` release job).
- **Integers** outside the i64 range fall through to f64 extraction. This is existing behaviour,
  and changing it changes merge output.
