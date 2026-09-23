//! Library entry points for embedding RTK filters.
// The lib target mounts the same module tree as the binary so
// `resolve_filter` and the core filter API expose the exact filter
// functions the CLI uses, without duplicating or shim-ing modules. The
// tree carries CLI runners too; they are simply not called by the lib
// API (dead_code is allowed for that reason).
#![allow(dead_code)]

pub mod pipe_cmd {
    //! Guarded library access to the CLI filters.
    //!
    //! # Why `apply` exists
    //!
    //! `resolve_filter` returns the raw filter functions. The `rtk pipe`
    //! CLI runner wraps them in a safety envelope before printing — the
    //! 10 MiB `RAW_CAP` input limit (`src/core/stream.rs`) and the
    //! `never_worse` output guard (`src/core/guard.rs`) — and the CLI
    //! guarantees each filter's expected input dialect by rewriting the
    //! command before running it (`go test -json`, prettier check mode,
    //! vitest reporter flags). A library consumer that filters
    //! *already-captured* command output has none of that, which
    //! corpus testing against this fork exposed as three hazards:
    //!
    //! 1. **Verdicts on misparsed input.** Verdict-style filters
    //!    summarize input they do not recognize instead of passing it
    //!    through: plain-text `go test` output becomes "Go test: No tests
    //!    found", off-format prettier output becomes "All files formatted
    //!    correctly", a bare `error TS2688:` config error becomes
    //!    "TypeScript compilation completed". Each is a confident lie the
    //!    caller cannot distinguish from a real result.
    //! 2. **Growth on small inputs.** Reformats (grouped grep) can exceed
    //!    the raw size, paying tokens for decoration.
    //! 3. **Degenerate collapses.** Pathological input (a 1 MiB
    //!    single-line diff) can collapse to near-empty output silently.
    //!
    //! `apply` closes all three for any consumer: the CLI envelope
    //! (RAW_CAP, `never_worse`), panic isolation, an empty-output guard,
    //! and — the piece the CLI gets from arg rewriting — an
    //! input-dialect gate: the filter only runs when the input matches
    //! the dialect its parser actually accepts (see `recognizes`).
    //! Every rejection returns `None` ("use the raw output"), so `apply`
    //! can only compress or decline, never invent.
    //!
    //! Filters without a confidently derivable dialect (vitest's pipe
    //! filter, the PHP-family tools, log/json) are not exposed through
    //! `apply` at all; establish their accepted input shape from the
    //! parser code before adding them to `recognizes`.

    use crate::cmds::system::pipe_cmd::resolve_filter;

    /// Unguarded raw filter access, as used by the CLI runner. Library
    /// consumers should prefer [`apply`].
    pub use crate::cmds::system::pipe_cmd::resolve_filter as resolve_filter_raw;

    /// Runs `name` over `input` with the full safety envelope. Returns
    /// `None` when the input is unfiltered material: over `RAW_CAP`,
    /// unrecognized dialect, unknown filter, filter panic, empty output,
    /// or an output that is not smaller than the input.
    pub fn apply(name: &str, input: &str) -> Option<String> {
        if input.len() > crate::core::stream::RAW_CAP {
            return None;
        }
        // Gate on an ANSI-stripped view: real tool output is colored, and
        // escape codes split the very tokens the predicates look for
        // (tsc wraps "error" and "TS2322" in separate sequences). The
        // filter itself still receives the raw input.
        let stripped = crate::core::utils::strip_ansi(input);
        if !recognizes(name, &stripped) {
            return None;
        }
        let filter = resolve_filter(name)?;
        let filtered = std::panic::catch_unwind(|| filter(input)).ok()?;
        // Same guard the pipe CLI runner applies; ties keep the filtered
        // form (upstream semantics).
        let guarded = crate::core::guard::never_worse(input, &filtered);
        if guarded.trim().is_empty() && !input.trim().is_empty() {
            return None;
        }
        if guarded == input {
            // Passthrough or never_worse reversion: no compression.
            return None;
        }
        Some(guarded.to_owned())
    }

