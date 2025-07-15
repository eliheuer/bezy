//! Sort entity management for text editor sorts

use bevy::prelude::*;
use crate::core::state::text_editor::TextEditorState;
use crate::core::state::AppState;
use crate::editing::sort::{Sort, InactiveSort, ActiveSort};
use crate::core::state::SortLayoutMode;
use std::collections::HashMap;

/// Component to track which buffer index this entity represents
#[derive(Component)]
pub struct BufferSortIndex(pub usize);

/// Resource to track which buffer sorts have entities
#[derive(Resource, Default)]
pub struct BufferSortEntities {
    pub entities: HashMap<usize, Entity>,
}

/// Initialize text editor sorts
pub fn initialize_text_editor_sorts(
    mut commands: Commands,
) {
    commands.init_resource::<BufferSortEntities>();
    info!("Initialized text editor sorts system");
}

/// Manage sort activation based on mouse clicks
pub fn manage_sort_activation(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::rendering::cameras::DesignCamera>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    sort_query: Query<(Entity, &Transform, &Sort, Option<&BufferSortIndex>)>,
    active_sort_query: Query<Entity, With<crate::editing::sort::ActiveSort>>,
    mut text_editor_state: ResMut<TextEditorState>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
) {
    // System is working - debug confirmed
    // info!("[manage_sort_activation] System called, ui_hover_state.is_hovering_ui = {}", ui_hover_state.is_hovering_ui);
    
    // Don't process clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        info!("[manage_sort_activation] Skipping - hovering over UI");
        return;
    }

    // Check for left mouse click
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        // info!("[manage_sort_activation] No left mouse click this frame");
        return;
    }
    
    // Get cursor position in world coordinates
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };
    
    let Ok(window) = windows.get_single() else {
        return;
    };
    
    info!("[manage_sort_activation] Left mouse button pressed at screen pos: {:?}", window.cursor_position());
    
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        debug!("[manage_sort_activation] Failed to convert cursor to world position");
        return;
    };

    // Check if click is on any sort
    let click_threshold = 200.0; // Increased click tolerance in design units
    
    debug!("[manage_sort_activation] Mouse click at world position ({:.1}, {:.1})", world_position.x, world_position.y);
    debug!("[manage_sort_activation] Checking {} sorts for handle clicks", sort_query.iter().count());
    
    for (entity, transform, sort, buffer_index) in sort_query.iter() {
        let sort_position = transform.translation.truncate();
        let distance = sort_position.distance(world_position);
        
        debug!("Sort '{}' at ({:.1}, {:.1}), distance: {:.1}", 
               sort.glyph_name, sort_position.x, sort_position.y, distance);
        
        if distance <= click_threshold {
            // TODO: Get modifier key state to support multi-select
            // For now, implement single-select behavior (clear others, select this one)
            
            // Clear selection from all other sorts
            let all_sorts = sort_query.iter().map(|(e, _, _, _)| e).collect::<Vec<_>>();
            for other_entity in all_sorts {
                if other_entity != entity {
                    commands.entity(other_entity).remove::<crate::editing::selection::components::Selected>();
                }
            }
            
            // Add this sort to the selection
            commands.entity(entity).insert(crate::editing::selection::components::Selected);
            info!("Selected sort '{}' at position ({:.1}, {:.1})", sort.glyph_name, sort_position.x, sort_position.y);
            
            // Note: Activation will be handled separately based on selection state
            break;
        }
    }
}

/// Spawn missing sort entities for sorts in the text editor buffer
pub fn spawn_missing_sort_entities(
    mut commands: Commands,
    mut text_editor_state: ResMut<TextEditorState>,
    mut buffer_entities: ResMut<BufferSortEntities>,
    app_state: Res<AppState>,
    existing_active_sorts: Query<Entity, With<crate::editing::sort::ActiveSort>>,
) {
    // Iterate through all sorts in the buffer
    for i in 0..text_editor_state.buffer.len() {
        // Skip if we already have an entity for this buffer index
        if buffer_entities.entities.contains_key(&i) {
            continue;
        }

        if let Some(sort_entry) = text_editor_state.buffer.get(i) {
            // Get the visual position for this sort using correct font metrics
            let font_metrics = &app_state.workspace.info.metrics;
            let position = match sort_entry.layout_mode {
                crate::core::state::SortLayoutMode::Text => {
                    if sort_entry.is_buffer_root {
                        // Text roots use their exact stored position
                        Some(sort_entry.root_position)
                    } else {
                        // Non-root text sorts flow from their text root using actual font metrics
                        text_editor_state.get_text_sort_flow_position(
                            i,
                            font_metrics,
                            0.0,
                        )
                    }
                }
                crate::core::state::SortLayoutMode::Freeform => Some(sort_entry.root_position),
            };
            
            if let Some(position) = position {
                // Create Sort component
                let sort = Sort {
                    glyph_name: sort_entry.kind.glyph_name().to_string(),
                    layout_mode: sort_entry.layout_mode.clone(),
                };

                // Spawn entity with Sort, Transform, and InactiveSort components (start inactive)
                let entity = commands.spawn((
                    sort,
                    Transform::from_translation(position.extend(0.0)),
                    InactiveSort, // Start inactive - activation will be handled by selection system
                    BufferSortIndex(i),
                    crate::editing::selection::components::Selectable, // Make sorts selectable
                    Name::new(format!("BufferSort[{}]", i)),
                )).id();

                // Track the entity
                buffer_entities.entities.insert(i, entity);
                
                // OLD: Also activate this sort in the text editor state
                // text_editor_state.activate_sort(i); // Disabled - let selection system handle activation
                
                debug!("Spawned INACTIVE sort entity for buffer index {} at position ({:.1}, {:.1})", 
                       i, position.x, position.y);
            }
        }
    }
}

