//! Auto-screenshot system: captures a screenshot after N frames and exits.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

/// Resource controlling automatic screenshot capture.
/// Counts down frames, takes a screenshot, then exits.
#[derive(Resource)]
pub struct ScreenshotMode {
    pub path: PathBuf,
    pub frames_remaining: u32,
    pub captured: bool,
}

impl ScreenshotMode {
    pub fn new(path: PathBuf, delay_frames: u32) -> Self {
        Self {
            path,
            frames_remaining: delay_frames,
            captured: false,
        }
    }
}

pub fn auto_screenshot_system(
    mut commands: Commands,
    mut mode: ResMut<ScreenshotMode>,
    mut exit: EventWriter<AppExit>,
) {
    if mode.captured {
        exit.send(AppExit::Success);
        return;
    }

    if mode.frames_remaining > 0 {
        mode.frames_remaining -= 1;
        return;
    }

    let path = mode.path.clone();
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
    mode.captured = true;
}

pub fn screenshot_plugin(app: &mut App) {
    app.add_systems(Update, auto_screenshot_system);
}
