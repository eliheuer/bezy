use bevy::prelude::*;
use crate::editing::selection::components::{PointType, GlyphPointReference};
use crate::systems::sort_manager::SortPointEntity;
use crate::editing::sort::{Sort, ActiveSort};
use crate::core::state::{AppState, FontIRAppState};
use crate::ui::theme::HANDLE_LINE_COLOR;

// Module-level debug to verify loading
static INIT: std::sync::Once = std::sync::Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        println!("[HANDLES] outline_elements.rs module loaded");
    });
}


/// Component to mark handle line entities
#[derive(Component)]
pub struct HandleLine {
    pub start_entity: Entity,
    pub end_entity: Entity,
}

/// Plugin for rendering various outline elements (handles, tangents, etc.)
pub struct OutlineElementsPlugin;

impl Plugin for OutlineElementsPlugin {
    fn build(&self, app: &mut App) {
        ensure_init();
        println!("[HANDLES] OutlineElementsPlugin: Registering handle systems");
        app.add_systems(
            Update,
            (update_handle_lines, cleanup_orphaned_handles).chain(),
        );
        println!("[HANDLES] OutlineElementsPlugin: Systems registered successfully");
    }
}

/// Updates handle lines between on-curve and off-curve points
fn update_handle_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    active_sort_query: Query<&Sort, With<ActiveSort>>,
    point_query: Query<
        (
            Entity,
            &Transform,
            &GlyphPointReference,
            &PointType,
        ),
        With<SortPointEntity>,
    >,
    existing_handles: Query<(Entity, &HandleLine)>,
) {
    // Add a periodic debug message to confirm system is running
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let point_count = point_query.iter().count();
    println!("[HANDLES] System called - update #{}, found {} points", count, point_count);
    
    // Early exit if no points
    if point_count == 0 {
        return;
    }
    
    // First clear existing handles
    for (entity, _) in existing_handles.iter() {
        commands.entity(entity).despawn();
    }
    
    // Check if we have any points to work with
    if point_query.is_empty() {
        return;
    }
    
    // Get glyph name from the first point (they should all be from the same glyph)
    let first_point = point_query.iter().next().unwrap();
    let glyph_name = &first_point.2.glyph_name;
    
    // Only log if we're creating handles
    if count % 120 == 0 {
        println!("[HANDLES] Processing glyph: {} with {} points", glyph_name, point_query.iter().count());
    }
    
    // Try FontIRAppState first, fall back to AppState
    if let Some(fontir_state) = fontir_app_state {
        // Use FontIR paths - this matches how points are actually created
        if let Some(paths) = fontir_state.get_glyph_paths(glyph_name) {
            if count % 120 == 0 {
                println!("[HANDLES] Found FontIR paths: {} contours", paths.len());
            }
            create_handles_from_fontir_paths(&mut commands, &mut meshes, &mut materials, &paths, &point_query, glyph_name);
        } else {
            println!("[HANDLES] No FontIR glyph data found for: {}", glyph_name);
        }
        return;
    } else if let Some(app_state) = app_state {
        // Fallback to UFO data (probably not used)
        let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) else {
            println!("[HANDLES] No UFO glyph data found for: {}", glyph_name);
            return;
        };
        
        let Some(outline) = &glyph_data.outline else {
            println!("[HANDLES] No UFO outline data found for: {}", glyph_name);
            return;
        };
        
        if count % 120 == 0 {
            println!("[HANDLES] Found UFO outline with {} contours", outline.contours.len());
        }
        // Continue with existing UFO logic (create_handles_from_ufo_data)
        create_handles_from_ufo_data(&mut commands, &mut meshes, &mut materials, &outline, &point_query, glyph_name);
    } else {
        println!("[HANDLES] No AppState or FontIRAppState available");
        return;
    };
}

