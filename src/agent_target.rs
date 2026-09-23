//! Shared `AgentTarget` CLI enum, mounted by both the binary and the
//! library crate roots.
use clap::ValueEnum;

/// Target agent for hook installation.
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum AgentTarget {
    /// Claude Code (default)
    Claude,
    /// Cursor Agent (editor and CLI)
    Cursor,
    /// Trae IDE
    Trae,
    /// Windsurf IDE (Cascade)
    Windsurf,
    /// Cline / Roo Code (VS Code)
    Cline,
    /// Kilo Code
    Kilocode,
    /// Google Antigravity
    Antigravity,
    /// Kimi AI
    Kimi,
    /// Pi coding agent
    Pi,
    /// Hermes CLI
    Hermes,
    /// Factory Droid CLI
    Droid,
    /// Mistral Vibe CLI
    Vibe,
    /// Oh My Pi (OMP)
    Omp,
}
