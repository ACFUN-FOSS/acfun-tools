#![forbid(unsafe_code)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows", feature = "gui"),
    windows_subsystem = "windows"
)]

mod live;

#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "cli")]
mod cli;

use anyhow::Result;

#[cfg(feature = "gui")]
use gui::Online;
#[cfg(feature = "gui")]
use iced::{window, Application, Settings};

#[cfg(feature = "gui")]
fn main() -> Result<()> {
    Online::run(Settings {
        window: window::Settings {
            size: (400, 300),
            ..Default::default()
        },
        ..Default::default()
    })?;

    Ok(())
}

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> Result<()> {
    cli::cli().await
}
