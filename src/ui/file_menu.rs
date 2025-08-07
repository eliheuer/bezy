//! Cross-platform file menu implementation
//!
//! Provides keyboard-based file menu functionality that works reliably across
//! all platforms without threading complexity.

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use crate::core::state::fontir_app_state::FontIRAppState;
use crate::ui::panes::file_pane::FileInfo;
use crate::core::state::{OutlineData, ContourData, PointData, PointTypeData, GlyphData};
use std::path::PathBuf;
use norad::{Font as NoradFont, designspace::DesignSpaceDocument};
use kurbo::PathEl;

// ============================================================================
// EVENTS
// ============================================================================

/// Event fired when the user triggers a save action from the menu
#[derive(Event)]
pub struct SaveFileEvent;

/// Resource to track the file menu state
#[derive(Resource)]
pub struct FileMenuState {
    pub initialized: bool,
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct FileMenuPlugin;

impl Plugin for FileMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveFileEvent>()
            .insert_resource(FileMenuState { initialized: false })
            .add_systems(Startup, setup_file_menu)
            .add_systems(Update, (
                handle_keyboard_shortcuts,
                handle_save_file_events,
                update_save_state,
            ));
    }
}

// ============================================================================
// MENU SETUP
// ============================================================================

/// Sets up the file menu system with keyboard shortcuts
fn setup_file_menu(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut file_menu_state: ResMut<FileMenuState>,
) {
    if let Ok(mut window) = windows.single_mut() {
        // Set window title to include app name
        window.title = "Bezy - Font Editor".to_string();
        
        // Initialize keyboard-based file menu
        info!("✅ File menu initialized with cross-platform keyboard shortcuts:");
        info!("   💾 Save: Cmd+S (macOS) or Ctrl+S (Windows/Linux)");
        info!("   ⚡ Reliable keyboard shortcuts work on all platforms");
        
        file_menu_state.initialized = true;
    }
}

// ============================================================================
// MENU EVENT HANDLING
// ============================================================================

/// Handles keyboard shortcuts for file operations
fn handle_keyboard_shortcuts(
    mut save_events: EventWriter<SaveFileEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    file_menu_state: Res<FileMenuState>,
) {
    if !file_menu_state.initialized {
        return;
    }
    
    // Handle Cmd+S (macOS) or Ctrl+S (Windows/Linux)
    let cmd_or_ctrl = keyboard_input.pressed(KeyCode::SuperLeft) 
        || keyboard_input.pressed(KeyCode::SuperRight)
        || keyboard_input.pressed(KeyCode::ControlLeft) 
        || keyboard_input.pressed(KeyCode::ControlRight);
    
    if cmd_or_ctrl && keyboard_input.just_pressed(KeyCode::KeyS) {
        info!("💾 Save shortcut triggered (Cmd+S/Ctrl+S)");
        save_events.write(SaveFileEvent);
    }
}

/// Handles save file events
fn handle_save_file_events(
    mut save_events: EventReader<SaveFileEvent>,
    fontir_state: Option<Res<FontIRAppState>>,
    mut file_info: ResMut<FileInfo>,
) {
    for _event in save_events.read() {
        if let Some(state) = fontir_state.as_ref() {
            match save_font_files(&state.source_path, state) {
                Ok(saved_paths) => {
                    info!("Successfully saved {} files", saved_paths.len());
                    for path in &saved_paths {
                        info!("  Saved: {}", path.display());
                    }
                    
                    // Update the last saved time in file info
                    file_info.last_saved = Some(std::time::SystemTime::now());
                }
                Err(e) => {
                    error!("Failed to save files: {}", e);
                }
            }
        } else {
            warn!("No font data to save");
        }
    }
}

/// Updates the save state display
fn update_save_state(
    file_info: Res<FileInfo>,
) {
    // This system can be extended to show save status in the UI
    if file_info.is_changed() {
        if let Some(_last_saved) = file_info.last_saved {
            // Could show a "saved" indicator in the UI
            debug!("Save state updated");
        }
    }
}

// ============================================================================
// SAVE FUNCTIONALITY
// ============================================================================

