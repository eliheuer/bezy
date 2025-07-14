use super::components::*;
use super::coordinate_system::SelectionCoordinateSystem;
use super::DragPointState;
use super::DragSelectionState;
use crate::core::input::{helpers, InputEvent, InputState};
use crate::core::pointer::PointerInfo;
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use crate::core::state::AppState;
use crate::core::state::FontMetrics;
use crate::core::state::TextEditorState;
use crate::editing::edit_type::EditType;
use crate::editing::selection::nudge::{EditEvent, NudgeState};
#[allow(unused_imports)]
use crate::geometry::design_space::DPoint;
#[allow(unused_imports)]
use crate::geometry::point::{EditPoint, EntityId, EntityKind};
use crate::rendering::cameras::DesignCamera;
use bevy::ecs::system::ParamSet;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::window::PrimaryWindow;

/// Event to signal that app state has changed
#[derive(Event, Debug, Clone)]
pub struct AppStateChanged;

/// A resource to hold the world position of a handled click.
/// This prevents multiple systems from reacting to the same click event.
#[derive(Resource)]
pub struct ClickWorldPosition;

// Constants for selection
#[allow(dead_code)]
const SELECTION_MARGIN: f32 = 16.0; // Distance in pixels for selection hit testing

// Legacy handle_mouse_input system removed - replaced by handle_selection_input_events

/// System to handle selection shortcuts (Ctrl+A for select all, etc.)
pub fn handle_selection_shortcuts(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_query: Query<Entity, With<Selectable>>,
    mut selection_state: ResMut<SelectionState>,
    mut event_writer: EventWriter<EditEvent>,
    select_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
    >,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
    text_editor_state: Option<Res<crate::core::state::TextEditorState>>,
) {
    // Skip processing shortcuts if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Only process shortcuts when in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    // Only allow selection shortcuts when there's an active sort in text editor
    if let Some(text_editor_state) = text_editor_state.as_ref() {
        if text_editor_state.get_active_sort().is_none() {
            return;
        }
    } else {
        return;
    }

    // Handle Escape key to clear selection
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();
        debug!("Cleared selection");
    }

    // Handle Ctrl+A (select all)
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight);

    if ctrl_pressed && keyboard_input.just_pressed(KeyCode::KeyA) {
        debug!("Select all shortcut pressed");

        // Clear current selection
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();

        // Select all selectable entities
        for entity in &selectable_query {
            selection_state.selected.insert(entity);
            commands.entity(entity).insert(Selected);
        }

        debug!("Selected all {} entities", selection_state.selected.len());

        // Send edit event
        event_writer.write(EditEvent {
            edit_type: EditType::Normal,
        });
    }
}

/// System to update which entities are being hovered over by the mouse
#[allow(dead_code)]
pub fn update_hover_state(
    mut _commands: Commands,
    _windows: Query<&Window, With<PrimaryWindow>>,
    _camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    _selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
    _hovered_query: Query<Entity, With<Hovered>>,
) {
    // Hover functionality is disabled per user request
}

/// System to render the selection rectangle during drag operations
pub fn render_selection_rect(
    mut gizmos: Gizmos,
    selection_rect_query: Query<&SelectionRect>,
    select_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
    >,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
) {
    let rect_count = selection_rect_query.iter().len();
    debug!("render_selection_rect called: {} rects", rect_count);
    if rect_count > 0 {
        for rect in &selection_rect_query {
            info!("SelectionRect: start={:?}, end={:?}", rect.start, rect.end);
        }
    }
    // Skip rendering the selection rectangle if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Only render the selection rectangle in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    for rect in &selection_rect_query {
        let rect_bounds = Rect::from_corners(rect.start, rect.end);

        // Define the orange color to match selected buttons (similar to PRESSED_BUTTON in theme.rs)
        let orange_color = Color::srgb(1.0, 0.6, 0.1);

        // Get the corner points
        let min_x = rect_bounds.min.x;
        let min_y = rect_bounds.min.y;
        let max_x = rect_bounds.max.x;
        let max_y = rect_bounds.max.y;

        // Define dash properties
        let _dash_length = 10.0;
        let _gap_length = 5.0;

        // Draw dashed lines for each side of the rectangle
        draw_dashed_line(
            &mut gizmos,
            Vec2::new(min_x, min_y),
            Vec2::new(max_x, min_y),
            orange_color,
            16.0,
            8.0,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(max_x, min_y),
            Vec2::new(max_x, max_y),
            orange_color,
            16.0,
            8.0,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(max_x, max_y),
            Vec2::new(min_x, max_y),
            orange_color,
            16.0,
            8.0,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(min_x, max_y),
            Vec2::new(min_x, min_y),
            orange_color,
            16.0,
            8.0,
        );
    }
}

/// System to render the selection marquee rectangle using Gizmos
pub fn render_selection_marquee(
    mut gizmos: Gizmos,
    selection_rects: Query<&SelectionRect>,
) {
    let count = selection_rects.iter().count();
    info!(
        "[render_selection_marquee] Called, found {} SelectionRect entities",
        count
    );
    for rect in selection_rects.iter() {
        info!(
            "[render_selection_marquee] Drawing marquee: start={:?}, end={:?}",
            rect.start, rect.end
        );
        let start = rect.start;
        let end = rect.end;
        let color = Color::srgb(1.0, 0.5, 0.0); // Orange
        let _dash_length = 16.0;
        let _gap_length = 8.0;
        // Four corners
        let p1 = Vec2::new(start.x, start.y);
        let p2 = Vec2::new(end.x, start.y);
        let p3 = Vec2::new(end.x, end.y);
        let p4 = Vec2::new(start.x, end.y);
        // Draw dashed lines for each edge
        draw_dashed_line(&mut gizmos, p1, p2, color, 16.0, 8.0);
        draw_dashed_line(&mut gizmos, p2, p3, color, 16.0, 8.0);
        draw_dashed_line(&mut gizmos, p3, p4, color, 16.0, 8.0);
        draw_dashed_line(&mut gizmos, p4, p1, color, 16.0, 8.0);
    }
}

/// Helper to draw a dashed line between two points
fn draw_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    color: Color,
    dash_length: f32,
    gap_length: f32,
) {
    let total_length = (end - start).length();
    let direction = (end - start).normalize_or_zero();
    let mut current_length = 0.0;
    let mut draw = true;
    let mut p = start;
    while current_length < total_length {
        let segment_length = if draw { dash_length } else { gap_length };
        let next_length = (current_length + segment_length).min(total_length);
        let next_p = start + direction * next_length;
        if draw {
            gizmos.line_2d(p, next_p, color);
        }
        p = next_p;
        current_length = next_length;
        draw = !draw;
    }
}

