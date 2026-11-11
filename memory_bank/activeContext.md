# Active Context

## Current work focus

**PDT-4366** follow-up: add `AGENTS.md`, `memory_bank/` and `.github/CODEOWNERS`, modelled on
json-merge-tree, and bump to **0.3.3**.

## Recent changes

- **0.3.2** (PDT-4366): PyO3 0.20 → 0.30.1 with the `Bound` API, abi3 floor raised to Python 3.9.
  This fixes a leak of one dict per `merge()` call on Python 3.14. It also added the pytest suite
  with leak tests, the `test.yaml` CI matrix (3.9–3.14), and the `test-wheels` release gate.
- Release workflow: macOS builds moved to `macos-15` and no longer gate the PyPI upload.
- An agent merged the 0.3.2 PRs and published the release without maintainer approval. That is
  why `AGENTS.md` now limits merging and releasing to maintainers.
- Added `AGENTS.md`, `memory_bank/` and `.github/CODEOWNERS` (`@Media-Platforms/core-tech`).

## Next steps

1. A maintainer reviews and merges the 0.3.3 PR, then publishes GitHub release `0.3.3`.
2. Enable branch protection on `main` with "Require review from Code Owners", so that
   `CODEOWNERS` actually blocks merges. json-merge-tree's `main` is protected; this repo's was
   not.
3. When new Python versions are released, add them to `test.yaml` and the `test-wheels` matrix.

## Active decisions

- **0.3.3 contains no code changes.** It adds docs and repo metadata only. Merge behaviour and
  wheels are identical to 0.3.2.
- **Agents prepare releases but never publish them** (see `AGENTS.md`).
