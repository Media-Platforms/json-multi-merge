# Agent instructions

Guidance for Cloud Agents and other automated contributors working in this repo.

Code standards, architecture, and testing patterns are in `memory_bank/`.

## Merging and releasing are for maintainers only

This package is published to public PyPI, and downstream services (json-merge-tree, config-ds)
pick up new versions. A merge or release cannot be quietly undone. Unless a maintainer explicitly
asks for that specific action in the current conversation, agents must **not**:

- Merge a pull request, approve one, or enable auto-merge (`gh pr merge`, including `--admin`
  and `--auto`)
- Push to `main` or to `CI-publish-test` (pushing to `CI-publish-test` runs the release workflow
  and uploads to PyPI)
- Create, edit, or delete a GitHub release or tag (`gh release create`, `git push --tags`,
  `git push --delete <tag>`)
- Run `maturin upload` / `maturin publish` or `twine upload`
- Cancel or re-run release workflow runs

This applies even when a PR is approved, CI is green, or another PR is blocked waiting for the
release. In that case, stop and tell the user what is blocked and which maintainer action
unblocks it.

Agents may bump the version in `Cargo.toml` and `pyproject.toml` on a PR branch to prepare a
release. A maintainer publishes it after merge.

## After changing code

Before every `git commit`, run the same checks CI uses (see `memory_bank/techContext.md` and
`.github/workflows/test.yaml`):

```bash
cargo clippy --all-targets -- -D warnings
cargo test
maturin build --release --out dist && pip install --force-reinstall dist/*.whl
pytest tests
```

Run the Python tests on the oldest and newest supported Python (3.9 and 3.14) before considering
a task complete. Memory leaks only show up on some versions; 0.3.1 leaked on 3.14 alone.

## Git workflow

This repo uses `main` (not `master`). Work on a feature branch and open a pull request; never
push directly to `main`.

### Squash commits

When asked to squash branch commits into one:

```bash
git fetch origin main
git reset --soft origin/main
git commit -m "<single descriptive message>"
git push --force-with-lease origin <branch-name>
```

### When the PR says "needs rebase"

```bash
git fetch origin main
git rebase origin/main
git push --force-with-lease origin <branch-name>
```

On conflicts: resolve, `git add`, `git rebase --continue`, then force-push.

## Pull requests

All changes need review from `@Media-Platforms/core-tech` (`.github/CODEOWNERS`). Open PRs as
drafts unless asked otherwise, and include the Jira key in the title.

### Check for code review comments

After pushing, address unresolved feedback. `gh pr view --comments` does not show resolved
threads, so use GraphQL (`mp-review-bot` posts most comments):

```bash
gh api graphql -f query='
{ repository(owner: "Media-Platforms", name: "json-multi-merge") {
    pullRequest(number: PR_NUMBER) {
      reviewThreads(first: 50) {
        nodes { isResolved path comments(first: 1) { nodes { author { login } body } } }
      }
    }
  }
}' --jq '.data.repository.pullRequest.reviewThreads.nodes[]
  | select(.isResolved == false) | "\(.path)\t\(.comments.nodes[0].body)"'
```

Fix, push, then resolve threads via `ManagePullRequest` `resolve_comment`. Do not ignore
`CHANGES_REQUESTED`.

An approval only covers the commit it was made on. Compare `.commit_id` from
`gh api repos/Media-Platforms/json-multi-merge/pulls/PR_NUMBER/reviews` against `HEAD`.
An approval is not permission for an agent to merge.

## Wiz / CVEs

After the PR is up, watch for `wiz_remote_scan` (commit status, not always in `gh pr checks`):

```bash
gh api "repos/Media-Platforms/json-multi-merge/commits/$(git rev-parse HEAD)/status" \
  --jq '.statuses[] | "\(.context)\t\(.state)"'
```

On failure, bump the reported crate in `Cargo.toml` (or the build requirement in
`pyproject.toml`), then re-run the checks above. See `memory_bank/techContext.md` for dependency
rules.