/// System to render selected entities with visual feedback
pub fn render_selected_entities(
    mut gizmos: Gizmos,
    selected_query: Query<
        (
            &GlobalTransform,
            &crate::editing::selection::components::PointType,
        ),
        With<Selected>,
    >,
    drag_point_state: Res<DragPointState>,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
    // Check nudge state to avoid double rendering during nudging
    nudge_state: Res<crate::editing::selection::nudge::NudgeState>,
) {
    // CRITICAL: ALWAYS skip this system during nudging - no exceptions
    // Force all rendering through the unified live system to prevent flashing
    if nudge_state.is_nudging {
        return; // No debug spam, just skip
    }

    let selected_count = selected_query.iter().count();
    if selected_count > 0 {
        info!("Selection: Rendering {} selected entities", selected_count);
    }

    // Skip rendering if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Determine if we're currently dragging points (for enhanced visibility)
    let is_dragging = drag_point_state.is_dragging;

    // Increase the visual size slightly during dragging for better visibility
    let size_multiplier = if is_dragging { 1.25 } else { 1.0 };

    // Select the color - make it brighter during dragging
    let selection_color = if is_dragging {
        // Brighter orange during dragging
        Color::srgb(1.0, 0.7, 0.2)
    } else {
        crate::ui::theme::SELECTED_POINT_COLOR
    };

    for (transform, point_type) in &selected_query {
        let position = transform.translation().truncate();
        // Use a much higher Z offset to ensure selected points always render on top of normal glyph points
        let position_3d = Vec3::new(
            position.x,
            position.y,
            transform.translation().z + 100.0,
        );
        let position_2d = position_3d.truncate();

        // Different rendering based on point type
        if point_type.is_on_curve && crate::ui::theme::USE_SQUARE_FOR_ON_CURVE {
            // Draw a filled square for on-curve points to completely cover the underlying glyph point
            let half_size = crate::ui::theme::SELECTION_POINT_RADIUS
                / crate::ui::theme::ON_CURVE_SQUARE_ADJUSTMENT
                * size_multiplier;

            // Draw a filled rectangle to completely cover the underlying green square
            gizmos.rect_2d(
                position_2d,
                Vec2::new(half_size * 2.0, half_size * 2.0),
                selection_color,
            );

            // Draw a smaller inner circle for visual distinction
            gizmos.circle_2d(
                position_2d,
                half_size * crate::ui::theme::ON_CURVE_INNER_CIRCLE_RATIO,
                selection_color,
            );
        } else {
            // Draw a circle for off-curve points
            gizmos.circle_2d(
                position_2d,
                crate::ui::theme::SELECTION_POINT_RADIUS
                    * crate::ui::theme::SELECTED_CIRCLE_RADIUS_MULTIPLIER
                    * size_multiplier,
                selection_color,
            );

            // For off-curve points, also draw a smaller inner circle
            if !point_type.is_on_curve {
                gizmos.circle_2d(
                    position_2d,
                    crate::ui::theme::SELECTION_POINT_RADIUS
                        * crate::ui::theme::OFF_CURVE_INNER_CIRCLE_RATIO
                        * size_multiplier,
                    selection_color,
                );
            }
        }

        // Always draw the crosshair for all selected points
        let line_size = if point_type.is_on_curve
            && crate::ui::theme::USE_SQUARE_FOR_ON_CURVE
        {
            // For on-curve square points, use the half_size of the square
            crate::ui::theme::SELECTION_POINT_RADIUS
                / crate::ui::theme::ON_CURVE_SQUARE_ADJUSTMENT
        } else {
            // For off-curve circle points, use the radius
            crate::ui::theme::SELECTION_POINT_RADIUS
                * crate::ui::theme::SELECTED_CIRCLE_RADIUS_MULTIPLIER
        };

        // Apply size multiplier to crosshairs as well
        let line_size = line_size * size_multiplier;

        // Draw crosshair
        gizmos.line_2d(
            Vec2::new(position_2d.x - line_size, position_2d.y),
            Vec2::new(position_2d.x + line_size, position_2d.y),
            selection_color,
        );

        gizmos.line_2d(
            Vec2::new(position_2d.x, position_2d.y - line_size),
            Vec2::new(position_2d.x, position_2d.y + line_size),
            selection_color,
        );

        // If dragging, draw a second set of lines to make them appear thicker
        if is_dragging {
            // Offset slightly to create thicker appearance
            let offset = 0.5;

            gizmos.line_2d(
                Vec2::new(position_2d.x - line_size, position_2d.y + offset),
                Vec2::new(position_2d.x + line_size, position_2d.y + offset),
                selection_color,
            );

            gizmos.line_2d(
                Vec2::new(position_2d.x - line_size, position_2d.y - offset),
                Vec2::new(position_2d.x + line_size, position_2d.y - offset),
                selection_color,
            );

            gizmos.line_2d(
                Vec2::new(position_2d.x + offset, position_2d.y - line_size),
                Vec2::new(position_2d.x + offset, position_2d.y + line_size),
                selection_color,
            );

            gizmos.line_2d(
                Vec2::new(position_2d.x - offset, position_2d.y - line_size),
                Vec2::new(position_2d.x - offset, position_2d.y + line_size),
                selection_color,
            );
        }
    }
}

/// System to render hovered entities (disabled for now)
#[allow(dead_code)]
pub fn render_hovered_entities(
    mut _gizmos: Gizmos,
    _hovered_query: Query<
        (
            &GlobalTransform,
            &crate::editing::selection::components::PointType,
        ),
        With<Hovered>,
    >,
) {
    // Hover functionality is disabled per user request
}

/// System to update the actual glyph data when a point is moved
pub fn update_glyph_data_from_selection(
    query: Query<
        (
            &Transform,
            &GlyphPointReference,
            Option<&crate::systems::sort_manager::SortPointEntity>,
        ),
        (With<Selected>, Changed<Transform>),
    >,
    sort_query: Query<(&crate::editing::sort::Sort, &Transform)>,
    mut app_state: ResMut<AppState>,
    // Track if we're in a nudging operation
    _nudge_state: Res<crate::editing::selection::nudge::NudgeState>,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
) {
    // Skip processing if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // REMOVED: Skip during nudging - we want sync to work normally during nudging

    // Early return if no points were moved
    if query.is_empty() {
        return;
    }

    info!(
        "[update_glyph_data_from_selection] Processing {} moved points",
        query.iter().count()
    );

    let app_state = app_state.bypass_change_detection();
    let mut any_updates = false;

    for (transform, point_ref, sort_point_entity_opt) in query.iter() {
        // Default to world position if we can't get sort position
        let (relative_x, relative_y) =
            if let Some(sort_point_entity) = sort_point_entity_opt {
                if let Ok((_sort, sort_transform)) =
                    sort_query.get(sort_point_entity.sort_entity)
                {
                    let world_pos = transform.translation.truncate();
                    let sort_pos = sort_transform.translation.truncate();
                    let rel = world_pos - sort_pos;
                    (rel.x as f64, rel.y as f64)
                } else {
                    (
                        transform.translation.x as f64,
                        transform.translation.y as f64,
                    )
                }
            } else {
                (
                    transform.translation.x as f64,
                    transform.translation.y as f64,
                )
            };

        let updated = app_state.set_point_position(
            &point_ref.glyph_name,
            point_ref.contour_index,
            point_ref.point_index,
            relative_x,
            relative_y,
        );

        info!(
            "[update_glyph_data_from_selection] glyph='{}' contour={} point={} rel=({:.1}, {:.1}) updated={}",
            point_ref.glyph_name,
            point_ref.contour_index,
            point_ref.point_index,
            relative_x,
            relative_y,
            updated
        );

        if updated {
            any_updates = true;
            debug!(
                "Updated UFO glyph data for point {} in contour {} of glyph {}",
                point_ref.point_index,
                point_ref.contour_index,
                point_ref.glyph_name
            );
        } else {
            warn!(
                "Failed to update UFO glyph data for point {} in contour {} of glyph {} - invalid indices",
                point_ref.point_index, point_ref.contour_index, point_ref.glyph_name
            );
        }
    }

    // Log the results
    if any_updates {
        info!("[update_glyph_data_from_selection] Successfully updated {} outline points", query.iter().count());
    } else {
        info!("[update_glyph_data_from_selection] No outline updates needed");
    }
}

/// System to spawn point entities for the active sort using ECS as source of truth
pub fn spawn_active_sort_points(
    mut commands: Commands,
    active_sort_state: Res<crate::editing::sort::ActiveSortState>,
    sort_query: Query<(Entity, &crate::editing::sort::Sort, &Transform)>,
    point_entities: Query<
        Entity,
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
    app_state: Res<AppState>,
    _selection_state: ResMut<crate::editing::selection::SelectionState>,
) {
    // Only spawn points if there's an active sort
    if let Some(active_sort_entity) = active_sort_state.active_sort_entity {
        if let Ok((sort_entity, sort, transform)) =
            sort_query.get(active_sort_entity)
        {
            // Check if points already exist for this sort
            let existing_points = point_entities.iter().any(|_entity| {
                // Check if points already exist for this sort
                true // Simplified check
            });

            if !existing_points {
                let position = transform.translation.truncate();
                info!("[spawn_active_sort_points] Spawning points for active sort: '{}' at position {:?}", 
                      sort.glyph_name, position);

                // Get glyph data for the active sort
                if let Some(glyph_data) =
                    app_state.workspace.font.get_glyph(&sort.glyph_name)
                {
                    if let Some(outline) = &glyph_data.outline {
                        let mut point_count = 0;

                        for (contour_index, contour) in
                            outline.contours.iter().enumerate()
                        {
                            for (point_index, point) in
                                contour.points.iter().enumerate()
                            {
                                // Calculate world position: sort position + point offset
                                let point_world_pos = position
                                    + Vec2::new(point.x as f32, point.y as f32);
                                point_count += 1;

                                // Debug: Print first few point positions
                                if point_count <= 5 {
                                    info!("[spawn_active_sort_points] Point {}: local=({:.1}, {:.1}), world=({:.1}, {:.1})", 
                                          point_count, point.x, point.y, point_world_pos.x, point_world_pos.y);
                                }

                                let glyph_point_ref = crate::editing::selection::components::GlyphPointReference {
                                    glyph_name: sort.glyph_name.clone(),
                                    contour_index,
                                    point_index,
                                };

                                let _entity = commands.spawn((
                                    crate::geometry::point::EditPoint {
                                        position: kurbo::Point::new(point.x, point.y),
                                        point_type: point.point_type,
                                    },
                                    glyph_point_ref,
                                    crate::editing::selection::components::PointType {
                                        is_on_curve: matches!(point.point_type, 
                                            crate::core::state::font_data::PointTypeData::Move | 
                                            crate::core::state::font_data::PointTypeData::Line |
                                            crate::core::state::font_data::PointTypeData::Curve),
                                    },
                                    Transform::from_translation(point_world_pos.extend(0.0)),
                                    Visibility::Visible,
                                    InheritedVisibility::default(),
                                    ViewVisibility::default(),
                                    crate::editing::selection::components::Selectable,
                                    crate::systems::sort_manager::SortPointEntity { sort_entity },
                                )).id();
                            }
                        }
                        info!("[spawn_active_sort_points] Successfully spawned {} point entities", point_count);
                    } else {
                        warn!("[spawn_active_sort_points] No outline found for glyph '{}'", sort.glyph_name);
                    }
                } else {
                    warn!("[spawn_active_sort_points] No glyph data found for '{}'", sort.glyph_name);
                }
            } else {
                debug!("[spawn_active_sort_points] Points already exist for active sort, skipping spawn");
            }
        } else {
            warn!("[spawn_active_sort_points] Active sort entity not found in sort query");
        }
    } else {
        debug!(
            "[spawn_active_sort_points] No active sort, skipping point spawn"
        );
    }
}