/// Create handles from UFO outline data (fallback)
fn create_handles_from_ufo_data(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline: &crate::core::state::OutlineData,
    point_query: &Query<
        (
            Entity,
            &Transform,
            &GlyphPointReference,
            &PointType,
        ),
        With<SortPointEntity>,
    >,
    glyph_name: &str,
) {
    // Group points by contour
    let mut contour_points: Vec<Vec<(Entity, Vec2, bool, usize)>> =
        vec![Vec::new(); outline.contours.len()];
    
    let mut total_points = 0;
    for (entity, transform, glyph_ref, point_type) in point_query.iter() {
        if &glyph_ref.glyph_name == glyph_name {
            let position = transform.translation.truncate();
            let is_on_curve = point_type.is_on_curve;
            let contour_index = glyph_ref.contour_index;
            let point_index = glyph_ref.point_index;
            
            total_points += 1;
            
            if contour_index < contour_points.len() {
                contour_points[contour_index].push((
                    entity,
                    position,
                    is_on_curve,
                    point_index,
                ));
            }
        }
    }
    
    println!("[HANDLES] Found {} total points for glyph: {}", total_points, glyph_name);
    
    // Sort points within each contour by their original index
    for contour_points in &mut contour_points {
        contour_points.sort_by_key(|(_, _, _, index)| *index);
    }
    
    // Debug: Print the point sequence for each contour
    for (contour_idx, contour_points) in contour_points.iter().enumerate() {
        let point_types: Vec<String> = contour_points.iter()
            .map(|(_, _, is_on_curve, idx)| format!("{}:{}", 
                idx, if *is_on_curve { "ON" } else { "OFF" }))
            .collect();
        println!("[HANDLES] UFO Contour {}: [{}]", contour_idx, point_types.join(", "));
    }
    
    // Create handle lines for each contour
    let handle_material = materials.add(ColorMaterial::from(HANDLE_LINE_COLOR));
    let mut handles_created = 0;
    
    for (contour_idx, contour_points) in contour_points.iter().enumerate() {
        println!("[HANDLES] UFO Contour {}: {} points", contour_idx, contour_points.len());
        if contour_points.len() < 2 {
            continue;
        }
        
        let created = create_contour_handle_lines(
            commands,
            meshes,
            &handle_material,
            &contour_points,
        );
        handles_created += created;
    }
    
    println!("[HANDLES] Created {} handle lines total", handles_created);
}

/// Creates handle line entities for a contour
fn create_contour_handle_lines(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    handle_material: &Handle<ColorMaterial>,
    contour_points: &[(Entity, Vec2, bool, usize)],
) -> usize {
    let len = contour_points.len();
    let mut created_count = 0;
    
    for i in 0..len {
        let (current_entity, current_pos, current_on_curve, _) = contour_points[i];
        
        // Check next point (wrapping around)
        let next_i = (i + 1) % len;
        let (next_entity, next_pos, next_on_curve, _) = contour_points[next_i];
        
        // Create handle line if one point is on-curve and the other is off-curve
        if current_on_curve != next_on_curve {
            let direction = next_pos - current_pos;
            let length = direction.length();
            let angle = direction.y.atan2(direction.x);
            let midpoint = (current_pos + next_pos) / 2.0;
            
            println!("[HANDLES] Creating handle line from {:?} to {:?}, length: {}", 
                     current_pos, next_pos, length);
            
            // Create a thin rectangle mesh for the line
            let line_thickness = 2.0; // Make slightly thicker for visibility
            let line_mesh = meshes.add(Rectangle::new(length, line_thickness));
            
            commands.spawn((
                Mesh2d(line_mesh),
                MeshMaterial2d(handle_material.clone()),
                Transform::from_translation(midpoint.extend(10.0)) // Z = 10 (between background and points)
                    .with_rotation(Quat::from_rotation_z(angle)),
                HandleLine {
                    start_entity: current_entity,
                    end_entity: next_entity,
                },
            ));
            
            created_count += 1;
        } else {
            // Debug: why no handle created
            println!("[HANDLES] No handle between points {} ({}) and {} ({})", 
                     i, if current_on_curve { "ON" } else { "OFF" },
                     next_i, if next_on_curve { "ON" } else { "OFF" });
        }
    }
    
    created_count
}

