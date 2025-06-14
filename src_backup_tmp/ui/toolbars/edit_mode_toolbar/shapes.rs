//! Primitives Tool - Comprehensive shape drawing with submenu
//!
//! This file demonstrates a complete submenu pattern that can be reused for other tools.
//! 
//! ## Architecture
//! 
//! 1. **Main Tool Implementation** - EditTool trait with submenu support
//! 2. **Submenu System** - Dynamic UI that appears below main toolbar  
//! 3. **Tool Variants** - Different shape types (Rectangle, Ellipse, etc.)
//! 4. **Drawing Logic** - Mouse handling and shape creation
//! 5. **UI Controls** - Tool-specific settings (e.g., corner radius)
//! 6. **Plugin System** - Registration and system setup
//!
//! ## Submenu Pattern (Reusable for other tools)
//!
//! ```rust
//! // 1. Define your tool variants
//! #[derive(Debug, Clone, Copy, PartialEq, Resource)]
//! pub enum MyToolType { Variant1, Variant2, Variant3 }
//!
//! // 2. Create submenu components
//! #[derive(Component)] pub struct MyToolSubMenuButton;
//! #[derive(Component)] pub struct MyToolTypeButton(pub MyToolType);
//!
//! // 3. Implement submenu spawn function
//! pub fn spawn_my_tool_submenu() { /* spawn buttons for each variant */ }
//!
//! // 4. Handle submenu interactions
//! pub fn handle_my_tool_selection() { /* switch between variants */ }
//! 
//! // 5. Toggle submenu visibility
//! pub fn toggle_my_tool_submenu_visibility() { /* show/hide based on active tool */ }
//! ```

use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use crate::ui::theme::*;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry, CurrentTool, ToolId};
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
// Removed unused imports
use norad::{Contour, ContourPoint, PointType};

// ==================== MAIN TOOL IMPLEMENTATION ====================

/// Main shapes tool - handles shape drawing with submenu support
pub struct ShapesTool;

impl EditTool for ShapesTool {
    fn id(&self) -> ToolId {
        "shapes"
    }
    
    fn name(&self) -> &'static str {
        "Shapes"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E016}" // Shapes icon
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('r')
    }
    
    fn default_order(&self) -> i32 {
        30 // After pen, before text
    }
    
    fn description(&self) -> &'static str {
        "Draw shapes (rectangles, ellipses, rounded rectangles)"
    }
    
    fn update(&self, commands: &mut Commands) {
        // Disable selection mode while in shapes mode
        commands.insert_resource(
            crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive(false),
        );
    }
    
    fn on_enter(&self) {
        info!("Entered Shapes tool");
    }
    
    fn on_exit(&self) {
        info!("Exited Shapes tool");
    }
}

// ==================== SUBMENU SYSTEM ====================

/// Types of primitive shapes available
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum PrimitiveType {
    #[default]
    Rectangle,
    Ellipse,
    RoundedRectangle,
}

impl PrimitiveType {
    /// Get the icon for each primitive type
    pub fn get_icon(&self) -> &'static str {
        match self {
            PrimitiveType::Rectangle => "\u{E018}",
            PrimitiveType::Ellipse => "\u{E019}",
            PrimitiveType::RoundedRectangle => "\u{E020}",
        }
    }

    /// Get the display name for each primitive type
    pub fn display_name(&self) -> &'static str {
        match self {
            PrimitiveType::Rectangle => "Rectangle",
            PrimitiveType::Ellipse => "Ellipse",
            PrimitiveType::RoundedRectangle => "Rounded Rectangle",
        }
    }
}

/// Component to mark primitive sub-menu buttons
#[derive(Component)]
pub struct PrimitiveSubMenuButton;

/// Component to associate a button with its primitive type
#[derive(Component)]
pub struct PrimitiveTypeButton(pub PrimitiveType);

/// Resource to track the currently selected primitive type
#[derive(Resource, Default)]
pub struct CurrentPrimitiveType(pub PrimitiveType);

// ==================== DRAWING STATE ====================