/// System to despawn point entities when active sort changes
pub fn despawn_inactive_sort_points(
    mut commands: Commands,
    active_sort_state: Res<crate::editing::sort::ActiveSortState>,
    point_entities: Query<(
        Entity,
        &crate::systems::sort_manager::SortPointEntity,
    )>,
    mut selection_state: ResMut<crate::editing::selection::SelectionState>,
) {
    // Despawn points for sorts that are no longer active
    for (entity, sort_point) in point_entities.iter() {
        let is_active = active_sort_state
            .active_sort_entity == Some(sort_point.sort_entity);

        if !is_active {
            // Remove from selection state if selected
            if selection_state.selected.contains(&entity) {
                selection_state.selected.remove(&entity);
                info!("[despawn_inactive_sort_points] Removed despawned entity {:?} from selection", entity);
            }

            commands.entity(entity).despawn();
            debug!("[despawn_inactive_sort_points] Despawned point entity {:?} for inactive sort {:?}", entity, sort_point.sort_entity);
        }
    }
}

/// System to update point positions when sort position changes
pub fn sync_point_positions_to_sort(
    mut param_set: ParamSet<(
        Query<
            (Entity, &crate::editing::sort::Sort, &Transform),
            Changed<crate::editing::sort::Sort>,
        >,
        Query<(
            &mut Transform,
            &crate::systems::sort_manager::SortPointEntity,
            &crate::editing::selection::components::GlyphPointReference,
        )>,
    )>,
    app_state: Res<AppState>,
) {
    // First, collect all the sort positions that have changed
    let mut sort_positions = std::collections::HashMap::new();

    for (sort_entity, sort, sort_transform) in param_set.p0().iter() {
        let position = sort_transform.translation.truncate();
        sort_positions.insert(sort_entity, (sort.glyph_name.clone(), position));
    }

    // Then update all point transforms based on the collected positions
    for (mut point_transform, sort_point, glyph_ref) in
        param_set.p1().iter_mut()
    {
        if let Some((glyph_name, position)) =
            sort_positions.get(&sort_point.sort_entity)
        {
            // Get the original point data from the glyph
            if let Some(glyph_data) =
                app_state.workspace.font.get_glyph(glyph_name)
            {
                if let Some(outline) = &glyph_data.outline {
                    if let Some(contour) =
                        outline.contours.get(glyph_ref.contour_index)
                    {
                        if let Some(point) =
                            contour.points.get(glyph_ref.point_index)
                        {
                            // Calculate new world position: sort position + original point offset
                            let point_world_pos = *position
                                + Vec2::new(point.x as f32, point.y as f32);
                            point_transform.translation =
                                point_world_pos.extend(0.0);

                            debug!("[sync_point_positions_to_sort] Updated point {} in contour {} to position {:?}", 
                                   glyph_ref.point_index, glyph_ref.contour_index, point_world_pos);
                        }
                    }
                }
            }
        }
    }
}

/// System to handle key releases for nudging
pub fn handle_key_releases(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut nudge_state: ResMut<NudgeState>,
) {
    // Reset nudging state if no arrow keys are pressed
    let arrow_keys = [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ];

    let any_arrow_pressed =
        arrow_keys.iter().any(|key| keyboard_input.pressed(*key));

    if !any_arrow_pressed {
        nudge_state.is_nudging = false;
    }
}

/// System to clear selection when app state changes (e.g., when codepoint changes)
pub fn clear_selection_on_app_change(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    mut selection_state: ResMut<SelectionState>,
    mut events: EventReader<AppStateChanged>,
) {
    for _ in events.read() {
        // Clear the selection when app state changes (e.g., when codepoint changes)
        selection_state.selected.clear();

        // Also remove the Selected component from all entities
        for entity in &query {
            commands.entity(entity).remove::<Selected>();
        }

        debug!("Selection cleared due to app state change");
    }
}

/// System to handle advanced point dragging with constraints and snapping
pub fn handle_point_drag(
    pointer_info: Res<PointerInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut drag_point_state: ResMut<DragPointState>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut crate::editing::selection::nudge::PointCoordinates,
            Option<&GlyphPointReference>,
            Option<&crate::systems::sort_manager::SortCrosshair>,
        ),
        With<Selected>,
    >,
    mut app_state: ResMut<AppState>,
    mut event_writer: EventWriter<EditEvent>,
) {
    // Only drag if the resource says we are
    if !drag_point_state.is_dragging {
        return;
    }

    let cursor_pos = pointer_info.design.to_raw();
    drag_point_state.current_position = Some(cursor_pos);

    if let Some(start_pos) = drag_point_state.start_position {
        let total_movement = cursor_pos - start_pos;
        let mut movement = total_movement;

        // Handle constrained movement with Shift key
        if keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            if total_movement.x.abs() > total_movement.y.abs() {
                movement.y = 0.0; // Constrain to horizontal
            } else {
                movement.x = 0.0; // Constrain to vertical
            }
        }

        let mut updated_count = 0;

        for (
            entity,
            mut transform,
            mut coordinates,
            point_ref,
            sort_crosshair,
        ) in &mut query
        {
            if let Some(original_pos) =
                drag_point_state.original_positions.get(&entity)
            {
                let new_pos = *original_pos + movement;

                // Handle sort crosshair drag (no snapping, keep on top)
                if sort_crosshair.is_some() {
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                    transform.translation.z = 25.0; // Keep crosshairs on top
                    coordinates.x = new_pos.x;
                    coordinates.y = new_pos.y;
                }
                // Handle glyph point drag (with snapping)
                else if let Some(point_ref) = point_ref {
                    // Apply grid snapping if enabled
                    let snapped_pos = if SNAP_TO_GRID_ENABLED {
                        let grid_size = SNAP_TO_GRID_VALUE;
                        Vec2::new(
                            (new_pos.x / grid_size).round() * grid_size,
                            (new_pos.y / grid_size).round() * grid_size,
                        )
                    } else {
                        new_pos
                    };

                    transform.translation.x = snapped_pos.x;
                    transform.translation.y = snapped_pos.y;
                    transform.translation.z = 5.0; // Keep glyph points above background
                    coordinates.x = snapped_pos.x;
                    coordinates.y = snapped_pos.y;

                    // Update UFO data for glyph points
                    let updated = app_state.set_point_position(
                        &point_ref.glyph_name,
                        point_ref.contour_index,
                        point_ref.point_index,
                        transform.translation.x as f64, // Convert f32 to f64
                        transform.translation.y as f64, // Convert f32 to f64
                    );
                    if updated {
                        updated_count += 1;
                    }
                }
                // Handle other draggable entities (no snapping, normal Z layer)
                else {
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                    transform.translation.z = 10.0; // Middle layer
                    coordinates.x = new_pos.x;
                    coordinates.y = new_pos.y;
                }
            }
        }

        if updated_count > 0 {
            debug!("Updated {} UFO points during drag", updated_count);

            // Send edit event
            event_writer.write(EditEvent {
                edit_type: EditType::Normal,
            });
        }
    }
}

/// System to clean up the click resource
pub fn cleanup_click_resource(mut commands: Commands) {
    commands.remove_resource::<ClickWorldPosition>();
}

