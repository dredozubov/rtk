//! Library entry points for embedding RTK filters.
// The lib target mounts the same module tree as the binary so
// `resolve_filter` and the core filter API expose the exact filter
// functions the CLI uses, without duplicating or shim-ing modules. The
// tree carries CLI runners too; they are simply not called by the lib
// API (dead_code is allowed for that reason).
#![allow(dead_code)]

pub mod pipe_cmd {
    pub use crate::cmds::system::pipe_cmd::resolve_filter;
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
