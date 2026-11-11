# Progress

## Current status

| Area | Status |
| --- | --- |
| `merge()` with `--`, `!`, `-history` modifiers | Production / stable |
| Python 3.14 support (no per-call leak) | Fixed in 0.3.2 |
| Rust unit tests + pytest suite | Enforced in CI |
| CI matrix Python 3.9–3.14 | `test.yaml` and release `test-wheels` |
| AGENTS.md + memory bank + CODEOWNERS + PR template | Added (PDT-4366, 0.3.3) |
| Branch protection on `main` | Not enabled (maintainer action) |

## What works

- Variadic deep merge of dicts, with a new dict returned on every call
- Key modifiers and type-conflict resolution, documented in `README.md`
- `TypeError` for non-dict arguments, unsupported value types and non-`str` keys
- abi3 wheels for Linux, Windows and macOS, plus an sdist, published to PyPI on GitHub release

## What is not in scope

- No hierarchy walking or database access (that is json-merge-tree)
- No support for non-JSON Python types (datetime, Decimal, sets, and so on)
- No free-threaded (`abi3t` / 3.14t) wheels

## Known documentation gaps

- `README.md` "API Reference" still shows the two-argument `merge(base, update)` signature.
  The function is variadic, as the "Variadic Input" section says.

## Version history note

The version is declared in **`Cargo.toml`** and **`pyproject.toml`**, which must match.
Publishing a GitHub release runs `release.yaml` and uploads to public PyPI. Only maintainers
publish releases.

| Version | Notes |
| --- | --- |
| 0.3.1 | PyO3 0.20, `cp37-abi3`. Leaks on Python 3.14. |
| 0.3.2 | PyO3 0.30.1, `cp39-abi3`. Leak fixed; CI 3.9–3.14. |
| 0.3.3 | AGENTS.md, memory bank, CODEOWNERS, PR template. No code changes. |