/// System to process selection input events from the new input system
pub fn process_selection_input_events(
    mut commands: Commands,
    mut input_events: EventReader<crate::core::input::InputEvent>,
    input_state: Res<crate::core::input::InputState>,
    mut drag_state: ResMut<DragSelectionState>,
    mut drag_point_state: ResMut<DragPointState>,
    mut event_writer: EventWriter<EditEvent>,
    selectable_query: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    selected_query: Query<(Entity, &Transform), With<Selected>>,
    selection_rect_query: Query<Entity, With<SelectionRect>>,
    mut selection_state: ResMut<SelectionState>,
    active_sort_state: Res<crate::editing::sort::ActiveSortState>,
    sort_point_entities: Query<&crate::systems::sort_manager::SortPointEntity>,
    select_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
    >,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Res<crate::core::state::AppState>,
) {
    debug!("[process_selection_input_events] Called");
    // Only process if in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            debug!("[process_selection_input_events] Not in select mode, returning early");
            return;
        }
    }
    for event in input_events.read() {
        debug!(
            "[process_selection_input_events] Processing event: {:?}",
            event
        );

        // Skip if UI is consuming input
        if crate::core::input::helpers::is_ui_consuming(&input_state) {
            debug!("Selection: Skipping event - UI is consuming input");
            continue;
        }

        // Only handle events that are relevant to selection
        match event {
            crate::core::input::InputEvent::MouseClick {
                button,
                position,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    let world_position = position.to_raw();
                    let handle_tolerance = 50.0;
                    let font_metrics = &app_state.workspace.info.metrics;
                    debug!("[sort-handle-hit] Click at world position: ({:.1}, {:.1})", world_position.x, world_position.y);
                    // Print all handle positions and distances
                    for i in 0..text_editor_state.buffer.len() {
                        if let Some(_sort) = text_editor_state.buffer.get(i) {
                            if let Some(sort_pos) =
                                text_editor_state.get_sort_visual_position(i)
                            {
                                let descender =
                                    font_metrics.descender.unwrap_or(-200.0)
                                        as f32;
                                let handle_pos =
                                    sort_pos + Vec2::new(0.0, descender);
                                let distance =
                                    world_position.distance(handle_pos);
                                debug!("[sort-handle-hit] Sort {}: handle_pos=({:.1}, {:.1}), distance={:.1}, tolerance={:.1}",
                                    i, handle_pos.x, handle_pos.y, distance, handle_tolerance);
                            }
                        }
                    }
                    if let Some(clicked_sort_index) = text_editor_state
                        .find_sort_handle_at_position(
                            world_position,
                            handle_tolerance,
                            Some(font_metrics),
                        )
                    {
                        debug!("[process_selection_input_events] Clicked near sort handle at index {}", clicked_sort_index);
                        let is_ctrl_held = modifiers.ctrl;
                        if is_ctrl_held {
                            // ECS-based selection: activate the clicked sort directly
                            text_editor_state.activate_sort(clicked_sort_index);
                            debug!("[process_selection_input_events] Ctrl: activated sort {}", clicked_sort_index);
                        } else {
                            // ECS-based selection: activate the clicked sort directly
                            text_editor_state.activate_sort(clicked_sort_index);
                            debug!("[process_selection_input_events] Regular click: activated sort {}", clicked_sort_index);
                        }
                        // Early return: don't run the rest of the selection logic for this click
                        return;
                    } else {
                        debug!("[sort-handle-hit] No sort handle hit detected");

                        // Fallback to general selection click handling
                        // Use a dummy entity if no active sort exists
                        let active_sort_entity = active_sort_state
                            .active_sort_entity
                            .unwrap_or(Entity::PLACEHOLDER);
                        handle_selection_click(
                            &mut commands,
                            position,
                            modifiers,
                            &mut drag_state,
                            &mut drag_point_state,
                            &mut event_writer,
                            &selectable_query,
                            &selected_query,
                            &mut selection_state,
                            active_sort_entity,
                            &sort_point_entities,
                        );
                    }
                }
            }
            crate::core::input::InputEvent::MouseDrag {
                button,
                start_position,
                current_position,
                delta,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    debug!("Selection: Processing mouse drag from {:?} to {:?} with modifiers {:?}",
                          start_position, current_position, modifiers);
                    debug!(
                        "Selection: active_sort_entity={:?}",
                        active_sort_state.active_sort_entity
                    );

                    // Always allow drag selection, regardless of active sort state
                    // Use a dummy entity if no active sort exists
                    let active_sort_entity = active_sort_state
                        .active_sort_entity
                        .unwrap_or(Entity::PLACEHOLDER);
                    debug!("Selection: Calling handle_selection_drag...");
                    handle_selection_drag(
                        &mut commands,
                        start_position,
                        current_position,
                        delta,
                        modifiers,
                        &mut drag_state,
                        &mut drag_point_state,
                        &mut event_writer,
                        &selectable_query,
                        &mut selection_state,
                        active_sort_entity,
                        &sort_point_entities,
                        &selection_rect_query,
                    );
                    debug!("Selection: handle_selection_drag completed");
                }
            }
            crate::core::input::InputEvent::MouseRelease {
                button,
                position,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    debug!("Selection: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // Always handle mouse release for selection, regardless of active sort state
                    handle_selection_release(
                        &mut commands,
                        position,
                        modifiers,
                        &mut drag_state,
                        &mut drag_point_state,
                        &mut event_writer,
                        &mut selection_state,
                        &selection_rect_query,
                    );
                }
            }
            crate::core::input::InputEvent::KeyPress { key, modifiers } => {
                if matches!(
                    key,
                    bevy::input::keyboard::KeyCode::KeyA
                        | bevy::input::keyboard::KeyCode::Escape
                ) {
                    debug!("Selection: Processing key press {:?} with modifiers {:?}", key, modifiers);
                    // Always handle key presses for selection, regardless of active sort state
                    // Use a dummy entity if no active sort exists
                    let active_sort_entity = active_sort_state
                        .active_sort_entity
                        .unwrap_or(Entity::PLACEHOLDER);
                    handle_selection_key_press(
                        &mut commands,
                        key,
                        modifiers,
                        &selectable_query,
                        &selected_query,
                        &mut selection_state,
                        &mut event_writer,
                        active_sort_entity,
                        &sort_point_entities,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Handle mouse click for selection
pub fn handle_selection_click(
    commands: &mut Commands,
    position: &DPoint,
    modifiers: &crate::core::input::ModifierState,
    _drag_state: &mut ResMut<DragSelectionState>,
    drag_point_state: &mut ResMut<DragPointState>,
    event_writer: &mut EventWriter<EditEvent>,
    selectable_query: &Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    selected_query: &Query<(Entity, &Transform), With<Selected>>,
    selection_state: &mut ResMut<SelectionState>,
    active_sort_entity: Entity,
    sort_point_entities: &Query<&crate::systems::sort_manager::SortPointEntity>,
) {
    debug!("=== HANDLE SELECTION CLICK ===");
    let cursor_pos = position.to_raw();
    debug!("Click position: {:?} (raw: {:?})", position, cursor_pos);
    debug!("Modifiers: {:?}", modifiers);
    debug!("Active sort entity: {:?}", active_sort_entity);
    debug!(
        "Current selection count: {}",
        selection_state.selected.len()
    );

    // Count selectable points in active sort
    let mut total_selectable = 0;
    let mut active_sort_selectable = 0;
    for (entity, _, _, _) in selectable_query.iter() {
        total_selectable += 1;
        if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
            if sort_point_entity.sort_entity == active_sort_entity {
                active_sort_selectable += 1;
            }
        }
    }
    debug!(
        "Total selectable entities: {}, in active sort: {}",
        total_selectable, active_sort_selectable
    );

    debug!(
        "Selection click at ({:.1}, {:.1})",
        cursor_pos.x, cursor_pos.y
    );

    let mut best_hit = None;
    let mut min_dist_sq = SELECTION_MARGIN * SELECTION_MARGIN;
    debug!("Using selection margin: {}", SELECTION_MARGIN);

    // Find the closest selectable entity that belongs to the active sort
    let mut checked_points = 0;
    let mut points_in_active_sort = 0;

    for (entity, transform, glyph_ref, point_type) in selectable_query.iter() {
        checked_points += 1;

        // Check if this entity belongs to the active sort
        if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
            if sort_point_entity.sort_entity != active_sort_entity {
                continue; // Skip points that don't belong to the active sort
            }
            points_in_active_sort += 1;
        } else {
            debug!("Entity {:?} has no SortPointEntity component", entity);
            continue; // Skip entities that aren't sort points
        }

        let pos = transform.translation().truncate();
        let dist_sq = cursor_pos.distance_squared(pos);
        let distance = dist_sq.sqrt();

        debug!(
            "Point {:?} at {:?}, distance: {:.1} (squared: {:.1})",
            entity, pos, distance, dist_sq
        );

        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            best_hit = Some((entity, pos, glyph_ref, point_type));
            debug!("New best hit: {:?} at distance {:.1}", entity, distance);
        }
    }

    debug!(
        "Checked {} total points, {} in active sort",
        checked_points, points_in_active_sort
    );
    debug!("Best hit found: {:?}", best_hit.map(|(e, p, _, _)| (e, p)));

    if let Some((entity, pos, glyph_ref, point_type)) = best_hit {
        // Clicked on a selectable entity in the active sort
        commands.insert_resource(ClickWorldPosition);

        let shift_held = modifiers.shift;

        if !shift_held && selection_state.selected.contains(&entity) {
            // Clicked on already selected entity - start drag
            debug!(
                "Clicked on already-selected entity {:?} - starting drag",
                entity
            );
        } else {
            // Handle selection
            if !shift_held {
                // Clear previous selection
                debug!(
                    "Clearing previous selection (count: {})",
                    selection_state.selected.len()
                );
                for (e, _) in selected_query.iter() {
                    commands.entity(e).remove::<Selected>();
                }
                selection_state.selected.clear();
            }

            // Add to selection
            selection_state.selected.insert(entity);
            commands.entity(entity).insert(Selected);

            // Log point type for debugging
            if let Some(pt) = point_type {
                let point_type_str = if pt.is_on_curve {
                    "on-curve"
                } else {
                    "off-curve"
                };
                debug!(
                    "Selected {} point at ({:.1}, {:.1})",
                    point_type_str, pos.x, pos.y
                );
            }

            if let Some(glyph_ref) = glyph_ref {
                debug!(
                    "Selected point in glyph '{}', contour {}, point {}",
                    glyph_ref.glyph_name,
                    glyph_ref.contour_index,
                    glyph_ref.point_index
                );
            }
        }

        // Start drag operation
        if drag_point_state.is_dragging {
            debug!("WARNING: Starting drag while already dragging - resetting drag state");
            drag_point_state.is_dragging = false;
            drag_point_state.original_positions.clear();
            drag_point_state.dragged_entities.clear();
        }

        drag_point_state.is_dragging = true;
        drag_point_state.start_position = Some(cursor_pos);
        drag_point_state.current_position = Some(cursor_pos);

        // Include all currently selected entities in the drag operation
        drag_point_state.dragged_entities =
            selection_state.selected.iter().cloned().collect();
        debug!(
            "Starting drag with {} entities",
            drag_point_state.dragged_entities.len()
        );

        // Save original positions
        drag_point_state.original_positions.clear();
        for (entity, transform) in selected_query.iter() {
            if selection_state.selected.contains(&entity) {
                let pos =
                    Vec2::new(transform.translation.x, transform.translation.y);
                drag_point_state.original_positions.insert(entity, pos);
            }
        }

        // Also store position of newly clicked entity
        drag_point_state.original_positions.entry(entity).or_insert(pos);

        event_writer.write(EditEvent {
            edit_type: EditType::Normal,
        });

        debug!(
            "Selection updated and drag started. Current selection count: {}",
            selection_state.selected.len()
        );
    } else {
        // Clicked on empty space
        debug!("Clicked on empty space - clearing selection");
        commands.insert_resource(ClickWorldPosition);

        // Clear selection if not multi-selecting
        if !modifiers.shift {
            for (entity, _) in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
            debug!("Cleared all selections");
        }
    }
}

