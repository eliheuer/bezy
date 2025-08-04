//! Sort entity management for text editor sorts

use crate::core::state::text_editor::TextEditorState;
use crate::core::state::AppState;
use crate::core::state::SortLayoutMode;
use crate::editing::selection::components::Selected;
use crate::editing::sort::{ActiveSort, InactiveSort, Sort};
use bevy::prelude::*;
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
pub fn initialize_text_editor_sorts(mut commands: Commands) {
    commands.init_resource::<BufferSortEntities>();
    info!("Initialized text editor sorts system");
}

/// Manage sort activation based on mouse clicks
#[allow(clippy::too_many_arguments)]
pub fn manage_sort_activation(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<
        (&Camera, &GlobalTransform),
        With<crate::rendering::cameras::DesignCamera>,
    >,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    sort_query: Query<(Entity, &Transform, &Sort, Option<&BufferSortIndex>)>,
    _active_sort_query: Query<Entity, With<crate::editing::sort::ActiveSort>>,
    _text_editor_state: ResMut<TextEditorState>,
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
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    info!("[manage_sort_activation] Left mouse button pressed at screen pos: {:?}", window.cursor_position());

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_position) =
        camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        debug!("[manage_sort_activation] Failed to convert cursor to world position");
        return;
    };

    // Check if click is on any sort
    let click_threshold = 200.0; // Increased click tolerance in design units

    debug!(
        "[manage_sort_activation] Mouse click at world position ({:.1}, {:.1})",
        world_position.x, world_position.y
    );
    debug!(
        "[manage_sort_activation] Checking {} sorts for handle clicks",
        sort_query.iter().count()
    );

    for (entity, transform, sort, _buffer_index) in sort_query.iter() {
        let sort_position = transform.translation.truncate();
        let distance = sort_position.distance(world_position);

        debug!(
            "Sort '{}' at ({:.1}, {:.1}), distance: {:.1}",
            sort.glyph_name, sort_position.x, sort_position.y, distance
        );

        if distance <= click_threshold {
            // TODO: Get modifier key state to support multi-select
            // For now, implement single-select behavior (clear others, select this one)

            // Clear selection from all other sorts
            let all_sorts =
                sort_query.iter().map(|(e, _, _, _)| e).collect::<Vec<_>>();
            for other_entity in all_sorts {
                if other_entity != entity {
                    commands.entity(other_entity).remove::<crate::editing::selection::components::Selected>();
                }
            }

            // Add this sort to the selection
            commands
                .entity(entity)
                .insert(crate::editing::selection::components::Selected);
            info!(
                "Selected sort '{}' at position ({:.1}, {:.1})",
                sort.glyph_name, sort_position.x, sort_position.y
            );

            // Note: Activation will be handled separately based on selection state
            break;
        }
    }
}

