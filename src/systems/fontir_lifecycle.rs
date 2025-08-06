//! FontIR-based application lifecycle systems
//!
//! This module contains systems that handle font loading and management
//! using FontIR instead of the old custom data structures.

use crate::core::cli::CliArgs;
use crate::core::state::{FontIRAppState, TextEditorState};
use bevy::prelude::*;
use fontir::source::Source;

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
    mut visual_update_tracker: ResMut<crate::rendering::unified_glyph_editing::SortVisualUpdateTracker>,
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
}