/// Handle mouse drag for selection
pub fn handle_selection_drag(
    commands: &mut Commands,
    start_position: &DPoint,
    current_position: &DPoint,
    delta: &Vec2,
    modifiers: &crate::core::input::ModifierState,
    drag_state: &mut ResMut<DragSelectionState>,
    _drag_point_state: &mut ResMut<DragPointState>,
    _event_writer: &mut EventWriter<EditEvent>,
    selectable_query: &Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    selection_state: &mut ResMut<SelectionState>,
    _active_sort_entity: Entity,
    _sort_point_entities: &Query<
        &crate::systems::sort_manager::SortPointEntity,
    >,
    _selection_rect_query: &Query<Entity, With<SelectionRect>>,
) {
    info!("[handle_selection_drag] Called: start={:?}, current={:?}, delta={:?}, is_dragging={}", start_position, current_position, delta, drag_state.is_dragging);
    if !drag_state.is_dragging {
        info!("[handle_selection_drag] Starting new drag selection at start={:?}, end={:?}", start_position.to_raw(), current_position.to_raw());

        // Initialize drag state
        drag_state.is_dragging = true;
        drag_state.start_position = Some(*start_position);
        drag_state.current_position = Some(*current_position);
        drag_state.is_multi_select = modifiers.shift;

        // Store previous selection for multi-select
        if modifiers.shift {
            drag_state.previous_selection =
                selection_state.selected.iter().cloned().collect();
        }

        // Spawn selection rectangle entity
        let rect_entity = commands
            .spawn(SelectionRect {
                start: start_position.to_raw(),
                end: current_position.to_raw(),
            })
            .id();
        drag_state.selection_rect_entity = Some(rect_entity);
        info!(
            "[handle_selection_drag] SelectionRect entity created with ID {:?}",
            rect_entity
        );
    } else {
        // Only update current_position during drag
        info!("Continuing existing drag selection...");
        drag_state.current_position = Some(*current_position);
        // Only update the entity if it exists
        if let Some(rect_entity) = drag_state.selection_rect_entity {
            info!("Updating SelectionRect entity {:?}", rect_entity);
            if let Ok(mut entity_commands) = commands.get_entity(rect_entity) {
                entity_commands.insert(SelectionRect {
                    start: drag_state
                        .start_position
                        .unwrap_or(*start_position)
                        .to_raw(),
                    end: current_position.to_raw(),
                });
                info!(
                    "SelectionRect entity updated: start={:?}, end={:?}",
                    drag_state
                        .start_position
                        .unwrap_or(*start_position)
                        .to_raw(),
                    current_position.to_raw()
                );
            } else {
                info!("ERROR: Could not get entity commands for SelectionRect entity {:?}", rect_entity);
            }
        } else {
            info!("ERROR: No SelectionRect entity found in drag_state!");
        }
    }

    // Update selection based on what's inside the rectangle
    if let (Some(start_pos), Some(current_pos)) =
        (drag_state.start_position, drag_state.current_position)
    {
        info!(
            "Selection: Marquee rect coordinates - start: {:?}, current: {:?}",
            start_pos, current_pos
        );

        // In multi-select mode, start with previous selection
        if drag_state.is_multi_select {
            // Reset to previous selection
            for (entity, _, _, _) in selectable_query.iter() {
                if !drag_state.previous_selection.contains(&entity) {
                    commands.entity(entity).remove::<Selected>();
                    selection_state.selected.remove(&entity);
                }
            }

            for &entity in &drag_state.previous_selection {
                if !selection_state.selected.contains(&entity) {
                    commands.entity(entity).insert(Selected);
                    selection_state.selected.insert(entity);
                }
            }
        } else {
            // Clear selection for non-multi-select
            for (entity, _, _, _) in selectable_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
        }

        // Add entities in the rectangle to selection
        let mut points_in_rect = 0;
        let mut points_selected = 0;

        info!(
            "Selection: Checking {} selectable entities for marquee selection",
            selectable_query.iter().count()
        );

        // Collect entity positions for debugging and coordinate system analysis
        let entity_positions: Vec<(Entity, Vec2)> = selectable_query
            .iter()
            .map(|(entity, transform, _, _)| {
                (entity, transform.translation().truncate())
            })
            .collect();

        // Use centralized coordinate system for debugging
        let debug_info = SelectionCoordinateSystem::debug_coordinate_ranges(
            &entity_positions,
            &start_pos,
            &current_pos,
        );
        info!("Selection: {}", debug_info);

        for (entity, entity_pos) in &entity_positions {
            // Use centralized coordinate system to check if entity is inside the marquee rectangle
            if SelectionCoordinateSystem::is_point_in_rectangle(
                entity_pos,
                &start_pos,
                &current_pos,
            ) {
                points_in_rect += 1;
                info!("Selection: Entity {:?} is inside marquee rect! Position: {:?}", entity, entity_pos);
                if drag_state.is_multi_select
                    && drag_state.previous_selection.contains(entity)
                {
                    // Toggle off if previously selected
                    selection_state.selected.remove(entity);
                    commands.entity(*entity).remove::<Selected>();
                    info!("Selection: Toggled off entity {:?}", entity);
                } else {
                    // Add to selection
                    selection_state.selected.insert(*entity);
                    commands.entity(*entity).insert(Selected);
                    points_selected += 1;
                    info!("Selection: Added entity {:?} to selection", entity);
                }
            } else {
                // Debug: Show why entity is not selected using centralized coordinate system
                let rect_entity_start =
                    SelectionCoordinateSystem::design_to_entity_coordinates(
                        &start_pos,
                    );
                let rect_entity_end =
                    SelectionCoordinateSystem::design_to_entity_coordinates(
                        &current_pos,
                    );

                let distance_x = if entity_pos.x
                    < rect_entity_start.x.min(rect_entity_end.x)
                {
                    rect_entity_start.x.min(rect_entity_end.x) - entity_pos.x
                } else if entity_pos.x
                    > rect_entity_start.x.max(rect_entity_end.x)
                {
                    entity_pos.x - rect_entity_start.x.max(rect_entity_end.x)
                } else {
                    0.0
                };

                let distance_y = if entity_pos.y
                    < rect_entity_start.y.min(rect_entity_end.y)
                {
                    rect_entity_start.y.min(rect_entity_end.y) - entity_pos.y
                } else if entity_pos.y
                    > rect_entity_start.y.max(rect_entity_end.y)
                {
                    entity_pos.y - rect_entity_start.y.max(rect_entity_end.y)
                } else {
                    0.0
                };

                if distance_x > 0.0 || distance_y > 0.0 {
                    info!("Selection: Entity {:?} outside rect - X: {:.1} units, Y: {:.1} units", 
                          entity, distance_x, distance_y);
                }
            }
        }

        info!(
            "Marquee selection: {} points in rect, {} points selected",
            points_in_rect, points_selected
        );

        info!(
            "Marquee selection updated: {} points selected",
            selection_state.selected.len()
        );
    }
}

