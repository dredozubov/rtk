# Fork Notice

This repository is a public fork of `rtk-ai/rtk`, maintained to expose RTK filters as a Rust library crate. The original work is `rtk-ai/rtk` by its upstream contributors. This fork is not an official `rtk-ai/rtk` distribution.

License: Apache-2.0. Keep `LICENSE` and upstream copyright/attribution notices with source or binary distributions. If upstream adds a `NOTICE` file, preserve relevant notices.

Fork modifications:

- Added a `[lib]` target exposing `rtk::core::filter` and `rtk::pipe_cmd::resolve_filter`; the lib mounts the same module tree as the binary (plus the binary's crate-root re-exports) so filters are shared, not duplicated.
- Updated fork package metadata to identify `github.com/dredozubov/rtk`.
- Fixed build/test blockers found while validating the fork branch.

For commercial redistribution, also scan transitive dependency licenses and include required third-party notices.
