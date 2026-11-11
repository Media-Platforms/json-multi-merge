# Product Context

## Why this library exists

Media OS stores configuration (themes, widgets, feature flags) as JSON at several levels of a
hierarchy, and services need one effective JSON document. A pure-Python deep merge was slow for
large or deeply nested documents. json-multi-merge does the merge in Rust and adds key modifiers
so that a child level can remove or replace inherited values without schema changes.

## Problems it solves

1. **Fast deep merges** of large, nested config documents
2. **Override semantics** in the data itself: remove a key (`--`), replace instead of merge
   (`!`), append to history lists (`-history`)
3. **One shared implementation** of those rules for json-merge-tree and services that call
   `merge()` directly (for example config-ds partial schema PATCHes)

## How it should work

1. The caller passes dicts in priority order, lowest first.
2. Each dict is converted to `serde_json::Value`, merged into the running result, and the final
   value is converted back to a new Python dict.
3. Inputs are never mutated, and the returned objects are new.

## User experience goals

- **Service developers**: a drop-in `merge()` with predictable, documented rules (`README.md`).
- **Platform operations**: no per-call memory growth. A leaked reference per call (as in 0.3.1 on
  Python 3.14) multiplies worker memory under load.