/// Active drawing state for primitives
#[derive(Resource)]
pub struct ActivePrimitiveDrawing {
    pub is_drawing: bool,
    pub tool_type: PrimitiveType,
    pub start_position: Option<Vec2>,
    pub current_position: Option<Vec2>,
}

impl Default for ActivePrimitiveDrawing {
    fn default() -> Self {
        Self {
            is_drawing: false,
            tool_type: PrimitiveType::Rectangle,
            start_position: None,
            current_position: None,
        }
    }
}

impl ActivePrimitiveDrawing {
    /// Get the rectangle from the current drawing state
    pub fn get_rect(&self) -> Option<Rect> {
        if let (Some(start), Some(current)) = (self.start_position, self.current_position) {
            let min_x = start.x.min(current.x);
            let min_y = start.y.min(current.y);
            let max_x = start.x.max(current.x);
            let max_y = start.y.max(current.y);

            Some(Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            })
        } else {
            None
        }
    }
}

// ==================== UI CONTROLS ====================

/// Component for the panel containing rounded rectangle specific settings
#[derive(Component)]
pub struct RoundedRectSettingsPanel;

/// Component for the corner radius input field
#[derive(Component)]
pub struct CornerRadiusInput;

/// Component to mark the text that displays the current radius value
#[derive(Component)]
pub struct RadiusValueText;

/// Resource to store the current corner radius
#[derive(Resource)]
pub struct CurrentCornerRadius(pub i32);

/// Resource to track when UI elements are being interacted with
#[derive(Resource, Default)]
pub struct UiInteractionState {
    pub is_interacting_with_ui: bool,
}

/// Local state for the corner radius input
#[derive(Default)]
pub struct CornerRadiusInputState {
    pub text: String,
    pub focused: bool,
}

impl Default for CurrentCornerRadius {
    fn default() -> Self {
        Self(32) // Default corner radius
    }
}

// ==================== PLUGIN ====================

/// Plugin for the Shapes tool
pub struct ShapesToolPlugin;

impl Plugin for ShapesToolPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentPrimitiveType>()
            .init_resource::<ActivePrimitiveDrawing>()
            .init_resource::<CurrentCornerRadius>()
            .init_resource::<UiInteractionState>()
            .add_systems(Startup, register_shapes_tool);
    }
}

fn register_shapes_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(ShapesTool));
}

// ==================== SUBMENU FUNCTIONS ====================

/// System to spawn the shapes sub-menu
/// 
/// This demonstrates the submenu pattern that can be reused for other tools.
/// The submenu appears below the main toolbar when the tool is active.
pub fn spawn_shapes_submenu(
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    // Create a submenu container that sits below the main toolbar
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                // Calculate position based on toolbar size, margin, and spacing
                // Main toolbar is positioned at TOOLBAR_MARGIN and has height of 64px
                // Add TOOLBAR_ITEM_SPACING to maintain the same spacing as the horizontal buttons
                top: Val::Px(TOOLBAR_MARGIN + 64.0 + TOOLBAR_ITEM_SPACING + 4.0),
                left: Val::Px(TOOLBAR_MARGIN), // Use TOOLBAR_MARGIN for consistent positioning
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(TOOLBAR_PADDING)), // Use theme padding
                margin: UiRect::all(Val::ZERO), // Set to ZERO since we're using absolute positioning
                row_gap: Val::Px(TOOLBAR_ROW_GAP), // Use theme row gap
                ..default()
            },
            Name::new("ShapesSubMenu"),
            // Start as hidden until shapes mode is selected
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            let primitive_types = [
                PrimitiveType::Rectangle,
                PrimitiveType::Ellipse,
                PrimitiveType::RoundedRectangle,
            ];

            for primitive_type in primitive_types.iter() {
                spawn_primitive_button(parent, primitive_type, asset_server);
            }
        });
}

