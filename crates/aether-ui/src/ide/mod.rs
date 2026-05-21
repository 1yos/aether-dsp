// Aether Studio IDE - Professional audio development environment

pub mod code_editor;
pub mod code_generator;
pub mod graph_view;
pub mod project_explorer;
pub mod terminal;
pub mod toolbar;

use crate::project::ProjectConfig;
use std::path::PathBuf;

/// The main IDE state
#[derive(Debug)]
pub struct AetherIDE {
    pub project: Option<ProjectConfig>,
    pub project_path: Option<PathBuf>,
    pub open_files: Vec<OpenFile>,
    pub active_file: Option<usize>,
    pub terminal_output: String,
    pub is_building: bool,
}

/// An open file in the editor
#[derive(Debug, Clone)]
pub struct OpenFile {
    pub path: PathBuf,
    pub content: String,
    pub modified: bool,
    pub language: FileLanguage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileLanguage {
    Rust,
    Toml,
    Markdown,
    Json,
    Other,
}

impl FileLanguage {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => FileLanguage::Rust,
            "toml" => FileLanguage::Toml,
            "md" => FileLanguage::Markdown,
            "json" => FileLanguage::Json,
            _ => FileLanguage::Other,
        }
    }
}

impl AetherIDE {
    pub fn new() -> Self {
        Self {
            project: None,
            project_path: None,
            open_files: Vec::new(),
            active_file: None,
            terminal_output: String::new(),
            is_building: false,
        }
    }

    pub fn create_project(&mut self, config: ProjectConfig, path: PathBuf) {
        self.project = Some(config);
        self.project_path = Some(path);
        self.terminal_output = format!("Created project at {:?}\n", self.project_path);
    }

    pub fn open_file(&mut self, path: PathBuf, content: String) {
        let language = path
            .extension()
            .and_then(|e| e.to_str())
            .map(FileLanguage::from_extension)
            .unwrap_or(FileLanguage::Other);

        let file = OpenFile {
            path,
            content,
            modified: false,
            language,
        };

        self.open_files.push(file);
        self.active_file = Some(self.open_files.len() - 1);
    }

    pub fn close_file(&mut self, index: usize) {
        if index < self.open_files.len() {
            self.open_files.remove(index);
            if let Some(active) = self.active_file {
                if active >= self.open_files.len() {
                    self.active_file = if self.open_files.is_empty() {
                        None
                    } else {
                        Some(self.open_files.len() - 1)
                    };
                }
            }
        }
    }

    pub fn get_active_file(&self) -> Option<&OpenFile> {
        self.active_file.and_then(|i| self.open_files.get(i))
    }

    pub fn get_active_file_mut(&mut self) -> Option<&mut OpenFile> {
        self.active_file.and_then(|i| self.open_files.get_mut(i))
    }

    pub fn update_active_file_content(&mut self, content: String) {
        if let Some(file) = self.get_active_file_mut() {
            file.content = content;
            file.modified = true;
        }
    }
}

impl Default for AetherIDE {
    fn default() -> Self {
        Self::new()
    }
}
