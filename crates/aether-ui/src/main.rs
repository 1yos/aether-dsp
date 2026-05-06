mod app_state;
mod engine;
mod theme;
mod daw_app;

use app_state::create_app_state;
use daw_app::DawApp;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    let state = create_app_state();
    engine::start(state.clone());
    iced::application("Aether Studio", DawApp::update, DawApp::view)
        .window_size((1440.0, 900.0))
        .theme(|_| iced::Theme::Dark)
        .run_with(move || DawApp::new(state))
}