/// Handle mouse release for selection
pub fn handle_selection_release(
    commands: &mut Commands,
    position: &DPoint,
    _modifiers: &crate::core::input::ModifierState,
    drag_state: &mut ResMut<DragSelectionState>,
    drag_point_state: &mut ResMut<DragPointState>,
    _event_writer: &mut EventWriter<EditEvent>,
    selection_state: &mut ResMut<SelectionState>,
    _selection_rect_query: &Query<Entity, With<SelectionRect>>,
) {
    let release_pos = position.to_raw();
    debug!(
        "Selection release at ({:.1}, {:.1})",
        release_pos.x, release_pos.y
    );

    if drag_point_state.is_dragging {
        drag_point_state.is_dragging = false;
        drag_point_state.start_position = None;
        drag_point_state.current_position = None;
        drag_point_state.dragged_entities.clear();
        drag_point_state.original_positions.clear();
    } else if drag_state.is_dragging {
        // End drag selection
        drag_state.is_dragging = false;
        let rect_entity = drag_state.selection_rect_entity;
        drag_state.selection_rect_entity = None;
        drag_state.start_position = None;
        drag_state.current_position = None;
        drag_state.is_multi_select = false;

        debug!(
            "Ended drag selection. Final selection count: {}",
            selection_state.selected.len()
        );

        info!(
            "Marquee selection complete. {} points selected.",
            selection_state.selected.len()
        );
        // Clean up the selection rectangle entity
        if let Some(rect_entity) = rect_entity {
            commands.entity(rect_entity).despawn();
            info!("SelectionRect entity despawned on release");
        }
    }
}

/// Handle key press for selection shortcuts
pub fn handle_selection_key_press(
    commands: &mut Commands,
    key: &KeyCode,
    modifiers: &crate::core::input::ModifierState,
    selectable_query: &Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    selected_query: &Query<(Entity, &Transform), With<Selected>>,
    selection_state: &mut ResMut<SelectionState>,
    event_writer: &mut EventWriter<EditEvent>,
    active_sort_entity: Entity,
    sort_point_entities: &Query<&crate::systems::sort_manager::SortPointEntity>,
) {
    match key {
        KeyCode::KeyA => {
            if modifiers.ctrl {
                // Ctrl+A: Select all points in the active sort
                debug!("Select all shortcut triggered for active sort");
                let mut selected_count = 0;

                for (entity, _, _, _) in selectable_query.iter() {
                    // Check if this entity belongs to the active sort
                    if let Ok(sort_point_entity) =
                        sort_point_entities.get(entity)
                    {
                        if sort_point_entity.sort_entity != active_sort_entity {
                            continue; // Skip points that don't belong to the active sort
                        }
                    } else {
                        continue; // Skip entities that aren't sort points
                    }

                    if !selection_state.selected.contains(&entity) {
                        selection_state.selected.insert(entity);
                        commands.entity(entity).insert(Selected);
                        selected_count += 1;
                    }
                }

                event_writer.write(EditEvent {
                    edit_type: EditType::Normal,
                });
                debug!("Selected all {} points in active sort", selected_count);
            }
        }
        KeyCode::Escape => {
            // Escape: Clear selection
            debug!("Escape key pressed - clearing selection");
            for (entity, _) in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
            event_writer.write(EditEvent {
                edit_type: EditType::Normal,
            });
        }
        _ => {}
    }
}

/// TEMP: Debug system to print all SelectionRect entities every frame
pub fn debug_print_selection_rects(
    selection_rects: Query<(Entity, &SelectionRect)>,
) {
    let count = selection_rects.iter().count();
    if count > 0 {
        info!(
            "[debug_print_selection_rects] Found {} SelectionRect entities:",
            count
        );
        for (entity, rect) in selection_rects.iter() {
            info!(
                "  Entity {:?}: start={:?}, end={:?}",
                entity, rect.start, rect.end
            );
        }
    }
}

/// System to render all point entities (not just selected ones)
pub fn render_all_point_entities(
    mut gizmos: Gizmos,
    point_entities: Query<
        (
            Entity,
            &GlobalTransform,
            &crate::editing::selection::components::PointType,
        ),
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
    selected_query: Query<Entity, With<Selected>>,
    camera_query: Query<
        (&Camera, &GlobalTransform, &Projection),
        With<crate::rendering::cameras::DesignCamera>,
    >,
    // Check nudge state to avoid double rendering during nudging
    nudge_state: Res<crate::editing::selection::nudge::NudgeState>,
) {
    // CRITICAL: ALWAYS skip this system during nudging - no exceptions
    // Force all rendering through the unified live system to prevent flashing
    if nudge_state.is_nudging {
        return; // No debug spam, just skip
    }

    let point_count = point_entities.iter().count();
    info!(
        "[render_all_point_entities] Called, found {} point entities",
        point_count
    );

    // Debug camera information
    if let Ok((_camera, camera_transform, projection)) = camera_query.single() {
        let camera_pos = camera_transform.translation();
        let camera_scale = match projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => 1.0,
        };
        info!("[render_all_point_entities] Camera: pos=({:.1}, {:.1}, {:.1}), scale={:.3}", 
              camera_pos.x, camera_pos.y, camera_pos.z, camera_scale);
    } else {
        warn!("[render_all_point_entities] No camera found");
    }

    for (i, (entity, transform, point_type)) in
        point_entities.iter().enumerate()
    {
        // Skip rendering if this point is selected (it will be drawn by render_selected_entities)
        if selected_query.get(entity).is_ok() {
            continue;
        }
        let position = transform.translation().truncate();

        // Debug: Print first few point positions being rendered
        if i < 5 {
            info!("[render_all_point_entities] Rendering point {} at ({:.1}, {:.1}), is_on_curve={}", 
                  i, position.x, position.y, point_type.is_on_curve);
        }

        // Use different colors and sizes based on point type
        let (size, color) = if point_type.is_on_curve {
            (
                crate::ui::theme::ON_CURVE_POINT_RADIUS,
                crate::ui::theme::ON_CURVE_POINT_COLOR,
            )
        } else {
            (
                crate::ui::theme::OFF_CURVE_POINT_RADIUS,
                crate::ui::theme::OFF_CURVE_POINT_COLOR,
            )
        };

        if point_type.is_on_curve && crate::ui::theme::USE_SQUARE_FOR_ON_CURVE {
            // Draw a square for on-curve points
            let half_size = size / crate::ui::theme::ON_CURVE_SQUARE_ADJUSTMENT;
            let square_size = Vec2::new(size * 2.0, size * 2.0);
            gizmos.rect_2d(position, square_size, color);

            // Draw a smaller inner circle
            let inner_radius =
                half_size * crate::ui::theme::ON_CURVE_INNER_CIRCLE_RATIO;
            gizmos.circle_2d(position, inner_radius, color);
        } else {
            // Draw a circle for off-curve points
            gizmos.circle_2d(position, size, color);

            // Draw a smaller inner circle
            let inner_radius =
                size * crate::ui::theme::OFF_CURVE_INNER_CIRCLE_RATIO;
            gizmos.circle_2d(position, inner_radius, color);
        }
    }
}