/// Helper function to spawn a single primitive type button
/// 
/// This pattern can be reused for other submenu buttons
fn spawn_primitive_button(
    parent: &mut ChildBuilder,
    primitive_type: &PrimitiveType,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            margin: UiRect::all(Val::Px(TOOLBAR_ITEM_SPACING)), // Use theme spacing
            ..default()
        })
        .with_children(|button_container| {
            button_container
                .spawn((
                    Button,
                    PrimitiveSubMenuButton,
                    PrimitiveTypeButton(*primitive_type),
                    Node {
                        width: Val::Px(64.0),
                        height: Val::Px(64.0),
                        padding: UiRect::all(Val::Px(TOOLBAR_PADDING)), // Use theme padding
                        border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)), // Use theme border width
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(TOOLBAR_BORDER_COLOR),
                    BorderRadius::all(Val::Px(TOOLBAR_BORDER_RADIUS)),
                    BackgroundColor(TOOLBAR_BACKGROUND_COLOR),
                ))
                .with_children(|button| {
                    // Add the icon using the primitive type's icon
                    button.spawn((
                        Text::new(primitive_type.get_icon().to_string()),
                        TextFont {
                            font: asset_server.load(DEFAULT_FONT_PATH),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(TOOLBAR_ICON_COLOR),
                    ));
                });
        });
}

/// Handle primitive type selection from the submenu
/// 
/// This demonstrates the submenu interaction pattern
pub fn handle_primitive_selection(
    mut button_queries: ParamSet<(
        // Query for buttons with changed interaction
        Query<
            (&Interaction, &PrimitiveTypeButton),
            (Changed<Interaction>, With<PrimitiveSubMenuButton>),
        >,
        // Query for all buttons to update their appearance
        Query<
            (
                &Interaction,
                &mut BackgroundColor,
                &mut BorderColor,
                &PrimitiveTypeButton,
                Entity,
            ),
            With<PrimitiveSubMenuButton>,
        >,
    )>,
    mut text_query: Query<(&Parent, &mut TextColor)>,
    mut current_type: ResMut<CurrentPrimitiveType>,
) {
    // Handle button clicks
    for (interaction, primitive_button) in button_queries.p0().iter() {
        if *interaction == Interaction::Pressed {
            let new_type = primitive_button.0;
            if current_type.0 != new_type {
                current_type.0 = new_type;
                info!("Switched to primitive type: {:?}", new_type);
            }
        }
    }

    // Update button appearances
    for (interaction, mut color, mut border_color, primitive_button, entity) in button_queries.p1().iter_mut() {
        let is_current = current_type.0 == primitive_button.0;

        // Update button colors
        match (*interaction, is_current) {
            (Interaction::Pressed, _) | (_, true) => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
            }
            (Interaction::Hovered, false) => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            }
            (Interaction::None, false) => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
            }
        }

        // Update text color for this button
        for (parent, mut text_color) in &mut text_query {
            if parent.get() == entity {
                text_color.0 = if is_current {
                    PRESSED_BUTTON_ICON_COLOR
                } else {
                    TOOLBAR_ICON_COLOR
                };
            }
        }
    }
}