/// Spawn missing sort entities for sorts in the text editor buffer
pub fn spawn_missing_sort_entities(
    mut commands: Commands,
    text_editor_state: ResMut<TextEditorState>,
    mut buffer_entities: ResMut<BufferSortEntities>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    _existing_active_sorts: Query<
        Entity,
        With<crate::editing::sort::ActiveSort>,
    >,
) {
    // Debug: Log buffer state
    if !text_editor_state.buffer.is_empty() {
        info!(
            "spawn_missing_sort_entities: Processing {} buffer entries",
            text_editor_state.buffer.len()
        );
        for i in 0..text_editor_state.buffer.len() {
            if let Some(sort) = text_editor_state.buffer.get(i) {
                info!(
                    "  Buffer[{}]: glyph='{}', is_buffer_root={}, is_active={}",
                    i,
                    sort.kind.glyph_name(),
                    sort.is_buffer_root,
                    sort.is_active
                );
            }
        }
    } else {
        debug!(
            "spawn_missing_sort_entities: Buffer is empty, nothing to spawn"
        );
    }

    // Iterate through all sorts in the buffer
    for i in 0..text_editor_state.buffer.len() {
        // Skip if we already have an entity for this buffer index
        if buffer_entities.entities.contains_key(&i) {
            debug!("spawn_missing_sort_entities: Entity already exists for buffer index {}", i);
            continue;
        }

        if let Some(sort_entry) = text_editor_state.buffer.get(i) {
            // Skip spawning entities for line breaks - they are invisible
            if sort_entry.kind.is_line_break() {
                debug!(
                    "Skipping entity spawn for line break at buffer index {}",
                    i
                );
                continue;
            }

            // Get font metrics from either FontIR or AppState
            let font_metrics = if let Some(fontir_state) =
                fontir_app_state.as_ref()
            {
                let fontir_metrics = fontir_state.get_font_metrics();
                crate::core::state::FontMetrics {
                    units_per_em: fontir_metrics.units_per_em as f64,
                    ascender: fontir_metrics.ascender.map(|a| a as f64),
                    descender: fontir_metrics.descender.map(|d| d as f64),
                    line_height: fontir_metrics.line_gap.unwrap_or(0.0) as f64,
                    x_height: None,
                    cap_height: None,
                    italic_angle: None,
                }
            } else if let Some(state) = app_state.as_ref() {
                state.workspace.info.metrics.clone()
            } else {
                // Fallback metrics
                crate::core::state::FontMetrics {
                    units_per_em: 1024.0,
                    ascender: Some(832.0),
                    descender: Some(-256.0),
                    line_height: 0.0,
                    x_height: None,
                    cap_height: None,
                    italic_angle: None,
                }
            };

            // Get the visual position for this sort using correct font metrics
            let position = match sort_entry.layout_mode {
                crate::core::state::SortLayoutMode::LTRText
                | crate::core::state::SortLayoutMode::RTLText => {
                    if sort_entry.is_buffer_root {
                        // Text roots use their exact stored position
                        Some(sort_entry.root_position)
                    } else {
                        // Non-root text sorts flow from their text root using actual font metrics
                        text_editor_state.get_text_sort_flow_position(
                            i,
                            &font_metrics,
                            0.0,
                        )
                    }
                }
                crate::core::state::SortLayoutMode::Freeform => {
                    Some(sort_entry.root_position)
                }
            };

            if let Some(position) = position {
                // Create Sort component
                let sort = Sort {
                    glyph_name: sort_entry.kind.glyph_name().to_string(),
                    layout_mode: sort_entry.layout_mode.clone(),
                };

                // Spawn entity with appropriate activation state
                let mut entity_commands = commands.spawn((
                    sort,
                    Transform::from_translation(position.extend(0.0)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    BufferSortIndex(i),
                    crate::editing::selection::components::Selectable, // Make sorts selectable
                    Name::new(format!("BufferSort[{i}]")),
                ));

                // Check if this sort should be active based on the text editor state
                // Only make sorts active if they are explicitly marked as active
                // Buffer roots are not automatically active anymore - only one sort should be active at a time
                let should_be_active = sort_entry.is_active;
                if should_be_active {
                    entity_commands.insert(ActiveSort);
                    info!("🟢 Spawned ACTIVE sort entity for buffer index {} at position ({:.1}, {:.1}) - glyph '{}' (is_active: {}, is_buffer_root: {})", 
                           i, position.x, position.y, sort_entry.kind.glyph_name(), sort_entry.is_active, sort_entry.is_buffer_root);
                } else {
                    entity_commands.insert(InactiveSort);
                    info!("🔴 Spawned INACTIVE sort entity for buffer index {} at position ({:.1}, {:.1}) - glyph '{}' (is_active: {}, is_buffer_root: {})", 
                           i, position.x, position.y, sort_entry.kind.glyph_name(), sort_entry.is_active, sort_entry.is_buffer_root);
                }

                let entity = entity_commands.id();

                // Track the entity
                buffer_entities.entities.insert(i, entity);

                // Debug logging
                info!("✅ Spawned buffer sort entity {} for glyph '{}' at position ({:.1}, {:.1})", 
                    i, sort_entry.kind.glyph_name(), position.x, position.y);
            } else {
                warn!("Failed to get position for buffer sort index {}", i);
            }
        }
    }
}

/// Update positions of existing buffer sort entities to match text flow
pub fn update_buffer_sort_positions(
    text_editor_state: Res<TextEditorState>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    buffer_entities: Res<BufferSortEntities>,
    mut sort_query: Query<&mut Transform, With<BufferSortIndex>>,
) {
    // CRITICAL PERFORMANCE FIX: Early return if TextEditorState hasn't changed
    // Prevents O(N²) position calculations every frame
    if !text_editor_state.is_changed() {
        return;
    }

    debug!("Buffer position update triggered - TextEditorState changed");

    // Get font metrics from either AppState or FontIR
    let font_metrics = if let Some(fontir_state) = fontir_app_state.as_ref() {
        let fontir_metrics = fontir_state.get_font_metrics();
        crate::core::state::FontMetrics {
            units_per_em: fontir_metrics.units_per_em as f64,
            ascender: fontir_metrics.ascender.map(|a| a as f64),
            descender: fontir_metrics.descender.map(|d| d as f64),
            line_height: fontir_metrics.line_gap.unwrap_or(0.0) as f64,
            x_height: None,
            cap_height: None,
            italic_angle: None,
        }
    } else if let Some(app_state) = app_state.as_ref() {
        app_state.workspace.info.metrics.clone()
    } else {
        warn!("Buffer sort position updates skipped - no font data available");
        return;
    };

    // Update Transform positions for all existing buffer sorts
    for (&buffer_index, &entity) in buffer_entities.entities.iter() {
        if let Ok(mut transform) = sort_query.get_mut(entity) {
            // Calculate correct position using the font metrics from app state
            if let Some(sort) = text_editor_state.buffer.get(buffer_index) {
                // Skip line breaks - they shouldn't have entities
                if sort.kind.is_line_break() {
                    warn!(
                        "Unexpected entity for line break at buffer index {}",
                        buffer_index
                    );
                    continue;
                }
                let new_position = match sort.layout_mode {
                    crate::core::state::SortLayoutMode::LTRText
                    | crate::core::state::SortLayoutMode::RTLText => {
                        if sort.is_buffer_root {
                            // Text roots use their exact stored position
                            Some(sort.root_position)
                        } else {
                            // Non-root text sorts flow from their text root using actual font metrics
                            text_editor_state.get_text_sort_flow_position(
                                buffer_index,
                                &font_metrics,
                                0.0,
                            )
                        }
                    }
                    crate::core::state::SortLayoutMode::Freeform => {
                        Some(sort.root_position)
                    }
                };

                if let Some(new_pos) = new_position {
                    let new_pos_3d = new_pos.extend(transform.translation.z);
                    transform.translation = new_pos_3d;
                    info!(
                        "Sort {} positioned at ({:.1}, {:.1})",
                        buffer_index, new_pos.x, new_pos.y
                    );
                }
            }
        }
    }
}

/// System to auto-activate sorts when exactly one is selected
pub fn auto_activate_selected_sorts(
    mut commands: Commands,
    mut text_editor_state: ResMut<TextEditorState>,
    _buffer_entities: Res<BufferSortEntities>,
    selected_sorts: Query<
        Entity,
        (
            With<Sort>,
            With<crate::editing::selection::components::Selected>,
        ),
    >,
    active_sorts: Query<Entity, With<ActiveSort>>,
    buffer_index_query: Query<&BufferSortIndex>,
) {
    let selected_count = selected_sorts.iter().count();
    let active_count = active_sorts.iter().count();

    // Debug: Log when this system runs and what it finds
    if selected_count > 0 || active_count > 0 {
        info!(
            "auto_activate_selected_sorts: selected={}, active={}",
            selected_count, active_count
        );
    }

    if selected_count == 1 {
        // Exactly one sort is selected - activate it
        if let Ok(selected_sort) = selected_sorts.single() {
            // Find the buffer index of the selected sort
            let selected_buffer_index = if let Ok(buffer_index) =
                buffer_index_query.get(selected_sort)
            {
                Some(buffer_index.0)
            } else {
                None
            };

            // Deactivate all currently active sorts (including buffer roots when another sort is selected)
            for active_entity in active_sorts.iter() {
                // Always deactivate active sorts when a different sort is explicitly selected
                // This allows buffer roots to become inactive when user clicks other handles
                if active_entity != selected_sort {
                    commands.entity(active_entity).remove::<ActiveSort>();
                    commands.entity(active_entity).insert(InactiveSort);

                    // Update text editor state to deactivate the sort
                    if let Ok(buffer_index) =
                        buffer_index_query.get(active_entity)
                    {
                        if let Some(sort) =
                            text_editor_state.buffer.get_mut(buffer_index.0)
                        {
                            let is_root = sort.is_buffer_root;
                            info!("🔻 Deactivating buffer sort {} - glyph '{}' (was_root: {})", buffer_index.0, sort.kind.glyph_name(), sort.is_buffer_root);
                            sort.is_active = false;

                            // CRITICAL DEBUG: Track root sort deactivation specifically
                            if is_root {
                                warn!("ROOT SORT DEACTIVATION: Entity {:?} (buffer index {}) is being deactivated and should show filled rendering!", active_entity, buffer_index.0);
                            }
                        }
                    }
                }
            }

            // Activate the selected sort
            commands.entity(selected_sort).remove::<InactiveSort>();
            commands.entity(selected_sort).insert(ActiveSort);

            // Update text editor state to activate the sort
            if let Some(buffer_index) = selected_buffer_index {
                info!("🔼 Activating buffer sort {} via auto_activate_selected_sorts", buffer_index);
                text_editor_state.activate_sort(buffer_index);
            }

            info!("Auto-activated selected sort {:?}", selected_sort);
        }
    } else if selected_count == 0 {
        // No sorts selected - keep current state for now
        // TODO: Could deactivate all if we want that behavior
        debug!("No sorts selected - keeping current activation state");
    } else {
        // Multiple sorts selected - keep current activation state for now
        // TODO: Could implement "first selected becomes active" logic here
        debug!(
            "Multiple sorts selected ({}) - keeping current activation state",
            selected_count
        );
    }
}

/// Despawn missing buffer sort entities
#[allow(clippy::too_many_arguments)]
pub fn despawn_missing_buffer_sort_entities(
    mut commands: Commands,
    text_editor_state: Res<TextEditorState>,
    mut buffer_entities: ResMut<BufferSortEntities>,
    mut unified_entities: ResMut<
        crate::rendering::unified_glyph_editing::UnifiedGlyphEntities,
    >,
    mut metrics_entities: ResMut<
        crate::rendering::metrics::MetricsLineEntities,
    >,
    sort_query: Query<Entity, With<BufferSortIndex>>,
    unified_element_query: Query<(
        Entity,
        &crate::rendering::unified_glyph_editing::UnifiedGlyphElement,
    )>,
    point_query: Query<
        Entity,
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
    sort_point_query: Query<&crate::systems::sort_manager::SortPointEntity>,
    sort_name_text_query: Query<(
        Entity,
        &crate::rendering::sort_renderer::SortGlyphNameText,
    )>,
    sort_unicode_text_query: Query<(
        Entity,
        &crate::rendering::sort_renderer::SortUnicodeText,
    )>,
) {
    // Always log to see if system is running
    info!("🔍 despawn_missing_buffer_sort_entities: SYSTEM CALLED - Buffer length: {}, Tracked entities: {}", 
          text_editor_state.buffer.len(), buffer_entities.entities.len());

    // Debug: Log current state
    if !buffer_entities.entities.is_empty() {
        info!("despawn_missing_buffer_sort_entities: Checking {} tracked entities against buffer length {}", 
              buffer_entities.entities.len(), text_editor_state.buffer.len());

        // Log all tracked entities
        for (&idx, &entity) in buffer_entities.entities.iter() {
            info!("  Tracked entity: buffer[{}] -> entity {:?}", idx, entity);
        }
    }

    // Remove entities for buffer indices that no longer exist
    let mut to_remove = Vec::new();

    for (&buffer_index, &entity) in buffer_entities.entities.iter() {
        // Check if this buffer index still exists in the actual buffer
        if buffer_index >= text_editor_state.buffer.len() {
            // Buffer index no longer exists, despawn entity
            if sort_query.get(entity).is_ok() {
                info!(
                    "🗑️ Despawning sort entity for deleted buffer index {} (buffer len: {})",
                    buffer_index, text_editor_state.buffer.len()
                );

                // First, despawn all unified glyph elements associated with this sort
                let mut unified_count = 0;

                // Despawn all unified elements (points, outlines, handles)
                if let Some(element_entities) = unified_entities.elements.get(&entity) {
                    for &element_entity in element_entities.iter() {
                        commands.entity(element_entity).despawn();
                        unified_count += 1;
                    }
                }

                // Also despawn any loose unified elements that might not be tracked
                for (unified_entity, unified_element) in unified_element_query.iter() {
                    if unified_element.sort_entity == entity {
                        commands.entity(unified_entity).despawn();
                        unified_count += 1;
                    }
                }

                // Despawn all point entities associated with this sort
                let mut point_count = 0;
                for point_entity in point_query.iter() {
                    if let Ok(sort_point) = sort_point_query.get(point_entity) {
                        if sort_point.sort_entity == entity {
                            commands.entity(point_entity).despawn();
                            point_count += 1;
                        }
                    }
                }

                // Despawn all metrics line entities associated with this sort
                let mut metrics_count = 0;
                if let Some(line_entities) = metrics_entities.lines.get(&entity)
                {
                    for &line_entity in line_entities.iter() {
                        commands.entity(line_entity).despawn();
                        metrics_count += 1;
                    }
                }

                // Despawn all sort label text entities associated with this sort
                let mut label_count = 0;

                // Despawn glyph name text entities
                for (text_entity, name_text) in sort_name_text_query.iter() {
                    if name_text.sort_entity == entity {
                        commands.entity(text_entity).despawn();
                        label_count += 1;
                    }
                }

                // Despawn unicode text entities
                for (text_entity, unicode_text) in
                    sort_unicode_text_query.iter()
                {
                    if unicode_text.sort_entity == entity {
                        commands.entity(text_entity).despawn();
                        label_count += 1;
                    }
                }

                info!("🗑️ Despawned {} unified elements, {} point entities, {} metrics entities, and {} label entities for sort {:?}", 
                      unified_count, point_count, metrics_count, label_count, entity);

                // Remove from unified tracking
                unified_entities.elements.remove(&entity);

                // Remove from metrics tracking
                metrics_entities.lines.remove(&entity);

                // Finally, despawn the sort entity itself
                commands.entity(entity).despawn();
                to_remove.push(buffer_index);
            } else {
                warn!("Entity {:?} for buffer index {} already despawned or invalid", entity, buffer_index);
                to_remove.push(buffer_index);
            }
        }
    }

    // Store count before consuming the vector
    let despawn_count = to_remove.len();

    // Remove from tracking
    for index in to_remove {
        buffer_entities.entities.remove(&index);
        info!("🗑️ Removed buffer index {} from entity tracking", index);
    }

    if despawn_count > 0 {
        info!("🗑️ Despawned {} entities total", despawn_count);
    }
}
