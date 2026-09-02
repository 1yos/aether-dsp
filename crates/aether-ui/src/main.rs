//! Aether Studio — World Music Production
//! Entry point: starts the audio engine, then launches the Iced UI.

mod components;
mod engine;
mod plugin_gui;
mod project;
mod screens;
mod theme;

use engine::Engine;
use iced::{Element, Task};
use project::Project;
use screens::{
    launch::{self, LaunchScreen},
    studio::{self, Studio},
};

fn main() -> iced::Result {
    iced::application("Aether Studio", AetherStudio::update, AetherStudio::view)
        .window_size((1440.0, 900.0))
        .theme(|_| iced::Theme::Dark)
        .run_with(AetherStudio::new)
}

// ── App state ─────────────────────────────────────────────────────────────────

enum Screen {
    Launch(LaunchScreen),
    Studio(Studio),
}

struct AetherStudio {
    screen: Screen,
    engine: Option<Engine>,
}

#[derive(Debug, Clone)]
enum Message {
    Launch(launch::Message),
    Studio(studio::Message),
}

// ── Init ──────────────────────────────────────────────────────────────────────

impl AetherStudio {
    fn new() -> (Self, Task<Message>) {
        let engine = match Engine::start() {
            Ok(e) => {
                println!("[app] audio engine running at {} Hz", e.sample_rate());
                Some(e)
            }
            Err(err) => {
                eprintln!("[app] audio engine failed to start: {err}");
                None
            }
        };

        (
            Self {
                screen: Screen::Launch(LaunchScreen::default()),
                engine,
            },
            Task::none(),
        )
    }
}

// ── Update ────────────────────────────────────────────────────────────────────

impl AetherStudio {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Launch(msg) => {
                if let Screen::Launch(ref mut launch) = self.screen {
                    match msg {
                        launch::Message::NewProject => {
                            launch.show_dialog = true;
                        }
                        launch::Message::OpenProject => {
                            // No file picker yet — open empty project
                            let project = Project::new("New Project", 120.0);
                            self.screen = Screen::Studio(Studio::new(project));
                        }
                        launch::Message::OpenRecent(name) => {
                            let project = Project::new(&name, 120.0);
                            self.screen = Screen::Studio(Studio::new(project));
                        }
                        launch::Message::ProjectNameChanged(s) => {
                            launch.project_name = s;
                        }
                        launch::Message::BpmChanged(s) => {
                            launch.bpm_str = s;
                        }
                        launch::Message::TuningSelected(t) => {
                            launch.selected_tuning = t;
                        }
                        launch::Message::ConfirmCreate => {
                            let bpm = launch.bpm_str.parse::<f32>().unwrap_or(120.0);
                            let project = Project::new(&launch.project_name, bpm.max(1.0));
                            self.screen = Screen::Studio(Studio::new(project));
                        }
                        launch::Message::CancelCreate => {
                            launch.show_dialog = false;
                        }
                    }
                }
            }
            Message::Studio(msg) => {
                if let Screen::Studio(ref mut studio) = self.screen {
                    studio.update(msg, &mut self.engine);
                }
            }
        }
        Task::none()
    }
}

// ── View ──────────────────────────────────────────────────────────────────────

impl AetherStudio {
    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Launch(launch) => launch.view().map(Message::Launch),
            Screen::Studio(studio) => studio.view().map(Message::Studio),
        }
    }
}
