//! FontIR-based application lifecycle systems
//!
//! This module contains systems that handle font loading and management
//! using FontIR instead of the old custom data structures.

use crate::core::cli::CliArgs;
use crate::core::state::{FontIRAppState, TextEditorState};
use bevy::prelude::*;
use fontir::source::Source;

/// Resource to trigger camera centering on the default sort
#[derive(Resource)]
pub struct CenterCameraOnDefaultSort {
    pub center_x: f32,
    pub center_y: f32,
    pub advance_width: f32,
}

/// System to load UFO/designspace font on startup using FontIR
pub fn load_fontir_font(mut commands: Commands, cli_args: Res<CliArgs>) {
    // clap provides the default value, so ufo_path is guaranteed to be Some
    if let Some(path) = &cli_args.ufo_path {
        match FontIRAppState::from_path(path.clone()) {
            Ok(mut app_state) => {
                // Try to load glyphs if possible
                if let Err(e) = app_state.load_glyphs() {
                    warn!("Could not load glyphs: {}", e);
                }

                // Try to load kerning groups
                if let Err(e) = app_state.load_kerning_groups() {
                    warn!("Could not load kerning groups: {}", e);
                }

                info!(
                    "Successfully loaded font with FontIR from: {}",
                    path.display()
                );
                commands.insert_resource(app_state);
            }
            Err(e) => {
                error!("Failed to load font with FontIR: {}", e);
                error!("Font path: {}", path.display());
                error!("The application will continue but some features may not work correctly.");

                // Don't insert any FontIR state if loading fails
                warn!("App will run without FontIR state - some features may not work");
            }
        }
    } else {
        warn!("No font path specified, running without a font loaded.");
        // Don't insert any FontIRAppState resource - systems will need to handle this
    }
}

/// System to create a default LTR text sort for first-time users
/// Runs after font loading to ensure font data is available
pub fn create_default_sort(
    fontir_state: Option<Res<FontIRAppState>>,
    mut text_editor_state: ResMut<TextEditorState>,
    // CRITICAL FIX: Trigger unified renderer update when default sort is created
    _visual_update_tracker: ResMut<crate::rendering::unified_glyph_editing::SortVisualUpdateTracker>,
    mut commands: Commands,
) {
    // Only create default sort if no sorts exist yet
    if !text_editor_state.buffer.is_empty() {
        return;
    }

    // Get the current glyph from FontIR state or use 'a' as fallback
    let glyph_name = if let Some(state) = &fontir_state {
        state.current_glyph.clone().unwrap_or_else(|| "a".to_string())
    } else {
        "a".to_string()
    };

    info!("Creating default LTR text sort for glyph '{}' at center (0,0)", glyph_name);

    // Get advance width from FontIR if available
    let advance_width = if let Some(state) = &fontir_state {
        state.get_glyph_advance_width(&glyph_name)
    } else {
        500.0 // Default fallback
    };

    // Create a SortEntry using the same pattern as manual sort placement
    let default_sort = crate::core::state::text_editor::SortEntry {
        kind: crate::core::state::text_editor::SortKind::Glyph {
            codepoint: Some('a'), // Default to 'a'
            glyph_name: glyph_name.clone(),
            advance_width,
        },
        is_active: true, // Make it active and ready to edit
        layout_mode: crate::core::state::text_editor::SortLayoutMode::Freeform,
        root_position: Vec2::ZERO, // Center of world space
        is_buffer_root: false, // Independent sort, not part of buffer
        buffer_cursor_position: None, // No cursor position for independent sorts
    };

    // Add to the text editor buffer at the end
    let buffer_len = text_editor_state.buffer.len();
    text_editor_state.buffer.insert(buffer_len, default_sort);
    
    // Mark the state as changed to trigger entity spawning
    text_editor_state.set_changed();
    
    // Note: Don't trigger visual update immediately - let the normal change detection handle it
    // after the points are spawned in the Update cycle

    info!("Default sort created for new users - ready to edit!");
    
    // TEMPORARY: Center camera on the visual center of the default glyph
    // The glyph is at (0,0) but we want to center on its visual middle
    // For glyph 'a' with advance width ~592, the visual center is around x=296
    // The glyph also needs vertical centering - estimate based on typical glyph height
    // TO REVERT: Simply comment out the lines below and the camera will center at (0,0)
    let visual_center_x = advance_width / 2.0;
    
    // For vertical centering, use the font metrics to find the visual center
    // Most Latin lowercase letters have their visual center around 200-400 units from baseline
    let visual_center_y = if let Some(_state) = &fontir_state {
        // Try to get x-height or use a reasonable estimate for lowercase 'a'
        // The visual center of 'a' is typically around 350 units above baseline
        // MANUAL ADJUSTMENT: Change this value to move camera up/down
        328.0  // <-- ADJUST THIS VALUE: Higher = camera moves up, Lower = camera moves down
    } else {
        328.0 // Default estimate for lowercase 'a'
    };
    
    // Insert a marker resource to trigger camera centering in the next frame
    commands.insert_resource(CenterCameraOnDefaultSort {
        center_x: visual_center_x,
        center_y: visual_center_y,
        advance_width,
    });
    
    info!("Camera will center on default sort at ({}, {}) (advance width: {})", 
          visual_center_x, visual_center_y, advance_width);
}

/// System to center the camera on the default sort after it's created
/// This runs once and then removes the resource
pub fn center_camera_on_default_sort(
    mut commands: Commands,
    center_resource: Option<Res<CenterCameraOnDefaultSort>>,
    mut camera_query: Query<&mut Transform, With<crate::rendering::cameras::DesignCamera>>,
) {
    if let Some(center) = center_resource {
        // Center the camera on the visual center of the default sort
        for mut transform in camera_query.iter_mut() {
            // Move camera to center on the glyph's visual center
            // The glyph is at (0,0) but its visual center is offset for better viewing
            transform.translation.x = center.center_x;
            transform.translation.y = center.center_y;
            
            info!("Centered camera on default sort at ({}, {}) (advance width: {})", 
                  center.center_x, center.center_y, center.advance_width);
        }
        
        // Remove the resource so this only happens once
        commands.remove_resource::<CenterCameraOnDefaultSort>();
    }
}