/// Update positions of existing buffer sort entities to match text flow
pub fn update_buffer_sort_positions(
    text_editor_state: Res<TextEditorState>,
    app_state: Res<AppState>,
    buffer_entities: Res<BufferSortEntities>,
    mut sort_query: Query<&mut Transform, With<BufferSortIndex>>,
) {
    let font_metrics = &app_state.workspace.info.metrics;
    
    // Update Transform positions for all existing buffer sorts
    for (&buffer_index, &entity) in buffer_entities.entities.iter() {
        if let Ok(mut transform) = sort_query.get_mut(entity) {
            // Calculate correct position using the font metrics from app state
            if let Some(sort) = text_editor_state.buffer.get(buffer_index) {
                let new_position = match sort.layout_mode {
                    crate::core::state::SortLayoutMode::Text => {
                        if sort.is_buffer_root {
                            // Text roots use their exact stored position
                            Some(sort.root_position)
                        } else {
                            // Non-root text sorts flow from their text root using actual font metrics
                            text_editor_state.get_text_sort_flow_position(
                                buffer_index,
                                font_metrics,
                                0.0,
                            )
                        }
                    }
                    crate::core::state::SortLayoutMode::Freeform => Some(sort.root_position),
                };
                
                if let Some(new_pos) = new_position {
                    let new_pos_3d = new_pos.extend(transform.translation.z);
                    transform.translation = new_pos_3d;
                    info!("Sort {} positioned at ({:.1}, {:.1})", 
                           buffer_index, new_pos.x, new_pos.y);
                }
            }
        }
    }
}

/// System to auto-activate sorts when exactly one is selected
pub fn auto_activate_selected_sorts(
    mut commands: Commands,
    selected_sorts: Query<Entity, (With<Sort>, With<crate::editing::selection::components::Selected>)>,
    active_sorts: Query<Entity, With<ActiveSort>>,
) {
    let selected_count = selected_sorts.iter().count();
    
    if selected_count == 1 {
        // Exactly one sort is selected - activate it
        if let Ok(selected_sort) = selected_sorts.get_single() {
            // Deactivate all currently active sorts
            for active_entity in active_sorts.iter() {
                commands.entity(active_entity).remove::<ActiveSort>();
                commands.entity(active_entity).insert(InactiveSort);
            }
            
            // Activate the selected sort
            commands.entity(selected_sort).remove::<InactiveSort>();
            commands.entity(selected_sort).insert(ActiveSort);
            
            info!("Auto-activated selected sort {:?}", selected_sort);
        }
    } else if selected_count == 0 {
        // No sorts selected - keep current state for now
        // TODO: Could deactivate all if we want that behavior
        debug!("No sorts selected - keeping current activation state");
    } else {
        // Multiple sorts selected - keep current activation state for now
        // TODO: Could implement "first selected becomes active" logic here
        debug!("Multiple sorts selected ({}) - keeping current activation state", selected_count);
    }
}

/// Despawn missing buffer sort entities
pub fn despawn_missing_buffer_sort_entities(
    mut commands: Commands,
    text_editor_state: Res<TextEditorState>,
    mut buffer_entities: ResMut<BufferSortEntities>,
    sort_query: Query<Entity, With<BufferSortIndex>>,
) {
    // Remove entities for buffer indices that no longer exist
    let mut to_remove = Vec::new();
    
    for (&buffer_index, &entity) in buffer_entities.entities.iter() {
        if buffer_index >= text_editor_state.buffer.len() {
            // Buffer index no longer exists, despawn entity
            if let Ok(_) = sort_query.get(entity) {
                commands.entity(entity).despawn();
                debug!("Despawned sort entity for deleted buffer index {}", buffer_index);
            }
            to_remove.push(buffer_index);
        }
    }
    
    // Remove from tracking
    for index in to_remove {
        buffer_entities.entities.remove(&index);
    }
}
