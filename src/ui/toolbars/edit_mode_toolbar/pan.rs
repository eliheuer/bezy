use super::EditModeSystem;
use crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive;
use bevy::prelude::*;
use bevy_pancam::PanCam;

pub struct PanMode;

impl EditModeSystem for PanMode {
    fn update(&self, commands: &mut Commands) {
        // Ensure select mode is disabled while in pan mode
        commands.insert_resource(SelectModeActive(false));
    }

    fn on_enter(&self) {
        // Enable panning on all PanCam components
        info!("Entering pan mode - enabling camera panning");
    }

    fn on_exit(&self) {
        // Disable panning on all PanCam components
        info!("Exiting pan mode - disabling camera panning");
    }
}

// System to enable/disable the PanCam component when entering/exiting pan mode
pub fn toggle_pancam_on_mode_change(
    mut query: Query<&mut PanCam>,
    current_mode: Res<super::ui::CurrentEditMode>,
) {
    // Only run this system when the current mode changes
    if current_mode.is_changed() {
        let should_enable = matches!(current_mode.0, super::ui::EditMode::Pan);

        for mut pancam in query.iter_mut() {
            // Only log if we're actually changing the state
            if pancam.enabled != should_enable {
                pancam.enabled = should_enable;
                if should_enable {
                    info!("PanCam enabled");
                } else {
                    info!("PanCam disabled");
                }
            }
        }
    }
}
