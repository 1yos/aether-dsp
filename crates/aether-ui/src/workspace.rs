// Main workspace for Aether Studio

use crate::project::ProjectConfig;

/// The main workspace state
#[derive(Debug, Clone)]
pub struct Workspace {
    pub current_project: Option<ProjectConfig>,
    pub active_mode: WorkspaceMode,
}

/// The three primary modes of Aether Studio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceMode {
    /// Welcome screen (project selection)
    Welcome,
    /// DSP Graph editor (visual + code)
    DspGraph,
    /// GUI Designer (drag-and-drop UI builder)
    GuiDesigner,
    /// Project settings (export, testing, etc.)
    ProjectSettings,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            current_project: None,
            active_mode: WorkspaceMode::Welcome,
        }
    }

    pub fn create_project(&mut self, config: ProjectConfig) {
        self.current_project = Some(config);
        self.active_mode = WorkspaceMode::DspGraph;
    }

    pub fn close_project(&mut self) {
        self.current_project = None;
        self.active_mode = WorkspaceMode::Welcome;
    }

    pub fn set_mode(&mut self, mode: WorkspaceMode) {
        self.active_mode = mode;
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}
