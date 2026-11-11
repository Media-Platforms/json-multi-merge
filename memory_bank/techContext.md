# Technical Context

## Stack

| Technology | Role |
| --- | --- |
| Rust (stable, edition 2021) | Implementation language |
| PyO3 0.30 (`abi3-py39`) | Python bindings, stable ABI wheels |
| serde_json | Intermediate JSON value type |
| maturin >= 1.9, < 2.0 | Build backend (`pyproject.toml`), wheel builds and upload in CI |
| Python 3.9+ | Supported runtime (CI tests 3.9–3.14) |
| pytest | Python behaviour and leak tests |
| clippy | Rust lint (`-D warnings` in CI) |

PyO3's `extension-module` feature is enabled by maturin (`[tool.maturin] features` in
`pyproject.toml`), not in `Cargo.toml`, so `cargo test` can link against libpython.

## Local development

```bash
cargo clippy --all-targets -- -D warnings
cargo test                      # Rust unit tests for merge_json_objects
pip install "maturin>=1.9,<2.0" pytest
maturin build --release --out dist && pip install --force-reinstall dist/*.whl
pytest tests                    # behaviour + leak regression tests
```

`cargo test` links against libpython. If the system Python has no shared library, set
`PYO3_PYTHON` to an interpreter that has one, and add its `lib/` directory to
`LD_LIBRARY_PATH`. `uv python install 3.9 3.14` provides suitable interpreters.

## CI

- **`.github/workflows/test.yaml`** runs on pull requests and pushes to `main`. A `rust` job
  runs clippy and `cargo test`. A `python` job builds the wheel and runs pytest on 3.9, 3.10,
  3.11, 3.12, 3.13 and 3.14.
- **`.github/workflows/release.yaml`** runs when a GitHub release is published, and on pushes
  to `CI-publish-test`. It builds wheels for Linux (manylinux and musllinux; x86_64, x86,
  aarch64, armv7), Windows (x64, x86) and macOS (`macos-15`, aarch64 and x86_64), plus an sdist.
  The `test-wheels` job runs pytest against the Linux x86_64 wheel on 3.9–3.14. `Release`
  uploads to public PyPI with `PYPI_API_TOKEN`. It waits on linux, windows, sdist and
  test-wheels, but not on macOS, because the org runner policy has rejected some macOS labels.

## Dependency management

- Keep the version in **`Cargo.toml`** and **`pyproject.toml`** identical. `Cargo.lock` is
  gitignored.
- When adding a new Python version, check PyO3 supports it before adding it to the CI matrix.
  The abi3 wheel will install on it regardless.
- Raising the abi3 floor (`abi3-pyXY`) also means raising `requires-python` and the classifiers
  in `pyproject.toml`.

## Testing patterns

- Rust tests in `src/lib.rs` cover the merge rules on `serde_json::Value`, with no Python
  involved.
- `tests/test_merge.py` covers the Python surface: modifiers, nesting, lists, value types,
  `TypeError` cases, and inputs not being mutated.
- Leak regression tests call `merge()` 1,000 times. They check that `gc.collect()` leaves no
  marked dicts alive, and that input reference counts do not grow. Keep a list value in the
  marked dicts so the garbage collector tracks them.

## Agent workflow

When behaviour or docs that agents rely on change, update the matching **`memory_bank/`** file
in the same pull request. **`AGENTS.md`** at the repo root covers commit, PR and release
rules; do not duplicate long architecture prose there.
