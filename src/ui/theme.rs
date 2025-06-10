use bevy::prelude::Color;
use bevy::prelude::*;

// Font Path
pub const DEFAULT_FONT_PATH: &str = "fonts/bezy-grotesk-regular.ttf";
pub const MONO_FONT_PATH: &str = "fonts/HasubiMono-Regular.ttf";

// Font Sizes
#[allow(dead_code)]
pub const WIDGET_TITLE_FONT_SIZE: f32 = 24.0;
pub const WIDGET_TEXT_FONT_SIZE: f32 = 24.0;

// Widget Visual Style Constants
pub const WIDGET_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);
pub const WIDGET_BORDER_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);
pub const WIDGET_BORDER_RADIUS: f32 = 0.0;
pub const WIDGET_BORDER_WIDTH: f32 = 2.0;
pub const WIDGET_PADDING: f32 = 16.0;
pub const WIDGET_MARGIN: f32 = 24.0;
pub const WIDGET_ROW_GAP: f32 = 0.0;

// Gizmo Configuration
// Change this value to customize the thickness of all visual drawing helpers:
// - 1.0 = thin, subtle lines (original Bevy default)
// - 2.0 = medium thickness
// - 3.0 = thick, bold lines (current default for better visibility)
// - 4.0+ = very thick lines
pub const GIZMO_LINE_WIDTH: f32 = 3.0;

// Toolbar Visual Style Constants
pub const TOOLBAR_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);
pub const TOOLBAR_ICON_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);
pub const TOOLBAR_BORDER_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);
pub const TOOLBAR_BORDER_RADIUS: f32 = 0.0;
pub const TOOLBAR_BORDER_WIDTH: f32 = 2.0;
pub const TOOLBAR_PADDING: f32 = 8.0;
pub const TOOLBAR_MARGIN: f32 = 16.0;
pub const TOOLBAR_ROW_GAP: f32 = 4.0;
pub const TOOLBAR_ITEM_SPACING: f32 = 4.0;

// Window Configuration
pub const WINDOW_TITLE: &str = "Bezy";
pub const WINDOW_WIDTH: f32 = 1024.0;
pub const WINDOW_HEIGHT: f32 = 768.0;

// Button Colors
pub const NORMAL_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(1.0, 0.4, 0.0);

// Button Outline Colors
pub const NORMAL_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
pub const HOVERED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);
pub const PRESSED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(1.0, 0.8, 0.3);
pub const PRESSED_BUTTON_ICON_COLOR: Color = Color::srgb(1.0, 0.9, 0.8);

// Path Drawing
//#[allow(dead_code)]
//pub const POINT_RADIUS: f32 = 4.0;
//#[allow(dead_code)]
//pub const PATH_COLOR: Color = Color::srgb(0.8, 0.0, 0.0);

// Glyph Point Rendering
pub const ON_CURVE_POINT_RADIUS: f32 = 4.0;
pub const OFF_CURVE_POINT_RADIUS: f32 = 4.0;

pub const ON_CURVE_POINT_COLOR: Color = Color::srgb(0.3, 1.0, 0.5);
pub const OFF_CURVE_POINT_COLOR: Color = Color::srgb(0.6, 0.4, 1.0);

// Point Layout Details
pub const ON_CURVE_SQUARE_ADJUSTMENT: f32 = 1.0; // Divider for square size to make it visually balanced
pub const ON_CURVE_INNER_CIRCLE_RATIO: f32 = 0.5; // Inner circle size as a ratio of half_size
pub const OFF_CURVE_INNER_CIRCLE_RATIO: f32 = 0.5; // Inner circle size as a ratio of the point radius

// Selection and Hover Styling
pub const SELECTION_POINT_RADIUS: f32 = 4.0; // Changed back to 4.0 to match unselected points
pub const SELECTED_CIRCLE_RADIUS_MULTIPLIER: f32 = 1.0; // Changed back to 1.0 for consistent sizing
#[allow(dead_code)]
pub const SELECTED_CROSS_SIZE_MULTIPLIER: f32 = 1.0; // Reduced to keep crosshairs within the circle/square
pub const SELECTED_POINT_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0); // Changed back to yellow

// Hover-related constants - disabled per user request
#[allow(dead_code)]
pub const HOVER_CIRCLE_RADIUS_MULTIPLIER: f32 = 1.0; // Multiplier for hover point circle
#[allow(dead_code)]
pub const HOVER_POINT_COLOR: Color = Color::srgba(0.3, 0.8, 1.0, 0.7); // Light blue with alpha
                                                                       // #[allow(dead_code)]
#[allow(dead_code)]
pub const HOVER_ORANGE_COLOR: Color = Color::srgb(1.0, 0.4, 0.0); // Bright orange for hover indicators

// Handle lines connecting on-curve and off-curve points
pub const HANDLE_LINE_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.3);

#[allow(dead_code)]
pub const POINT_STROKE_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
pub const PATH_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
#[allow(dead_code)]
pub const PATH_LINE_WIDTH: f32 = 2.0;
pub const USE_SQUARE_FOR_ON_CURVE: bool = true;

// Metrics Guide
pub const METRICS_GUIDE_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 0.5);

// Debug Settings
pub const DEBUG_SHOW_ORIGIN_CROSS: bool = true; // Set to true to show the red cross at origin

// UI Panel Colors
#[allow(dead_code)]
pub const PANEL_BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
#[allow(dead_code)]
pub const SECONDARY_TEXT_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);

// Coordinate Pane
#[allow(dead_code)]
pub const FOCUS_BACKGROUND_COLOR: Color = Color::srgb(1.0, 0.5, 0.0);
#[allow(dead_code)]
pub const OFF_CURVE_POINT_OUTER_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
#[allow(dead_code)]
pub const PATH_FILL_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);