/// Create handles from FontIR BezPath data (matches actual point creation)
fn create_handles_from_fontir_paths(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    paths: &[kurbo::BezPath],
    point_query: &Query<
        (
            Entity,
            &Transform,
            &GlyphPointReference,
            &PointType,
        ),
        With<SortPointEntity>,
    >,
    glyph_name: &str,
) {
    println!("[HANDLES] Creating handles from FontIR paths for glyph: {}", glyph_name);
    
    // Create material for handles
    let handle_material = materials.add(ColorMaterial::from(HANDLE_LINE_COLOR));
    let mut handles_created = 0;
    
    // Group existing point entities by contour and index for mapping
    let mut point_entities: std::collections::HashMap<(usize, usize), Entity> = std::collections::HashMap::new();
    let mut point_positions: std::collections::HashMap<Entity, Vec2> = std::collections::HashMap::new();
    
    for (entity, transform, glyph_ref, _) in point_query.iter() {
        if glyph_ref.glyph_name == glyph_name {
            let position = transform.translation.truncate();
            point_entities.insert((glyph_ref.contour_index, glyph_ref.point_index), entity);
            point_positions.insert(entity, position);
        }
    }
    
    println!("[HANDLES] Found {} point entities for glyph {}", point_entities.len(), glyph_name);
    
    // Create handles based on point connectivity (adjacent on/off-curve points)
    // Use the same logic as UFO system - connect consecutive points where one is on-curve and other is off-curve
    for (entity, transform, glyph_ref, point_type) in point_query.iter() {
        if glyph_ref.glyph_name != glyph_name {
            continue;
        }
        
        let current_pos = transform.translation.truncate();
        let current_is_on_curve = point_type.is_on_curve;
        
        // Find the next point in the same contour
        let next_index = glyph_ref.point_index + 1;
        if let Some(next_entity) = point_entities.get(&(glyph_ref.contour_index, next_index)) {
            if let Some(next_pos) = point_positions.get(next_entity) {
                // Get the next point's type
                if let Ok((_, _, _, next_point_type)) = point_query.get(*next_entity) {
                    let next_is_on_curve = next_point_type.is_on_curve;
                    
                    // Create handle if one is on-curve and the other is off-curve
                    if current_is_on_curve != next_is_on_curve {
                        let direction = *next_pos - current_pos;
                        let length = direction.length();
                        let angle = direction.y.atan2(direction.x);
                        let midpoint = (current_pos + *next_pos) / 2.0;
                        
                        println!("[HANDLES] Creating FontIR handle line from {:?} to {:?}", current_pos, next_pos);
                        
                        // Create a thin rectangle mesh for the line
                        let line_thickness = 2.0;
                        let line_mesh = meshes.add(Rectangle::new(length, line_thickness));
                        
                        commands.spawn((
                            Mesh2d(line_mesh),
                            MeshMaterial2d(handle_material.clone()),
                            Transform::from_translation(midpoint.extend(10.0))
                                .with_rotation(Quat::from_rotation_z(angle)),
                            HandleLine {
                                start_entity: entity,
                                end_entity: *next_entity,
                            },
                        ));
                        
                        handles_created += 1;
                    }
                }
            }
        }
    }
    
    println!("[HANDLES] Created {} handles from FontIR paths", handles_created);
}

/// Cleans up handle lines when their connected points are removed
fn cleanup_orphaned_handles(
    mut commands: Commands,
    handles: Query<(Entity, &HandleLine)>,
    points: Query<Entity, With<SortPointEntity>>,
) {
    for (handle_entity, handle_line) in handles.iter() {
        // Check if both connected points still exist
        if points.get(handle_line.start_entity).is_err() 
            || points.get(handle_line.end_entity).is_err() {
            commands.entity(handle_entity).despawn();
        }
    }
}