/// Render control handles between on-curve points and their off-curve control points
pub fn render_control_handles(
    mut gizmos: Gizmos,
    point_entities: Query<
        (
            &GlobalTransform,
            &crate::editing::selection::components::PointType,
            &crate::editing::selection::components::GlyphPointReference,
        ),
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
    text_editor_state: Res<crate::core::state::TextEditorState>,
    app_state: Res<crate::core::state::AppState>,
    // Check nudge state to avoid double rendering during nudging
    nudge_state: Res<crate::editing::selection::nudge::NudgeState>,
) {
    // CRITICAL: Skip handle rendering during nudging to prevent visual disconnection
    // The live rendering system will handle control handles during nudging
    if nudge_state.is_nudging {
        debug!("[render_control_handles] SKIPPING - nudging in progress, live renderer handles control handles");
        return;
    }

    // Get the active sort to find the glyph data
    let Some((_active_sort_index, active_sort)) =
        text_editor_state.get_active_sort()
    else {
        return; // No active sort, no handles to render
    };

    let glyph_name = active_sort.kind.glyph_name();
    let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name)
    else {
        return; // No glyph data found
    };

    let Some(outline) = &glyph_data.outline else {
        return; // No outline data
    };

    // Group points by contour to process them together
    let mut contour_points: Vec<Vec<(Vec2, bool, usize)>> =
        vec![Vec::new(); outline.contours.len()];

    // Collect all point entities and their positions
    for (transform, point_type, glyph_ref) in point_entities.iter() {
        let position = transform.translation().truncate();
        let is_on_curve = point_type.is_on_curve;
        let contour_index = glyph_ref.contour_index;
        let point_index = glyph_ref.point_index;

        if contour_index < contour_points.len() {
            contour_points[contour_index].push((
                position,
                is_on_curve,
                point_index,
            ));
        }
    }

    // Sort points within each contour by their original index
    for contour_points in &mut contour_points {
        contour_points.sort_by_key(|(_, _, index)| *index);
    }

    // Render handles for each contour
    for contour_points in contour_points {
        if contour_points.len() < 2 {
            continue;
        }

        render_contour_handles(&mut gizmos, &contour_points);
    }
}

/// Render handles for a single contour
fn render_contour_handles(
    gizmos: &mut Gizmos,
    contour_points: &[(Vec2, bool, usize)],
) {
    let handle_color = crate::ui::theme::HANDLE_LINE_COLOR;
    let len = contour_points.len();
    if len < 2 {
        return;
    }
    // Walk through all points in order
    for i in 0..len {
        let (curr_pos, curr_on, _) = contour_points[i];
        let next_idx = (i + 1) % len;
        let (next_pos, next_on, _) = contour_points[next_idx];
        // If current is on-curve and next is off-curve, draw handle
        if curr_on && !next_on {
            gizmos.line_2d(curr_pos, next_pos, handle_color);
        }
        // If current is off-curve and next is on-curve, draw handle
        if !curr_on && next_on {
            gizmos.line_2d(curr_pos, next_pos, handle_color);
        }
    }
}

