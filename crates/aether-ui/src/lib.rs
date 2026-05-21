// Aether Studio - Audio Software Development Platform
// This is the main UI crate for Aether Studio

pub mod dsp_graph;
pub mod ide;
pub mod plugin_gui;
pub mod project;
pub mod theme;
pub mod widgets;
pub mod workspace;

pub use project::ProjectType;
pub use workspace::Workspace;