    /// Input-dialect gate. Each predicate encodes the dialect the
    /// filter's parser accepts in `src/cmds/**`, tightened by corpus
    /// evidence; follow the parser when it changes. Unknown names and
    /// names without a derivable dialect are rejected — the safe
    /// direction is always "do not filter".
    fn recognizes(name: &str, input: &str) -> bool {
        match name {
            // git-diff parses unified `diff --git` file headers.
            "git-diff" => input.lines().any(|l| l.starts_with("diff --git ")),
            // git-status parses porcelain short lines: `XY path` codes,
            // `?? untracked`, `## branch` headers.
            "git-status" => input.lines().any(|l| {
                let l = l.trim_end();
                l.starts_with("## ")
                    || (l.len() >= 4
                        && is_porcelain_code(&l[..2])
                        && l.as_bytes()[2] == b' ')
            }),
            // git-log parses `commit <sha>` records.
            "git-log" => input.lines().any(|l| l.starts_with("commit ")),
            // grep/rg parse `path:line:content` match lines — the same
            // shape grep_wrapper itself keys on.
            "grep" | "rg" => input.lines().any(is_match_line),
            // find/fd emit path lists (relative ./ or absolute).
            "find" | "fd" => input
                .lines()
                .filter(|l| !l.trim().is_empty())
                .all(|l| l.starts_with("./") || l.starts_with('/')),
            // cargo-test parses `running N tests` blocks and
            // `test result:` summaries.
            "cargo-test" => {
                (input.contains("running ") && input.contains(" tests"))
                    || input.contains("test result:")
            }
            // pytest prints a `test session starts` banner every run and
            // a `short test summary info` section on failure.
            "pytest" => {
                input.contains("test session starts")
                    || input.contains("short test summary info")
            }
            // The tsc parser accepts located error forms —
            // `path:line:col - error TS####` (pretty) and
            // `path(line,col): error TS####` (default). Bare `error TS`
            // config errors are NOT counted by the parser and previously
            // produced a false "compilation completed" verdict, so the
            // located forms are required explicitly.
            "tsc" => input.lines().any(|l| {
                l.contains("error TS")
                    && (l.contains(" - ") || (l.contains('(') && l.contains(')')))
            }),
            // mypy marks `path:line: error:`.
            "mypy" => input.lines().any(|l| l.contains(": error:")),
            // The ruff pipe filters parse ruff's JSON output.
            "ruff-check" | "ruff-format" => input.trim_start().starts_with('['),
            // go-test parses `go test -json` event lines; the CLI
            // produces them by rewriting the command. Plain-text
            // `go test` output is NOT that dialect.
            "go-test" => input.lines().any(|l| l.trim_start().starts_with("{\"Action\"")),
            // go-build parses `# package` headers and error lines
            // (plain text; no rewriting involved).
            "go-build" => {
                input.lines().any(|l| l.starts_with("# ")) || input.contains("error[")
            }
            // prettier's parser switches on the check-mode header line.
            "prettier" => input.lines().any(|l| l.contains("Checking formatting")),
            _ => false,
        }
    }

    fn is_porcelain_code(pair: &str) -> bool {
        // Either column may be blank (" M", "D "), but not both.
        pair.chars().all(|c| " MADRCUT?!".contains(c)) && pair != "  "
    }

    fn is_match_line(line: &str) -> bool {
        let mut parts = line.splitn(3, ':');
        let (Some(_path), Some(line_no), Some(_rest)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return false;
        };
        line_no.parse::<usize>().is_ok()
    }
}

#[path = "analytics/mod.rs"]
pub(crate) mod analytics;
#[path = "agent_target.rs"]
pub(crate) mod agent_target;
#[path = "cmds/mod.rs"]
pub(crate) mod cmds;
#[path = "core/mod.rs"]
pub mod core;
#[path = "discover/mod.rs"]
pub(crate) mod discover;
#[path = "hooks/mod.rs"]
pub(crate) mod hooks;
#[path = "learn/mod.rs"]
pub(crate) mod learn;
#[path = "parser/mod.rs"]
pub(crate) mod parser;

// Same crate-root re-exports as the binary's main.rs: the command modules
// reference each other through these flat `crate::<name>` paths. `pipe_cmd`
// is re-exported as a public module below instead.
#[allow(unused_imports)]
use cmds::cloud::{aws_cmd, container, curl_cmd, psql_cmd, wget_cmd};
#[allow(unused_imports)]
use cmds::dotnet::{binlog, dotnet_cmd, dotnet_format_report, dotnet_trx};
#[allow(unused_imports)]
use cmds::git::{diff_cmd, gh_cmd, git_cmd, glab_cmd, gt_cmd};
#[allow(unused_imports)]
use cmds::go::{go_cmd, golangci_cmd};
#[allow(unused_imports)]
use cmds::js::{
    bun_cmd, deno_cmd, lint_cmd, next_cmd, npm_cmd, playwright_cmd, pnpm_cmd, prettier_cmd,
    prisma_cmd, tsc_cmd, vitest_cmd,
};
#[allow(unused_imports)]
use cmds::jvm::{gradlew_cmd, mvn_cmd};
#[allow(unused_imports)]
use cmds::php::{
    ecs_cmd, paratest_cmd, pest_cmd, php_cmd, phpstan_cmd, phpt_cmd, phpunit_cmd, pint_cmd,
};
#[allow(unused_imports)]
use cmds::python::{mypy_cmd, pip_cmd, pytest_cmd, ruff_cmd, sqlfluff_cmd, uv_cmd};
#[allow(unused_imports)]
use cmds::ruby::{rake_cmd, rspec_cmd, rubocop_cmd};
#[allow(unused_imports)]
use cmds::rust::{cargo_cmd, runner};
#[allow(unused_imports)]
use cmds::scala::sbt_cmd;
#[allow(unused_imports)]
use cmds::system::{
    ast_grep_cmd, ctest_cmd, deps, env_cmd, find_cmd, format_cmd, json_cmd, local_llm, log_cmd, ls,
    read, search, summary, tree, wc_cmd,
};

// Private compatibility shim for the vitest filter module, which pattern-
// matches the CLI enum without field access; only the variants it (and the
// other lib-mounted filters) inspect are needed.
#[allow(dead_code)]
pub(crate) enum Commands {
    Vitest {},
    Jest {},
    Other,
}

// Root-visible like the binary's main.rs so `crate::AgentTarget` resolves
// inside the hooks modules' tests.
#[allow(unused_imports)]
use agent_target::AgentTarget;

#[doc(hidden)]
pub use core::{tracking, utils};