/// Toggle submenu visibility based on current tool
/// 
/// This pattern can be reused for other tool submenus
pub fn toggle_shapes_submenu_visibility(
    current_tool: Res<CurrentTool>,
    mut submenu_query: Query<(&mut Visibility, &Name)>,
) {
    let is_shapes_active = current_tool.get_current() == Some("shapes");
    
    for (mut visibility, name) in submenu_query.iter_mut() {
        if name.as_str() == "ShapesSubMenu" {
            *visibility = if is_shapes_active {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

// ==================== DRAWING LOGIC ====================

/// Handle mouse events for shape drawing
pub fn handle_primitive_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    current_primitive_type: Res<CurrentPrimitiveType>,
    current_tool: Res<CurrentTool>,
    mut active_drawing: ResMut<ActivePrimitiveDrawing>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut app_state_changed: EventWriter<crate::rendering::draw::AppStateChanged>,
    mut app_state: ResMut<crate::core::state::AppState>,
    glyph_navigation: Res<crate::core::state::GlyphNavigation>,
    corner_radius: Res<CurrentCornerRadius>,
    ui_state: Res<UiInteractionState>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    active_sort_query: Query<&crate::editing::sort::Sort, With<crate::editing::sort::ActiveSort>>,
) {
    // Only handle events when in shapes mode
    if current_tool.get_current() != Some("shapes") {
        return;
    }

    // Don't process drawing events when hovering over or interacting with UI
    if ui_state.is_interacting_with_ui || ui_hover_state.is_hovering_ui {
        return;
    }

    // Get the primary camera
    let camera_entity = camera_q.iter().find(|(camera, _)| camera.is_active);
    let Some((camera, camera_transform)) = camera_entity else {
        return;
    };

    // Get the main window
    let Ok(window) = windows.get_single() else {
        return;
    };

    // Handle cursor movement
    for _cursor_moved in cursor_moved_events.read() {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                // Apply snap to grid if enabled
                let snapped_position = if SNAP_TO_GRID_ENABLED {
                    Vec2::new(
                        (world_position.x / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                        (world_position.y / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                    )
                } else {
                    world_position
                };

                active_drawing.current_position = Some(snapped_position);
            }
        }
    }

    // Handle mouse button input
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = active_drawing.current_position {
            // Start a new drawing
            active_drawing.is_drawing = true;
            active_drawing.tool_type = current_primitive_type.0;
            active_drawing.start_position = Some(cursor_pos);
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        if active_drawing.is_drawing {
            println!("Mouse released - attempting to create shape");
            // Finish the drawing and create the shape
            if let Some(rect) = active_drawing.get_rect() {
                println!("Creating primitive shape with rect: {:?}", rect);
                
                // Get active sort if present (same pattern as pen tool)
                let active_sort = active_sort_query.get_single().ok();
                
                create_primitive_shape(
                    rect,
                    active_drawing.tool_type,
                    corner_radius.0,
                    &glyph_navigation,
                    &mut app_state,
                    &mut app_state_changed,
                    active_sort,
                );
            } else {
                println!("No rect available for shape creation");
            }

            // Reset drawing state
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
        }
    }
}

/// Render the active shape being drawn
pub fn render_active_primitive_drawing(
    mut gizmos: Gizmos,
    active_drawing: Res<ActivePrimitiveDrawing>,
    current_tool: Res<CurrentTool>,
) {
    // Only render when in shapes mode and actively drawing
    if current_tool.get_current() != Some("shapes") || !active_drawing.is_drawing {
        return;
    }

    if let Some(rect) = active_drawing.get_rect() {
        let color = Color::linear_rgba(0.0, 0.8, 1.0, 0.8); // Cyan preview
        
        match active_drawing.tool_type {
            PrimitiveType::Rectangle => {
                draw_dashed_rectangle(&mut gizmos, rect, color);
            }
            PrimitiveType::Ellipse => {
                draw_dashed_ellipse(&mut gizmos, rect, color);
            }
            PrimitiveType::RoundedRectangle => {
                draw_dashed_rounded_rectangle(&mut gizmos, rect, 32.0, color);
            }
        }
    }
}

/// Create a shape and add it to the current glyph
fn create_primitive_shape(
    rect: Rect,
    primitive_type: PrimitiveType,
    corner_radius: i32,
    glyph_navigation: &crate::core::state::GlyphNavigation,
    app_state: &mut crate::core::state::AppState,
    app_state_changed: &mut EventWriter<crate::rendering::draw::AppStateChanged>,
    active_sort: Option<&crate::editing::sort::Sort>,
) {
    println!("create_primitive_shape called with type: {:?}", primitive_type);
    
    // When there's an active sort, use the sort's glyph; otherwise use glyph navigation (same as pen tool)
    let glyph_name = if let Some(sort) = active_sort {
        println!("SHAPES TOOL: Using active sort glyph: {}", sort.glyph_name);
        sort.glyph_name.clone()
    } else {
        let Some(glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo) else {
            println!("ERROR: No glyph found for shape drawing and no active sort");
            return;
        };
        println!("SHAPES TOOL: Using glyph navigation glyph: {}", glyph_name);
        glyph_name
    };

    if let Some(layer) = app_state.workspace.font.ufo.get_default_layer_mut() {
        println!("Got default layer");
        if let Some(glyph) = layer.get_glyph_mut(&glyph_name) {
            println!("Got glyph, creating contour...");
            let contour = match primitive_type {
                PrimitiveType::Rectangle => create_rectangle_contour(rect),
                PrimitiveType::Ellipse => create_ellipse_contour(rect),
                PrimitiveType::RoundedRectangle => create_rounded_rectangle_contour(rect, corner_radius as f32),
            };

            if let Some(contour) = contour {
                println!("Contour created with {} points", contour.points.len());
                
                // Get or create the outline (same pattern as pen tool)
                let outline = glyph.outline.get_or_insert_with(|| norad::glyph::Outline {
                    contours: Vec::new(),
                    components: Vec::new(),
                });
                
                // Add the contour to the outline
                outline.contours.push(contour);
                app_state_changed.send(crate::rendering::draw::AppStateChanged);
                
                let source = if active_sort.is_some() { "active sort" } else { "glyph navigation" };
                println!("SHAPES TOOL: Successfully created {:?} shape in glyph {} (from {}). Total contours now: {}", 
                        primitive_type, glyph_name, source, outline.contours.len());
            } else {
                println!("ERROR: Failed to create contour");
            }
        } else {
            println!("ERROR: Failed to get glyph: {}", glyph_name);
        }
    } else {
        println!("ERROR: Failed to get default layer");
    }
}

/// Create a rectangle contour
fn create_rectangle_contour(rect: Rect) -> Option<Contour> {
    let mut contour = Contour::new(Vec::new(), None, None);
    
    // Add points clockwise from bottom-left
    contour.points.push(create_point(rect.min.x, rect.min.y, PointType::Line, false));
    contour.points.push(create_point(rect.max.x, rect.min.y, PointType::Line, false));
    contour.points.push(create_point(rect.max.x, rect.max.y, PointType::Line, false));
    contour.points.push(create_point(rect.min.x, rect.max.y, PointType::Line, false));
    
    Some(contour)
}

/// Create an ellipse contour
fn create_ellipse_contour(rect: Rect) -> Option<Contour> {
    let mut contour = Contour::new(Vec::new(), None, None);
    
    let center_x = (rect.min.x + rect.max.x) / 2.0;
    let center_y = (rect.min.y + rect.max.y) / 2.0;
    let radius_x = (rect.max.x - rect.min.x) / 2.0;
    let radius_y = (rect.max.y - rect.min.y) / 2.0;
    
    // Ellipse approximation with Bezier curves (4 segments)
    let kappa = 0.5522847498; // Magic number for circle approximation
    let kappa_x = kappa * radius_x;
    let kappa_y = kappa * radius_y;
    
    // Bottom point
    contour.points.push(create_point(center_x, rect.min.y, PointType::Curve, true));
    contour.points.push(create_point(center_x + kappa_x, rect.min.y, PointType::OffCurve, false));
    contour.points.push(create_point(rect.max.x, center_y - kappa_y, PointType::OffCurve, false));
    
    // Right point
    contour.points.push(create_point(rect.max.x, center_y, PointType::Curve, true));
    contour.points.push(create_point(rect.max.x, center_y + kappa_y, PointType::OffCurve, false));
    contour.points.push(create_point(center_x + kappa_x, rect.max.y, PointType::OffCurve, false));
    
    // Top point
    contour.points.push(create_point(center_x, rect.max.y, PointType::Curve, true));
    contour.points.push(create_point(center_x - kappa_x, rect.max.y, PointType::OffCurve, false));
    contour.points.push(create_point(rect.min.x, center_y + kappa_y, PointType::OffCurve, false));
    
    // Left point
    contour.points.push(create_point(rect.min.x, center_y, PointType::Curve, true));
    contour.points.push(create_point(rect.min.x, center_y - kappa_y, PointType::OffCurve, false));
    contour.points.push(create_point(center_x - kappa_x, rect.min.y, PointType::OffCurve, false));
    
    Some(contour)
}

/// Create a rounded rectangle contour
fn create_rounded_rectangle_contour(rect: Rect, radius: f32) -> Option<Contour> {
    let mut contour = Contour::new(Vec::new(), None, None);
    
    let r = radius.min((rect.max.x - rect.min.x) / 2.0).min((rect.max.y - rect.min.y) / 2.0);
    let kappa = 0.5522847498 * r; // Magic number for circle approximation
    
    // Start from bottom edge (moving clockwise)
    contour.points.push(create_point(rect.min.x + r, rect.min.y, PointType::Line, false));
    contour.points.push(create_point(rect.max.x - r, rect.min.y, PointType::Line, false));
    
    // Bottom-right corner
    contour.points.push(create_point(rect.max.x - r + kappa, rect.min.y, PointType::OffCurve, false));
    contour.points.push(create_point(rect.max.x, rect.min.y + r - kappa, PointType::OffCurve, false));
    contour.points.push(create_point(rect.max.x, rect.min.y + r, PointType::Curve, true));
    
    // Right edge
    contour.points.push(create_point(rect.max.x, rect.max.y - r, PointType::Line, false));
    
    // Top-right corner
    contour.points.push(create_point(rect.max.x, rect.max.y - r + kappa, PointType::OffCurve, false));
    contour.points.push(create_point(rect.max.x - r + kappa, rect.max.y, PointType::OffCurve, false));
    contour.points.push(create_point(rect.max.x - r, rect.max.y, PointType::Curve, true));
    
    // Top edge
    contour.points.push(create_point(rect.min.x + r, rect.max.y, PointType::Line, false));
    
    // Top-left corner
    contour.points.push(create_point(rect.min.x + r - kappa, rect.max.y, PointType::OffCurve, false));
    contour.points.push(create_point(rect.min.x, rect.max.y - r + kappa, PointType::OffCurve, false));
    contour.points.push(create_point(rect.min.x, rect.max.y - r, PointType::Curve, true));
    
    // Left edge
    contour.points.push(create_point(rect.min.x, rect.min.y + r, PointType::Line, false));
    
    // Bottom-left corner
    contour.points.push(create_point(rect.min.x, rect.min.y + r - kappa, PointType::OffCurve, false));
    contour.points.push(create_point(rect.min.x + r - kappa, rect.min.y, PointType::OffCurve, false));
    
    Some(contour)
}

/// Helper function to create a contour point
fn create_point(x: f32, y: f32, typ: PointType, smooth: bool) -> ContourPoint {
    ContourPoint::new(x, y, typ, smooth, None, None, None)
}

// ==================== DRAWING HELPERS ====================

/// Draw a dashed rectangle for preview
fn draw_dashed_rectangle(gizmos: &mut Gizmos, rect: Rect, color: Color) {
    let dash_length = 10.0;
    let gap_length = 5.0;
    
    // Top edge
    draw_dashed_line(gizmos, Vec2::new(rect.min.x, rect.max.y), Vec2::new(rect.max.x, rect.max.y), dash_length, gap_length, color);
    // Right edge  
    draw_dashed_line(gizmos, Vec2::new(rect.max.x, rect.max.y), Vec2::new(rect.max.x, rect.min.y), dash_length, gap_length, color);
    // Bottom edge
    draw_dashed_line(gizmos, Vec2::new(rect.max.x, rect.min.y), Vec2::new(rect.min.x, rect.min.y), dash_length, gap_length, color);
    // Left edge
    draw_dashed_line(gizmos, Vec2::new(rect.min.x, rect.min.y), Vec2::new(rect.min.x, rect.max.y), dash_length, gap_length, color);
}

/// Draw a dashed ellipse for preview
fn draw_dashed_ellipse(gizmos: &mut Gizmos, rect: Rect, color: Color) {
    let center = Vec2::new((rect.min.x + rect.max.x) / 2.0, (rect.min.y + rect.max.y) / 2.0);
    let radius_x = (rect.max.x - rect.min.x) / 2.0;
    let radius_y = (rect.max.y - rect.min.y) / 2.0;
    
    // Draw ellipse as series of dashed lines
    let segments = 32;
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        
        let point1 = Vec2::new(
            center.x + radius_x * angle1.cos(),
            center.y + radius_y * angle1.sin(),
        );
        let point2 = Vec2::new(
            center.x + radius_x * angle2.cos(),
            center.y + radius_y * angle2.sin(),
        );
        
        gizmos.line_2d(point1, point2, color);
    }
}

/// Draw a dashed rounded rectangle for preview
fn draw_dashed_rounded_rectangle(gizmos: &mut Gizmos, rect: Rect, radius: f32, color: Color) {
    let r = radius.min((rect.max.x - rect.min.x) / 2.0).min((rect.max.y - rect.min.y) / 2.0);
    let dash_length = 10.0;
    let gap_length = 5.0;
    
    // Top edge
    draw_dashed_line(gizmos, Vec2::new(rect.min.x + r, rect.max.y), Vec2::new(rect.max.x - r, rect.max.y), dash_length, gap_length, color);
    // Right edge
    draw_dashed_line(gizmos, Vec2::new(rect.max.x, rect.max.y - r), Vec2::new(rect.max.x, rect.min.y + r), dash_length, gap_length, color);
    // Bottom edge
    draw_dashed_line(gizmos, Vec2::new(rect.max.x - r, rect.min.y), Vec2::new(rect.min.x + r, rect.min.y), dash_length, gap_length, color);
    // Left edge
    draw_dashed_line(gizmos, Vec2::new(rect.min.x, rect.min.y + r), Vec2::new(rect.min.x, rect.max.y - r), dash_length, gap_length, color);
    
    // Draw rounded corners
    draw_dashed_corner(gizmos, Vec2::new(rect.max.x - r, rect.max.y - r), r, 0.0, 90.0, color);
    draw_dashed_corner(gizmos, Vec2::new(rect.min.x + r, rect.max.y - r), r, 90.0, 180.0, color);
    draw_dashed_corner(gizmos, Vec2::new(rect.min.x + r, rect.min.y + r), r, 180.0, 270.0, color);
    draw_dashed_corner(gizmos, Vec2::new(rect.max.x - r, rect.min.y + r), r, 270.0, 360.0, color);
}

/// Draw a dashed quarter circle for rounded corners
fn draw_dashed_corner(gizmos: &mut Gizmos, center: Vec2, radius: f32, start_angle_deg: f32, end_angle_deg: f32, color: Color) {
    let segments = 8;
    let start_rad = start_angle_deg.to_radians();
    let end_rad = end_angle_deg.to_radians();
    let angle_step = (end_rad - start_rad) / segments as f32;
    
    for i in 0..segments {
        let angle1 = start_rad + i as f32 * angle_step;
        let angle2 = start_rad + (i + 1) as f32 * angle_step;
        
        let point1 = Vec2::new(
            center.x + radius * angle1.cos(),
            center.y + radius * angle1.sin(),
        );
        let point2 = Vec2::new(
            center.x + radius * angle2.cos(),
            center.y + radius * angle2.sin(),
        );
        
        gizmos.line_2d(point1, point2, color);
    }
}

/// Draw a dashed line
fn draw_dashed_line(gizmos: &mut Gizmos, start: Vec2, end: Vec2, dash_length: f32, gap_length: f32, color: Color) {
    let direction = (end - start).normalize();
    let total_length = start.distance(end);
    let dash_gap_length = dash_length + gap_length;
    
    let mut current_pos = 0.0;
    while current_pos < total_length {
        let dash_start = start + direction * current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);
        let dash_end = start + direction * dash_end_pos;
        
        gizmos.line_2d(dash_start, dash_end, color);
        current_pos += dash_gap_length;
    }
} 