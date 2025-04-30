mod app;
mod config;
mod event_logic;
mod event_ui;
mod style_utils;
mod time_format;
mod timer_logic;
mod timer_provider;
mod timer_service;
mod view_components;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    sycamore::render(App);
}
