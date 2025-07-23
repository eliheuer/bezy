//! Shared sort rendering for both text buffer and freeform/entity sorts

use crate::core::state::font_data::OutlineData;
use crate::core::state::font_metrics::FontMetrics;
use crate::core::state::{FontIRAppState, SortLayoutMode};
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selected,
};
use crate::editing::selection::nudge::NudgeState;
use crate::rendering::camera_responsive::CameraResponsiveScale;
use crate::rendering::fontir_glyph_outline::{
    draw_fontir_glyph_outline_at_position, get_fontir_glyph_paths,
};
use crate::rendering::glyph_outline::{
    draw_glyph_outline_at_position, draw_glyph_outline_from_live_transforms,
};
use crate::rendering::metrics::draw_metrics_at_position;
use crate::systems::sort_manager::SortPointEntity;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::BezPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortRenderStyle {
    TextBuffer,
    Freeform,
}

/// Component to mark entities as sort handle visual elements
#[derive(Component)]
pub struct SortHandle {
    pub sort_entity: Entity,
    pub handle_type: SortHandleType,
}

/// Types of sort handle elements
#[derive(Debug, Clone)]
pub enum SortHandleType {
    Square,
    Circle,
    SelectionIndicator,
}

/// Resource to track sort handle entities
#[derive(Resource, Default)]
pub struct SortHandleEntities {
    pub handles: std::collections::HashMap<Entity, Vec<Entity>>, // sort_entity -> handle entities
}

/// Z-levels for sort handles
const SORT_HANDLE_Z: f32 = 20.0; // Above glyph editing elements

