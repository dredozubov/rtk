# Fork Notice

This repository is a public fork of `rtk-ai/rtk`, maintained to expose RTK filters as a Rust library crate. The original work is `rtk-ai/rtk` by its upstream contributors. This fork is not an official `rtk-ai/rtk` distribution.

License: Apache-2.0. Keep `LICENSE` and upstream copyright/attribution notices with source or binary distributions. If upstream adds a `NOTICE` file, preserve relevant notices.

Fork modifications:

- Upstream's own lib/bin split (2026-10) keeps every module private ("rtk is an application, not a library API"); the fork exposes the minimum embedding surface: the `pub mod pipe_cmd` block (guarded `apply` + raw resolver) and the `pub mod filters` re-export of the source-filter API, both in `src/lib.rs`. Everything else stays private so upstream doc examples stay out of doctest compilation.
- `rtk::pipe_cmd::apply` wraps the filters in a library safety envelope the upstream CLI gets from its runner plus argument rewriting: the 10 MiB RAW_CAP, the `never_worse` guard, panic isolation, an empty-output guard, and a per-filter input-dialect gate (a filter only runs when the input matches the dialect its parser accepts, so misparsed input can never be summarized into a false verdict like "No tests found" or "All files formatted correctly"). See `src/lib.rs` for the full rationale and `tests/library_api.rs` for the gate tests.
- rusqlite bumped to 0.37 in this crate only (ri-engine's pin; single libsqlite3-sys in ri's graph). proxy/ stays on upstream's 0.34.
- Updated fork package metadata to identify `github.com/dredozubov/rtk`.
- Fixed build/test blockers found while validating the fork branch.

Upstream's 2026-10 split moved the compression core into `engine/` under the
Business Source License 1.1, pulled in by the default `engine` feature. The
filter surface this fork exposes (pipe_cmd::apply, filters) lives entirely in
the Apache-2.0 hook core: consume this fork with `default-features = false`
(as ri does) and no BUSL code enters the graph.

For commercial redistribution, also scan transitive dependency licenses and include required third-party notices.
