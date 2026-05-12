// Project types and management

use serde::{Deserialize, Serialize};

/// The type of project being created
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Audio plugin (VST3, AU, CLAP, AAX)
    Plugin,
    /// Digital Audio Workstation
    Daw,
    /// Custom DSP node library
    NodeLibrary,
    /// Hardware controller application
    HardwareController,
    /// Audio utility/tool
    Utility,
}

impl ProjectType {
    pub fn name(&self) -> &str {
        match self {
            ProjectType::Plugin => "Audio Plugin",
            ProjectType::Daw => "Digital Audio Workstation",
            ProjectType::NodeLibrary => "DSP Node Library",
            ProjectType::HardwareController => "Hardware Controller",
            ProjectType::Utility => "Audio Utility",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ProjectType::Plugin => "Build VST3, AU, CLAP, or AAX plugins",
            ProjectType::Daw => "Build a complete music production application",
            ProjectType::NodeLibrary => "Create reusable DSP components",
            ProjectType::HardwareController => "Build software for audio hardware",
            ProjectType::Utility => "Build specialized audio tools",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            ProjectType::Plugin => "🎛️",
            ProjectType::Daw => "🎹",
            ProjectType::NodeLibrary => "🔧",
            ProjectType::HardwareController => "🎚️",
            ProjectType::Utility => "🔊",
        }
    }

    pub fn complexity(&self) -> &str {
        match self {
            ProjectType::Plugin => "⭐⭐⭐ Medium",
            ProjectType::Daw => "⭐⭐⭐⭐⭐ Extreme",
            ProjectType::NodeLibrary => "⭐⭐⭐⭐ High",
            ProjectType::HardwareController => "⭐⭐⭐⭐ High",
            ProjectType::Utility => "⭐⭐ Low-Medium",
        }
    }

    pub fn time_to_market(&self) -> &str {
        match self {
            ProjectType::Plugin => "1-4 weeks",
            ProjectType::Daw => "6-24 months",
            ProjectType::NodeLibrary => "1-2 weeks",
            ProjectType::HardwareController => "2-6 months",
            ProjectType::Utility => "1-4 weeks",
        }
    }
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub project_type: ProjectType,
    pub version: String,
    pub author: String,
    pub description: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            project_type: ProjectType::Plugin,
            version: "0.1.0".to_string(),
            author: String::new(),
            description: String::new(),
        }
    }
}
