//! Aether Studio — World Music Production
//! Entry point: starts the audio engine, then launches the Iced UI.

mod components;
mod engine;
mod plugin_gui;
mod project;
mod screens;
mod theme;

use engine::Engine;
use iced::keyboard::key::Named;
use iced::{Element, Subscription, Task};
use project::Project;
use screens::{
    launch::{self, LaunchScreen},
    studio::{self, Studio},
};

fn main() -> iced::Result {
    iced::application("Aether Studio", AetherStudio::update, AetherStudio::view)
        .window_size((1440.0, 900.0))
        .theme(|_| iced::Theme::Dark)
        .subscription(AetherStudio::subscription)
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
    Tick,
    KeyPressed(iced::keyboard::Key),
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

    fn subscription(&self) -> Subscription<Message> {
        let tick = if let Screen::Studio(s) = &self.screen {
            if s.transport.is_playing {
                // 60fps tick for playhead animation
                iced::time::every(std::time::Duration::from_millis(16))
                    .map(|_| Message::Tick)
            } else {
                Subscription::none()
            }
        } else {
            Subscription::none()
        };

        let keyboard = iced::keyboard::on_key_press(|key, _mods| {
            Some(Message::KeyPressed(key))
        });

        Subscription::batch([tick, keyboard])
    }
}

// ── Update ────────────────────────────────────────────────────────────────────

impl AetherStudio {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                if let Screen::Studio(ref mut studio) = self.screen {
                    if studio.transport.is_playing {
                        // Advance playhead: beats_per_sec * 16ms / beats_per_bar
                        let bpm = studio.project.bpm;
                        let beats_per_sec = bpm / 60.0;
                        let delta_secs = 0.016_f32;
                        let beats_per_bar = 4.0_f32;
                        studio.playhead_bar += (beats_per_sec * delta_secs) / beats_per_bar;
                        // Update elapsed time display
                        let elapsed_beats = studio.playhead_bar * beats_per_bar;
                        let elapsed_secs = elapsed_beats / beats_per_sec;
                        let mins = (elapsed_secs / 60.0) as u32;
                        let secs = (elapsed_secs % 60.0) as u32;
                        studio.transport.elapsed_str = format!("{:02}:{:02}", mins, secs);
                    }
                }
            }

            Message::KeyPressed(key) => {
                // Space = play/stop toggle
                if key == iced::keyboard::Key::Named(Named::Space) {
                    if let Screen::Studio(_) = &self.screen {
                        let msg = if let Screen::Studio(s) = &self.screen {
                            if s.transport.is_playing {
                                studio::Message::Transport(
                                    screens::studio::transport_stop_msg(),
                                )
                            } else {
                                studio::Message::Transport(
                                    screens::studio::transport_play_msg(),
                                )
                            }
                        } else {
                            return Task::none();
                        };
                        if let Screen::Studio(ref mut studio) = self.screen {
                            studio.update(msg, &mut self.engine);
                        }
                    }
                }
            }

            Message::Launch(msg) => {
                if let Screen::Launch(ref mut launch) = self.screen {
                    match msg {
                        launch::Message::NewProject => {
                            launch.show_dialog = true;
                        }
                        launch::Message::OpenProject => {
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