// Background Color
pub const BACKGROUND_COLOR: Color = Color::srgb(0.05, 0.05, 0.05);

// Checkerboard Configuration
pub const CHECKERBOARD_UNIT_SIZE: f32 = 16.0; // Width and height of each square in pixels
pub const CHECKERBOARD_COLOR: Color = Color::srgb(0.128, 0.128, 0.128); // Single color for checkerboard squares

// Dynamic Checkerboard Scaling Configuration
// These settings control how the checkerboard grid scales with zoom levels to maintain performance
//
// How it works:
// - As you zoom out, the checkerboard grid size automatically doubles at regular intervals
// - This prevents performance issues from rendering too many small squares
// - The system calculates the appropriate grid size mathematically based on zoom level
//
// Configuration:
// - CHECKERBOARD_SCALE_FACTOR: How aggressively the grid scales (higher = more aggressive scaling)
// - CHECKERBOARD_MAX_ZOOM_VISIBLE: Hide checkerboard completely when zoomed out beyond this level
//
// Example: With scale factor 2.0:
//          At zoom scale 1.0, grid size = 16 pixels
//          At zoom scale 2.0, grid size = 32 pixels  
//          At zoom scale 4.0, grid size = 64 pixels
//          At zoom scale 8.0, grid size = 128 pixels (and so on, doubling infinitely)
//
pub const CHECKERBOARD_SCALE_FACTOR: f32 = 2.0; // How much the grid scales with zoom (2.0 = doubles)
pub const CHECKERBOARD_MAX_ZOOM_VISIBLE: f32 = 32.0; // Hide checkerboard when zoomed out beyond this level

// Sort Configuration
// Colors for sort metrics display based on active/inactive state
pub const SORT_ACTIVE_METRICS_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 0.5); // Same as current metrics color
pub const SORT_INACTIVE_METRICS_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.5); // Gray exactly between 0 and 1

// Button Styling
// pub const BUTTON_BORDER_RADIUS: f32 = 8.0;

// Knife Tool Colors
pub const KNIFE_LINE_COLOR: Color = Color::srgba(1.0, 0.3, 0.3, 0.9); // Reddish for cut line
pub const KNIFE_INTERSECTION_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0); // Yellow for intersections
pub const KNIFE_START_POINT_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 1.0); // Green for start point (same as on-curve points)
pub const KNIFE_DASH_LENGTH: f32 = 8.0; // Length of dash segments
pub const KNIFE_GAP_LENGTH: f32 = 4.0; // Length of gaps between dashes
pub const KNIFE_CROSS_SIZE: f32 = 8.0; // Size of crosses at intersection points

/// Creates a consistent styled container for UI widgets/panes
///
/// Returns a bundle of components that can be used to spawn a widget with
/// consistent styling across the application.
pub fn create_widget_style<T: Component + Default>(
    _asset_server: &Res<AssetServer>,
    position: PositionType,
    position_props: UiRect,
    marker: T,
    name: &str,
) -> impl Bundle {
    (
        Node {
            position_type: position,
            left: position_props.left,
            right: position_props.right,
            top: position_props.top,
            bottom: position_props.bottom,
            padding: UiRect::all(Val::Px(WIDGET_PADDING)),
            margin: UiRect::all(Val::Px(0.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(WIDGET_ROW_GAP),
            border: UiRect::all(Val::Px(WIDGET_BORDER_WIDTH)),
            // Add size constraints to keep widgets compact
            width: Val::Auto,
            height: Val::Auto,
            min_width: Val::Auto,
            min_height: Val::Auto,
            max_width: Val::Px(256.0), // Reduced maximum width for more compact widgets
            max_height: Val::Percent(50.0), // Limit height to prevent stretching to top of screen
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            ..default()
        },
        BackgroundColor(WIDGET_BACKGROUND_COLOR),
        BorderColor(WIDGET_BORDER_COLOR),
        BorderRadius::all(Val::Px(WIDGET_BORDER_RADIUS)),
        marker,
        Name::new(name.to_string()),
    )
}

/// Creates a text component with the mono font and standard styling
#[allow(dead_code)]
pub fn create_widget_text(
    asset_server: &Res<AssetServer>,
    text: &str,
    font_size: f32,
    color: Color,
) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font: asset_server.load(MONO_FONT_PATH),
            font_size,
            ..default()
        },
        TextColor(color),
    )
}

/// Creates a label (dim) and value (bright) text pair for a widget row
#[allow(dead_code)]
pub fn create_widget_label_value_pair(
    asset_server: &Res<AssetServer>,
    label: &str,
    value: &str,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Auto,
            height: Val::Auto,
            ..default()
        },
        (
            Node {
                margin: UiRect::right(Val::Px(4.0)),
                width: Val::Auto,
                ..default()
            },
            Text::new(label),
            TextFont {
                font: asset_server.load(MONO_FONT_PATH),
                font_size: WIDGET_TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
        ),
        (
            Text::new(value),
            TextFont {
                font: asset_server.load(MONO_FONT_PATH),
                font_size: WIDGET_TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ),
    )
}

#[allow(dead_code)]
pub fn get_default_text_style(asset_server: &Res<AssetServer>) -> TextFont {
    TextFont {
        font: asset_server.load(DEFAULT_FONT_PATH),
        font_size: 40.0,
        ..default()
    }
}

#[allow(dead_code)]
pub fn get_mono_text_style(asset_server: &Res<AssetServer>) -> TextFont {
    TextFont {
        font: asset_server.load(MONO_FONT_PATH),
        font_size: 40.0,
        ..default()
    }
}