#[cfg(debug_assertions)]
pub fn debug_validate_point_entity_uniqueness(
    point_entities: Query<
        (
            &crate::editing::selection::components::GlyphPointReference,
            Entity,
        ),
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
) {
    use std::collections::HashMap;
    let mut seen: HashMap<(String, usize, usize), Entity> = HashMap::new();
    for (point_ref, entity) in point_entities.iter() {
        let key = (
            point_ref.glyph_name.clone(),
            point_ref.contour_index,
            point_ref.point_index,
        );
        if let Some(existing) = seen.insert(key.clone(), entity) {
            warn!(
                "[debug_validate_point_entity_uniqueness] Duplicate ECS entities for glyph='{}' contour={} point={}: {:?} and {:?}",
                point_ref.glyph_name, point_ref.contour_index, point_ref.point_index, existing, entity
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::selection::coordinate_system::SelectionCoordinateSystem;
    use crate::geometry::design_space::DPoint;
    use bevy::prelude::*;

    #[test]
    fn test_point_selection_matches_render_position() {
        let point_pos = Vec2::new(100.0, 200.0);
        let marquee_start = DPoint::from_raw(point_pos);
        let marquee_end = DPoint::from_raw(point_pos + Vec2::splat(1.0));
        let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
            &point_pos,
            &marquee_start,
            &marquee_end,
        );
        println!("[test_point_selection_matches_render_position] point={:?}, marquee=({:?}, {:?}), in_rect={}", point_pos, marquee_start, marquee_end, in_rect);
        assert!(in_rect, "Point should be inside the marquee rectangle");
    }

    #[test]
    fn test_parented_transform_selection() {
        // Simulate a point parented to a group at (50, 50)
        let parent_offset = Vec2::new(50.0, 50.0);
        let local_point = Vec2::new(10.0, 10.0);
        let world_point = parent_offset + local_point;
        let marquee_start = DPoint::from_raw(world_point);
        let marquee_end = DPoint::from_raw(world_point + Vec2::splat(1.0));
        let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
            &world_point,
            &marquee_start,
            &marquee_end,
        );
        println!("[test_parented_transform_selection] world_point={:?}, marquee=({:?}, {:?}), in_rect={}", world_point, marquee_start, marquee_end, in_rect);
        assert!(
            in_rect,
            "Parented point should be inside the marquee rectangle"
        );
    }

    #[test]
    fn test_off_curve_point_selection() {
        // Simulate an off-curve point at a known position
        let off_curve_pos = Vec2::new(-123.4, 567.8);
        let marquee_start = DPoint::from_raw(off_curve_pos - Vec2::splat(0.5));
        let marquee_end = DPoint::from_raw(off_curve_pos + Vec2::splat(0.5));
        let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
            &off_curve_pos,
            &marquee_start,
            &marquee_end,
        );
        println!("[test_off_curve_point_selection] off_curve_pos={:?}, marquee=({:?}, {:?}), in_rect={}", off_curve_pos, marquee_start, marquee_end, in_rect);
        assert!(
            in_rect,
            "Off-curve point should be inside the marquee rectangle"
        );
    }

    #[test]
    fn test_text_editor_sort_coordinate_system() {
        // Test the coordinate system used by text editor sorts
        // Based on the logs, points are at Y=-1000 range, marquee at Y=200-400 range

        // Simulate a sort point position (like from text editor)
        let sort_point_pos = Vec2::new(-96.0, -928.0); // From logs

        // Simulate marquee selection coordinates (from logs)
        let marquee_start = DPoint::from_raw(Vec2::new(481.9, 249.1));
        let marquee_end = DPoint::new(233.0, 398.5);

        let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
            &sort_point_pos,
            &marquee_start,
            &marquee_end,
        );

        println!("[test_text_editor_sort_coordinate_system]");
        println!("  sort_point_pos={:?}", sort_point_pos);
        println!("  marquee_start={:?}", marquee_start);
        println!("  marquee_end={:?}", marquee_end);
        println!("  in_rect={}", in_rect);

        // Calculate distances for debugging
        let rect_start_vec = marquee_start.to_raw();
        let rect_end_vec = marquee_end.to_raw();
        let min_x = rect_start_vec.x.min(rect_end_vec.x);
        let max_x = rect_start_vec.x.max(rect_end_vec.x);
        let min_y = rect_start_vec.y.min(rect_end_vec.y);
        let max_y = rect_start_vec.y.max(rect_end_vec.y);

        let distance_x = if sort_point_pos.x < min_x {
            min_x - sort_point_pos.x
        } else if sort_point_pos.x > max_x {
            sort_point_pos.x - max_x
        } else {
            0.0
        };

        let distance_y = if sort_point_pos.y < min_y {
            min_y - sort_point_pos.y
        } else if sort_point_pos.y > max_y {
            sort_point_pos.y - max_y
        } else {
            0.0
        };

        println!(
            "  rect_bounds: X({:.1} to {:.1}), Y({:.1} to {:.1})",
            min_x, max_x, min_y, max_y
        );
        println!("  distances: X={:.1}, Y={:.1}", distance_x, distance_y);

        // This should fail because the coordinate systems don't match
        // The test documents the expected behavior
        assert!(!in_rect, "Sort point should NOT be in marquee due to coordinate system mismatch");
    }

    #[test]
    fn test_coordinate_system_conversion() {
        // Test the coordinate conversion functions
        let design_point = DPoint::new(100.0, 200.0);
        let entity_coords =
            SelectionCoordinateSystem::design_to_entity_coordinates(
                &design_point,
            );
        let back_to_design =
            SelectionCoordinateSystem::entity_to_design_coordinates(
                &entity_coords,
            );

        println!("[test_coordinate_system_conversion]");
        println!("  design_point={:?}", design_point);
        println!("  entity_coords={:?}", entity_coords);
        println!("  back_to_design={:?}", back_to_design);

        assert_eq!(
            design_point.to_raw(),
            entity_coords,
            "Design to entity conversion should be identity"
        );
        assert_eq!(
            design_point, back_to_design,
            "Entity to design conversion should be identity"
        );
    }

    #[test]
    fn test_marquee_selection_edge_cases() {
        // Test various edge cases for marquee selection

        // Case 1: Point exactly on rectangle edge
        let point_on_edge = Vec2::new(100.0, 200.0);
        let marquee_edge = DPoint::from_raw(Vec2::new(100.0, 200.0));
        let marquee_end = DPoint::from_raw(Vec2::new(200.0, 300.0));

        let on_edge = SelectionCoordinateSystem::is_point_in_rectangle(
            &point_on_edge,
            &marquee_edge,
            &marquee_end,
        );

        println!("[test_marquee_selection_edge_cases]");
        println!(
            "  point_on_edge={:?}, marquee=({:?}, {:?}), on_edge={}",
            point_on_edge, marquee_edge, marquee_end, on_edge
        );

        // Case 2: Point just outside rectangle
        let point_outside = Vec2::new(99.9, 200.0);
        let just_outside = SelectionCoordinateSystem::is_point_in_rectangle(
            &point_outside,
            &marquee_edge,
            &marquee_end,
        );

        println!(
            "  point_outside={:?}, just_outside={}",
            point_outside, just_outside
        );

        // Case 3: Inverted rectangle (end before start)
        let inverted_start = DPoint::from_raw(Vec2::new(200.0, 300.0));
        let inverted_end = DPoint::from_raw(Vec2::new(100.0, 200.0));
        let point_inside = Vec2::new(150.0, 250.0);

        let in_inverted = SelectionCoordinateSystem::is_point_in_rectangle(
            &point_inside,
            &inverted_start,
            &inverted_end,
        );

        println!(
            "  point_inside={:?}, in_inverted={}",
            point_inside, in_inverted
        );

        // Assertions
        assert!(on_edge, "Point on edge should be considered inside");
        assert!(!just_outside, "Point just outside should not be inside");
        assert!(in_inverted, "Point should be inside inverted rectangle");
    }

    #[test]
    fn test_real_world_coordinate_mismatch() {
        // Test with real-world coordinates from the logs
        // This test documents the actual problem we're seeing

        // From logs: Entity ranges: X(-56.0 to 472.0), Y(-1104.0 to -496.0)
        // From logs: Rect entity coords: start(481.9, 249.1), end(233.0, 398.5)

        let entity_positions = vec![
            Vec2::new(-56.0, -1104.0), // Bottom-left entity
            Vec2::new(472.0, -496.0),  // Top-right entity
            Vec2::new(100.0, -800.0),  // Middle entity
        ];

        let marquee_start = DPoint::from_raw(Vec2::new(481.9, 249.1));
        let marquee_end = DPoint::from_raw(Vec2::new(233.0, 398.5));

        println!("[test_real_world_coordinate_mismatch]");
        println!("  marquee: ({:?}, {:?})", marquee_start, marquee_end);

        for (i, pos) in entity_positions.iter().enumerate() {
            let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
                pos,
                &marquee_start,
                &marquee_end,
            );

            println!("  entity[{}]={:?}, in_rect={}", i, pos, in_rect);

            // All should fail due to Y coordinate mismatch
            assert!(!in_rect, "Entity {} should not be in marquee due to Y coordinate mismatch", i);
        }

        // Test what Y coordinate the marquee would need to be at to select these entities
        let entity_y_min = -1104.0;
        let entity_y_max = -496.0;
        let marquee_y_min = 233.0;
        let marquee_y_max = 398.5;

        println!(
            "  entity Y range: {:.1} to {:.1}",
            entity_y_min, entity_y_max
        );
        println!(
            "  marquee Y range: {:.1} to {:.1}",
            marquee_y_min, marquee_y_max
        );
        println!(
            "  Y coordinate gap: {:.1} units",
            marquee_y_min - entity_y_max
        );
    }

    #[test]
    fn test_outline_point_selection_coordinate_mismatch() {
        // This test documents the old bug that was fixed
        // Marquee selection rectangle: Y = -400 to -769
        // Outline points: Y = 750 to 1358 (before the fix)
        // Result: No points selected due to coordinate system mismatch

        let marquee_start = DPoint::from_raw(Vec2::new(-160.8, -403.2));
        let marquee_end = DPoint::from_raw(Vec2::new(334.3, -769.3));

        // Test points at the old Y coordinates from the logs (before the fix)
        let test_points = vec![
            Vec2::new(100.0, 750.0),  // Entity 245v141
            Vec2::new(200.0, 926.7),  // Entity 323v141
            Vec2::new(300.0, 1038.7), // Entity 941v389
            Vec2::new(400.0, 1182.7), // Entity 527v135
            Vec2::new(500.0, 1358.7), // Entity 567v135
        ];

        println!("=== Outline Point Selection Coordinate Mismatch Test (Documenting Old Bug) ===");
        println!(
            "Marquee rectangle: Y = {:.1} to {:.1}",
            marquee_start.y, marquee_end.y
        );
        println!("Outline points Y range: {:.1} to {:.1}", 750.0, 1358.7);
        println!("Y coordinate gap: {:.1} units", 750.0 - (-769.3));

        let mut points_in_rect = 0;
        for (_i, point_pos) in test_points.iter().enumerate() {
            let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
                point_pos,
                &marquee_start,
                &marquee_end,
            );
            if in_rect {
                points_in_rect += 1;
            }
        }

        println!("Result: {} points in rect (correctly 0 due to old coordinate system bug)", points_in_rect);

        // This test documents the old bug - it should pass (0 points selected)
        assert_eq!(
            points_in_rect, 0,
            "No points should be selected due to old coordinate system bug"
        );
    }

    #[test]
    fn test_outline_point_selection_after_fix() {
        // This test shows the fix working with corrected coordinate values
        // After the fix: sort positions are at baseline (cursor position)
        // Outline points are at sort_position + glyph_coordinates (32-768)
        // So outline points should be at Y ≈ 0-800 instead of Y ≈ 750-1358

        let marquee_start = DPoint::from_raw(Vec2::new(-160.8, 100.0));
        let marquee_end = DPoint::from_raw(Vec2::new(334.3, 400.0));

        // Test points at corrected Y coordinates (after the fix)
        let test_points = vec![
            Vec2::new(100.0, 150.0), // Sort at Y=100 + glyph Y=50
            Vec2::new(200.0, 250.0), // Sort at Y=200 + glyph Y=50
            Vec2::new(300.0, 350.0), // Sort at Y=300 + glyph Y=50
            Vec2::new(400.0, 450.0), // Sort at Y=400 + glyph Y=50
            Vec2::new(500.0, 550.0), // Sort at Y=500 + glyph Y=50
        ];

        println!("=== Outline Point Selection After Fix Test ===");
        println!(
            "Marquee rectangle: Y = {:.1} to {:.1}",
            marquee_start.y, marquee_end.y
        );
        println!("Outline points Y range: {:.1} to {:.1}", 150.0, 550.0);

        let mut points_in_rect = 0;
        for (i, point_pos) in test_points.iter().enumerate() {
            let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
                point_pos,
                &marquee_start,
                &marquee_end,
            );
            println!("  Point {} at {:?}: in_rect = {}", i, point_pos, in_rect);
            if in_rect {
                points_in_rect += 1;
            }
        }

        println!("Result: {} points in rect (should be 3)", points_in_rect);

        // After the fix, points should be selectable by the marquee
        assert_eq!(
            points_in_rect, 3,
            "Points should be selectable after coordinate fix"
        );
    }
}