/// Saves the font files back to disk
fn save_font_files(
    source_path: &PathBuf, 
    fontir_state: &FontIRAppState
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut saved_paths = Vec::new();
    
    // Check if it's a designspace file
    if source_path.extension().and_then(|s| s.to_str()) == Some("designspace") {
        info!("💾 Saving changes to designspace UFO sources...");
        
        // Parse designspace to get UFO source paths
        let designspace = DesignSpaceDocument::load(source_path)?;
        let designspace_dir = source_path.parent().unwrap_or(std::path::Path::new("."));
        
        info!("Found {} UFO sources in designspace", designspace.sources.len());
        
        // Check for modified glyphs in working copies
        let modified_glyphs: Vec<_> = fontir_state.working_copies
            .iter()
            .filter(|((_glyph_name, _location), working_copy)| working_copy.is_dirty)
            .collect();
            
        if modified_glyphs.is_empty() {
            info!("No modified glyphs found - nothing to save");
            return Ok(saved_paths);
        }
        
        info!("Found {} modified glyphs to save", modified_glyphs.len());
        
        // Process each UFO source
        for source in &designspace.sources {
            let ufo_path = designspace_dir.join(&source.filename);
            
            // Check if any modified glyphs belong to this UFO source
            let source_has_changes = modified_glyphs.iter().any(|((_, location), _)| {
                // For now, save to first UFO (regular). In a full implementation,
                // you'd match the location to the correct UFO source
                ufo_path.file_name() == Some(std::ffi::OsStr::new("bezy-grotesk-regular.ufo"))
            });
            
            if source_has_changes {
                info!("Saving changes to UFO: {}", ufo_path.display());
                
                // Load the UFO
                let mut ufo_font = NoradFont::load(&ufo_path)?;
                
                // Update modified glyphs
                for ((glyph_name, _location), working_copy) in &modified_glyphs {
                    if working_copy.is_dirty {
                        info!("  Updating glyph: {}", glyph_name);
                        
                        // Convert BezPath back to UFO glyph using existing conversions
                        let glyph_data = convert_working_copy_to_glyph_data(glyph_name, working_copy);
                        let ufo_glyph = glyph_data.to_norad_glyph();
                        
                        // Update the glyph in the UFO
                        let layer = ufo_font.default_layer_mut();
                        layer.insert_glyph(ufo_glyph);
                    }
                }
                
                // Save the modified UFO
                ufo_font.save(&ufo_path)?;
                saved_paths.push(ufo_path.clone());
                
                info!("✅ Successfully saved UFO: {}", ufo_path.display());
            }
        }
        
    } else if source_path.extension().and_then(|s| s.to_str()) == Some("ufo") {
        // Handle single UFO file
        info!("💾 Saving changes to UFO file: {}", source_path.display());
        
        // Check for modified glyphs
        let modified_glyphs: Vec<_> = fontir_state.working_copies
            .iter()
            .filter(|((_glyph_name, _location), working_copy)| working_copy.is_dirty)
            .collect();
            
        if modified_glyphs.is_empty() {
            info!("No modified glyphs found - nothing to save");
            return Ok(saved_paths);
        }
        
        // Load the UFO
        let mut ufo_font = NoradFont::load(source_path)?;
        
        // Update modified glyphs
        for ((glyph_name, _location), working_copy) in &modified_glyphs {
            if working_copy.is_dirty {
                info!("  Updating glyph: {}", glyph_name);
                
                // Convert BezPath back to UFO glyph using existing conversions
                let glyph_data = convert_working_copy_to_glyph_data(glyph_name, working_copy);
                let ufo_glyph = glyph_data.to_norad_glyph();
                
                // Update the glyph in the UFO
                let layer = ufo_font.default_layer_mut();
                layer.insert_glyph(ufo_glyph);
            }
        }
        
        // Save the modified UFO
        ufo_font.save(source_path)?;
        saved_paths.push(source_path.clone());
        
        info!("✅ Successfully saved UFO: {}", source_path.display());
    }
    
    Ok(saved_paths)
}

/// Convert working copy back to GlyphData using existing conversion infrastructure
fn convert_working_copy_to_glyph_data(
    glyph_name: &str,
    working_copy: &crate::core::state::fontir_app_state::EditableGlyphInstance,
) -> GlyphData {
    let outline = if working_copy.contours.is_empty() {
        None
    } else {
        let mut all_contours = Vec::new();
        
        // Convert each BezPath to ContourData
        for bez_path in &working_copy.contours {
            let mut current_contour = Vec::new();
            
            // Convert BezPath elements to our point format (same logic as fontir_adapter.rs)
            for element in bez_path.elements() {
                match element {
                    PathEl::MoveTo(pt) => {
                        current_contour.push(PointData {
                            x: pt.x,
                            y: pt.y,
                            point_type: PointTypeData::Move,
                        });
                    }
                    PathEl::LineTo(pt) => {
                        current_contour.push(PointData {
                            x: pt.x,
                            y: pt.y,
                            point_type: PointTypeData::Line,
                        });
                    }
                    PathEl::CurveTo(p1, p2, p3) => {
                        // Add off-curve control points and the curve point
                        current_contour.push(PointData {
                            x: p1.x,
                            y: p1.y,
                            point_type: PointTypeData::OffCurve,
                        });
                        current_contour.push(PointData {
                            x: p2.x,
                            y: p2.y,
                            point_type: PointTypeData::OffCurve,
                        });
                        current_contour.push(PointData {
                            x: p3.x,
                            y: p3.y,
                            point_type: PointTypeData::Curve,
                        });
                    }
                    PathEl::QuadTo(p1, p2) => {
                        // Add off-curve control point and quadratic curve point
                        current_contour.push(PointData {
                            x: p1.x,
                            y: p1.y,
                            point_type: PointTypeData::OffCurve,
                        });
                        current_contour.push(PointData {
                            x: p2.x,
                            y: p2.y,
                            point_type: PointTypeData::QCurve,
                        });
                    }
                    PathEl::ClosePath => {
                        // Close current contour - no additional point needed
                    }
                }
            }
            
            // Add contour if it has points
            if !current_contour.is_empty() {
                all_contours.push(ContourData {
                    points: current_contour,
                });
            }
        }
        
        if all_contours.is_empty() {
            None
        } else {
            Some(OutlineData {
                contours: all_contours,
            })
        }
    };
    
    GlyphData {
        name: glyph_name.to_string(),
        advance_width: working_copy.width,
        advance_height: working_copy.height,
        unicode_values: Vec::new(), // Will preserve existing unicode values in actual implementation
        outline,
    }
}