/// Helper to spawn a mesh-based square handle
fn spawn_square_handle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    size: f32,
    color: Color,
    sort_entity: Entity,
    handle_type: SortHandleType,
    camera_scale: &CameraResponsiveScale,
) -> Entity {
    // Create square outline using 4 line segments with camera-responsive width
    let half_size = camera_scale.adjusted_handle_size(size);
    let line_width = camera_scale.adjusted_line_width();

    // Create a container entity for the 4 lines
    let container = commands
        .spawn((
            SortHandle {
                sort_entity,
                handle_type,
            },
            Transform::from_translation(position.extend(SORT_HANDLE_Z)),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    // Top line
    let top_line = crate::rendering::mesh_glyph_outline::create_line_mesh(
        Vec2::new(-half_size, half_size),
        Vec2::new(half_size, half_size),
        line_width,
    );
    commands
        .spawn((
            Mesh2d(meshes.add(top_line)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .insert(ChildOf(container));

    // Right line
    let right_line = crate::rendering::mesh_glyph_outline::create_line_mesh(
        Vec2::new(half_size, half_size),
        Vec2::new(half_size, -half_size),
        line_width,
    );
    commands
        .spawn((
            Mesh2d(meshes.add(right_line)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .insert(ChildOf(container));

    // Bottom line
    let bottom_line = crate::rendering::mesh_glyph_outline::create_line_mesh(
        Vec2::new(half_size, -half_size),
        Vec2::new(-half_size, -half_size),
        line_width,
    );
    commands
        .spawn((
            Mesh2d(meshes.add(bottom_line)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .insert(ChildOf(container));

    // Left line
    let left_line = crate::rendering::mesh_glyph_outline::create_line_mesh(
        Vec2::new(-half_size, -half_size),
        Vec2::new(-half_size, half_size),
        line_width,
    );
    commands
        .spawn((
            Mesh2d(meshes.add(left_line)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .insert(ChildOf(container));

    container
}

/// Helper to spawn a mesh-based circle handle
fn spawn_circle_handle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    radius: f32,
    color: Color,
    sort_entity: Entity,
    handle_type: SortHandleType,
    camera_scale: &CameraResponsiveScale,
) -> Entity {
    // Create circle outline using line segments with camera-responsive width
    let line_width = camera_scale.adjusted_line_width();
    let adjusted_radius = camera_scale.adjusted_handle_size(radius);
    let segments = 24; // 24 segments for smooth circle

    let container = commands
        .spawn((
            SortHandle {
                sort_entity,
                handle_type,
            },
            Transform::from_translation(position.extend(SORT_HANDLE_Z)),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    // Create circle segments
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let angle2 =
            ((i + 1) as f32 / segments as f32) * 2.0 * std::f32::consts::PI;

        let start = Vec2::new(
            angle1.cos() * adjusted_radius,
            angle1.sin() * adjusted_radius,
        );
        let end = Vec2::new(
            angle2.cos() * adjusted_radius,
            angle2.sin() * adjusted_radius,
        );

        let segment_line =
            crate::rendering::mesh_glyph_outline::create_line_mesh(
                start, end, line_width,
            );
        commands
            .spawn((
                Mesh2d(meshes.add(segment_line)),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
                Transform::from_translation(Vec3::ZERO),
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .insert(ChildOf(container));
    }

    container
}

/// Draws a sort (outline, metrics, handles) at the given position with the given style
pub fn render_sort_visuals(
    gizmos: &mut Gizmos,
    outline: &Option<OutlineData>,
    advance_width: f32,
    metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
    style: SortRenderStyle,
) {
    render_sort_visuals_with_selection(
        gizmos,
        outline,
        advance_width,
        metrics,
        position,
        metrics_color,
        style,
        false, // not selected
        false, // not active
    );
}

/// Draws a sort with selection and activation state support
#[allow(clippy::too_many_arguments)]
pub fn render_sort_visuals_with_selection(
    gizmos: &mut Gizmos,
    outline: &Option<OutlineData>,
    advance_width: f32,
    metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
    style: SortRenderStyle,
    is_selected: bool,
    is_active: bool,
) {
    // Determine colors based on state
    let handle_color = if is_selected {
        Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
    } else if is_active {
        Color::srgb(0.0, 1.0, 0.0) // Green for active
    } else {
        metrics_color // Default metrics color
    };

    let metrics_render_color = if is_active {
        Color::srgb(0.0, 1.0, 0.0) // Green metrics for active sorts
    } else {
        metrics_color // Default metrics color
    };

    // Draw outline - DISABLED: Now using mesh-based rendering
    // draw_glyph_outline_at_position(gizmos, outline, position);
    // Draw metrics
    draw_metrics_at_position(
        gizmos,
        advance_width,
        metrics,
        position,
        metrics_render_color,
    );

    // Draw handle at descender position (matching click detection logic)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    let handle_position = position + Vec2::new(0.0, descender);

    // Normal handle: smaller and uses appropriate color
    let normal_size = 16.0;

    // DISABLED: Gizmo-based handle rendering - now using mesh system
    // match style {
    //     SortRenderStyle::TextBuffer => {
    //         // Square handle for text sorts
    //         let square_size = Vec2::new(normal_size * 2.0, normal_size * 2.0);
    //         gizmos.rect_2d(handle_position, square_size, handle_color);
    //     }
    //     SortRenderStyle::Freeform => {
    //         // Circle handle for freeform sorts
    //         gizmos.circle_2d(handle_position, normal_size, handle_color);
    //     }
    // }
}

/// Enhanced sort rendering that uses live Transform positions during nudging
/// This ensures perfect synchronization between points and outline
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn render_sort_visuals_with_live_sync(
    gizmos: &mut Gizmos,
    outline: &Option<OutlineData>,
    advance_width: f32,
    metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
    style: SortRenderStyle,
    // Live rendering parameters
    sort_entity: Option<Entity>,
    sort_transform: Option<&Transform>,
    glyph_name: Option<&str>,
    #[allow(clippy::type_complexity)] point_query: Option<
        &Query<
            (
                Entity,
                &Transform,
                &GlyphPointReference,
                &crate::editing::selection::components::PointType,
            ),
            With<SortPointEntity>,
        >,
    >,
    selected_query: Option<
        &Query<Entity, With<crate::editing::selection::components::Selected>>,
    >,
    app_state: Option<&crate::core::state::AppState>,
    nudge_state: Option<&NudgeState>,
) {
    // Determine if we should use live rendering
    // FIXED: Check if we have selected points, not just if nudging is active
    // This ensures outline stays synced with Transform positions even after nudging stops
    let nudge_active = nudge_state.is_some_and(|ns| ns.is_nudging);
    let has_sort_entity = sort_entity.is_some();
    let has_sort_transform = sort_transform.is_some();
    let has_glyph_name = glyph_name.is_some();
    let has_point_query = point_query.is_some();
    let has_app_state = app_state.is_some();
    let has_selected_query = selected_query.is_some();

    // Check if there are any selected points for this sort
    let has_selected_points = if let (
        Some(_sort_entity_val),
        Some(point_query),
        Some(selected_query),
    ) = (sort_entity, point_query, selected_query)
    {
        point_query.iter().any(|(entity, _, _, _)| {
            // Check if this point is selected
            selected_query.get(entity).is_ok()
        })
    } else {
        false
    };

    // Use live rendering during nudging OR when there are selected points OR when there are any points visible
    // This ensures handles are always visible when points are shown
    let has_any_points = if let Some(point_query) = point_query {
        !point_query.is_empty()
    } else {
        false
    };

    let use_live_rendering =
        (nudge_active || has_selected_points || has_any_points)
            && has_sort_entity
            && has_sort_transform
            && has_glyph_name
            && has_point_query
            && has_selected_query
            && has_app_state;

    // Debug logging with println for visibility
    static COUNTER: std::sync::atomic::AtomicU32 =
        std::sync::atomic::AtomicU32::new(0);
    let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if count % 60 == 0 {
        // Every second
        println!("[LIVE RENDER CHECK] use_live_rendering={}, has_any_points={}, nudge_active={}, has_selected_points={}", 
               use_live_rendering, has_any_points, nudge_active, has_selected_points);
    }

    // Draw outline with appropriate method
    // DISABLED: All gizmo-based outline rendering - now using mesh system exclusively
    // if use_live_rendering {
    //     println!("[LIVE RENDER] *** USING LIVE TRANSFORM POSITIONS FOR OUTLINE RENDERING ***");
    //     draw_glyph_outline_from_live_transforms(
    //         gizmos,
    //         sort_entity.unwrap(),
    //         sort_transform.unwrap(),
    //         glyph_name.unwrap(),
    //         point_query.unwrap(),
    //         app_state.unwrap(),
    //         selected_query.unwrap(),
    //     );
    // } else {
    //     // Use normal rendering from glyph data
    //     debug!("[NORMAL RENDER] Using glyph data for outline rendering");
    //     // draw_glyph_outline_at_position(gizmos, outline, position); // DISABLED: Using mesh rendering
    // }

    // Draw metrics (always from original data)
    draw_metrics_at_position(
        gizmos,
        advance_width,
        metrics,
        position,
        metrics_color,
    );

    // Draw handle at descender position (matching click detection logic)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    let handle_position = position + Vec2::new(0.0, descender);

    // Normal handle: smaller and uses appropriate color based on selection state
    let normal_size = 16.0;

    // Determine handle color based on selection state
    let handle_color = if let (Some(entity), Some(selected_query)) =
        (sort_entity, selected_query)
    {
        // Check if this sort is selected
        if selected_query
            .iter()
            .any(|selected_entity| selected_entity == entity)
        {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
        } else {
            metrics_color // Default metrics color
        }
    } else {
        metrics_color // Default metrics color if we can't check selection
    };

    // DISABLED: Gizmo-based handle rendering - now using mesh system
    // match style {
    //     SortRenderStyle::TextBuffer => {
    //         // Square handle for text sorts
    //         let square_size = Vec2::new(normal_size * 2.0, normal_size * 2.0);
    //         gizmos.rect_2d(handle_position, square_size, handle_color);
    //
    //         // Add bigger square indicator for selected sorts
    //         if let (Some(entity), Some(selected_query)) =
    //             (sort_entity, selected_query)
    //         {
    //             if selected_query
    //                 .iter()
    //                 .any(|selected_entity| selected_entity == entity)
    //             {
    //                 let big_square_size =
    //                     Vec2::new(normal_size * 3.0, normal_size * 3.0);
    //                 gizmos.rect_2d(
    //                     handle_position,
    //                     big_square_size,
    //                     Color::srgba(1.0, 1.0, 0.0, 0.5),
    //                 );
    //             }
    //         }
    //     }
    //     SortRenderStyle::Freeform => {
    //         // Circle handle for freeform sorts
    //         gizmos.circle_2d(handle_position, normal_size, handle_color);
    //
    //         // Add bigger circle indicator for selected sorts
    //         if let (Some(entity), Some(selected_query)) =
    //             (sort_entity, selected_query)
    //         {
    //             if selected_query
    //                 .iter()
    //                 .any(|selected_entity| selected_entity == entity)
    //             {
    //                 gizmos.circle_2d(
    //                     handle_position,
    //                     normal_size * 1.5,
    //                     Color::srgba(1.0, 1.0, 0.0, 0.5),
    //                 );
    //             }
    //         }
    //     }
    // }
}

/// FontIR-compatible version of render_sort_visuals
pub fn render_fontir_sort_visuals(
    gizmos: &mut Gizmos,
    fontir_app_state: &FontIRAppState,
    glyph_name: &str,
    advance_width: f32,
    _metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
    style: SortRenderStyle,
) {
    // Get FontIR glyph paths (with working copy edits if available)
    if let Some(paths) = fontir_app_state.get_glyph_paths_with_edits(glyph_name)
    {
        // DISABLED: Draw FontIR outline - now using mesh system
        // draw_fontir_glyph_outline_at_position(gizmos, &paths, position);
    }

    // Get real FontIR metrics instead of using converted FontMetrics
    let fontir_metrics = fontir_app_state.get_font_metrics();

    // Draw FontIR metrics with real values
    crate::rendering::metrics::draw_fontir_metrics_at_position(
        gizmos,
        advance_width,
        &fontir_metrics,
        position,
        metrics_color,
    );

    // Draw handle at descender position using real FontIR descender
    let descender = fontir_metrics.descender.unwrap_or(-200.0);
    let handle_position = position + Vec2::new(0.0, descender);
    let normal_size = 16.0;

    // DISABLED: Gizmo-based handle rendering - now using mesh system
    // match style {
    //     SortRenderStyle::TextBuffer => {
    //         // Square handle for text sorts
    //         let square_size = Vec2::new(normal_size * 2.0, normal_size * 2.0);
    //         gizmos.rect_2d(handle_position, square_size, metrics_color);
    //     }
    //     SortRenderStyle::Freeform => {
    //         // Circle handle for freeform sorts
    //         gizmos.circle_2d(handle_position, normal_size, metrics_color);
    //     }
    // }
}

/// Render FontIR sort with optimized live rendering during nudging
pub fn render_fontir_sort_visuals_with_live_sync(
    gizmos: &mut Gizmos,
    fontir_app_state: &FontIRAppState,
    glyph_name: &str,
    advance_width: f32,
    _metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
    style: SortRenderStyle,
    // Unused parameters for API compatibility
    _sort_entity: Option<Entity>,
    _sort_transform: Option<&Transform>,
    _point_query: Option<
        &Query<
            (Entity, &Transform, &GlyphPointReference, &PointType),
            With<SortPointEntity>,
        >,
    >,
    _selected_query: Option<&Query<Entity, With<Selected>>>,
    _nudge_state: Option<&NudgeState>,
) {
    // SIMPLIFIED: Always use stable FontIR working copy rendering
    // Working copy updates happen immediately during nudging for synchronization
    if let Some(paths) = fontir_app_state.get_glyph_paths_with_edits(glyph_name)
    {
        // DISABLED: Draw FontIR outline - now using mesh system
        // draw_fontir_glyph_outline_at_position(gizmos, &paths, position);
    }

    // Always draw metrics and handles the same way
    let fontir_metrics = fontir_app_state.get_font_metrics();

    crate::rendering::metrics::draw_fontir_metrics_at_position(
        gizmos,
        advance_width,
        &fontir_metrics,
        position,
        metrics_color,
    );

    let descender = fontir_metrics.descender.unwrap_or(-200.0);
    let handle_position = position + Vec2::new(0.0, descender);
    let normal_size = 16.0;

    // DISABLED: Gizmo-based handle rendering - now using mesh system
    // match style {
    //     SortRenderStyle::TextBuffer => {
    //         let square_size = Vec2::new(normal_size * 2.0, normal_size * 2.0);
    //         gizmos.rect_2d(handle_position, square_size, metrics_color);
    //     }
    //     SortRenderStyle::Freeform => {
    //         gizmos.circle_2d(handle_position, normal_size, metrics_color);
    //     }
    // }
}

// REMOVED: draw_simple_live_outline
// This function was causing curves to become straight lines during nudging.
// Now using direct FontIR working copy updates for immediate synchronization.

/// System to render mesh-based sort handles for all active sorts
pub fn render_mesh_sort_handles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handle_entities: ResMut<SortHandleEntities>,
    sort_query: Query<
        (Entity, &Transform, &crate::editing::sort::Sort),
        With<crate::editing::sort::ActiveSort>,
    >,
    existing_handles: Query<Entity, With<SortHandle>>,
    selected_query: Query<Entity, With<Selected>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    camera_scale: Res<CameraResponsiveScale>,
) {
    // Clear existing handles
    for entity in existing_handles.iter() {
        commands.entity(entity).despawn();
    }
    handle_entities.handles.clear();

    if let Some(fontir_state) = fontir_app_state {
        let fontir_metrics = fontir_state.get_font_metrics();

        for (sort_entity, sort_transform, sort) in sort_query.iter() {
            let position = sort_transform.translation.truncate();

            // Get advance width from FontIR
            let advance_width =
                fontir_state.get_glyph_advance_width(&sort.glyph_name);

            // Handle position at descender
            let descender = fontir_metrics.descender.unwrap_or(-200.0);
            let handle_position = position + Vec2::new(0.0, descender);

            // Determine handle color
            let is_selected = selected_query.get(sort_entity).is_ok();
            let handle_color = if is_selected {
                Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
            } else {
                crate::ui::theme::SORT_INACTIVE_METRICS_COLOR // Default metrics color
            };

            let normal_size = 16.0;

            // Create appropriate handle based on sort layout mode
            let style = match sort.layout_mode {
                SortLayoutMode::Text => SortRenderStyle::TextBuffer,
                SortLayoutMode::Freeform => SortRenderStyle::Freeform,
            };

            let handle_entity = match style {
                SortRenderStyle::TextBuffer => {
                    spawn_square_handle(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        handle_position,
                        normal_size, // Pass normal_size so total square size = 2 * normal_size (matching original)
                        handle_color,
                        sort_entity,
                        SortHandleType::Square,
                        &camera_scale,
                    )
                }
                SortRenderStyle::Freeform => spawn_circle_handle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    handle_position,
                    normal_size,
                    handle_color,
                    sort_entity,
                    SortHandleType::Circle,
                    &camera_scale,
                ),
            };

            // Store handle entity
            handle_entities
                .handles
                .insert(sort_entity, vec![handle_entity]);

            // Add selection indicator if selected
            if is_selected {
                let selection_entity = match style {
                    SortRenderStyle::TextBuffer => {
                        spawn_square_handle(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            handle_position,
                            normal_size * 1.5, // Pass 1.5x so total square size = 3 * normal_size (matching original)
                            Color::srgba(1.0, 1.0, 0.0, 0.5),
                            sort_entity,
                            SortHandleType::SelectionIndicator,
                            &camera_scale,
                        )
                    }
                    SortRenderStyle::Freeform => spawn_circle_handle(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        handle_position,
                        normal_size * 1.5,
                        Color::srgba(1.0, 1.0, 0.0, 0.5),
                        sort_entity,
                        SortHandleType::SelectionIndicator,
                        &camera_scale,
                    ),
                };

                handle_entities
                    .handles
                    .get_mut(&sort_entity)
                    .unwrap()
                    .push(selection_entity);
            }
        }
    }
}

/// Plugin for mesh-based sort handle rendering
pub struct SortHandleRenderingPlugin;

impl Plugin for SortHandleRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SortHandleEntities>()
            .add_systems(Update, render_mesh_sort_handles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_render_style_distinction() {
        // Test that TextBuffer and Freeform styles are distinct
        assert_ne!(SortRenderStyle::TextBuffer, SortRenderStyle::Freeform);

        // Test that each style is equal to itself
        assert_eq!(SortRenderStyle::TextBuffer, SortRenderStyle::TextBuffer);
        assert_eq!(SortRenderStyle::Freeform, SortRenderStyle::Freeform);
    }

    #[test]
    fn test_sort_render_style_debug() {
        // Test that styles can be debug printed
        assert_eq!(format!("{:?}", SortRenderStyle::TextBuffer), "TextBuffer");
        assert_eq!(format!("{:?}", SortRenderStyle::Freeform), "Freeform");
    }
}
