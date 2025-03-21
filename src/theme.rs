use bevy::prelude::Color;
use bevy::prelude::*;

// Font Path
pub const DEFAULT_FONT_PATH: &str = "fonts/bezy-grotesk-regular.ttf";

// Window Configuration
pub const WINDOW_TITLE: &str = "Bezy";
pub const WINDOW_WIDTH: f32 = 1024.0;
pub const WINDOW_HEIGHT: f32 = 900.0;

// Button Colors
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(1.0, 0.4, 0.0);

// Button Outline Colors
pub const NORMAL_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
pub const HOVERED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.99, 0.99, 0.99);
pub const PRESSED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(1.0, 0.6, 0.0);

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
pub const SELECTED_CROSS_SIZE_MULTIPLIER: f32 = 0.7; // Reduced to keep crosshairs within the circle/square
pub const SELECTED_POINT_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0); // Changed back to yellow

// Hover-related constants - disabled per user request
#[allow(dead_code)]
pub const HOVER_CIRCLE_RADIUS_MULTIPLIER: f32 = 1.0; // Multiplier for hover point circle
#[allow(dead_code)]
pub const HOVER_POINT_COLOR: Color = Color::srgba(0.3, 0.8, 1.0, 0.7); // Light blue with alpha
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
pub const DEBUG_SHOW_ORIGIN_CROSS: bool = false; // Set to true to show the red cross at origin

// Background Color
pub const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

// UI Panel Colors
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

// Checkerboard Configuration
pub const CHECKERBOARD_UNIT_SIZE: f32 = 8.0; // Width and height of each square in pixels
pub const CHECKERBOARD_COLOR: Color = Color::srgb(0.125, 0.125, 0.125); // Single color for checkerboard squares

// Button Styling
pub const BUTTON_BORDER_RADIUS: f32 = 8.0;

// Knife Tool Colors
pub const KNIFE_LINE_COLOR: Color = Color::srgba(1.0, 0.3, 0.3, 0.9); // Reddish for cut line
pub const KNIFE_INTERSECTION_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0); // Yellow for intersections
pub const KNIFE_START_POINT_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 1.0); // Green for start point (same as on-curve points)
pub const KNIFE_DASH_LENGTH: f32 = 8.0; // Length of dash segments
pub const KNIFE_GAP_LENGTH: f32 = 4.0; // Length of gaps between dashes
pub const KNIFE_CROSS_SIZE: f32 = 8.0; // Size of crosses at intersection points

// Camera Settings
pub const CAMERA_ZOOM_FACTOR: f32 = 0.5; // Factor used in zoom level calculation
pub const CAMERA_MIN_SCALE: f32 = 0.8; // Minimum camera scale to prevent excessive zooming

pub fn get_default_text_style(asset_server: &Res<AssetServer>) -> TextFont {
    TextFont {
        font: asset_server.load(DEFAULT_FONT_PATH),
        font_size: 40.0,
        ..default()
    }
}
