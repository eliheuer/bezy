//! The cursor resource and plugin.
//!
//! This provides a centralized resource, `CursorInfo`, which contains the cursor's
//! position in both screen space and design space. A system runs in `PreUpdate`
//! to ensure this resource is up-to-date before any other systems rely on it.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::rendering::cameras::DesignCamera;
use crate::geometry::design_space::DPoint;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorInfo>()
            .add_systems(PreUpdate, update_cursor_info_system);
    }
}

/// A resource that holds the cursor's position in both screen and design space.
/// This is updated by `update_cursor_info_system` in the `PreUpdate` stage.
#[derive(Resource, Default, Debug)]
pub struct CursorInfo {
    pub screen_position: Option<Vec2>,
    pub design_position: Option<DPoint>,
}

fn update_cursor_info_system(
    mut cursor: ResMut<CursorInfo>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    let Ok(window) = windows.single() else {
        cursor.screen_position = None;
        cursor.design_position = None;
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.single() else {
        cursor.screen_position = None;
        cursor.design_position = None;
        return;
    };

    if let Some(screen_pos) = window.cursor_position() {
        cursor.screen_position = Some(screen_pos);
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
            cursor.design_position = Some(DPoint::new(world_pos.x, world_pos.y));
        } else {
            cursor.design_position = None;
        }
    } else {
        cursor.screen_position = None;
        cursor.design_position = None;
    }
} 