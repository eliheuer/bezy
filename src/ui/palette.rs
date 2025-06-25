//! A palette of colors and other style constants for the Bezy UI.

use bevy::prelude::Color;

// Font Path
#[allow(dead_code)]
pub const DEFAULT_FONT_PATH: &str = "fonts/bezy-grotesk-regular.ttf";
#[allow(dead_code)]
pub const MONO_FONT_PATH: &str = "fonts/HasubiMono-Regular.ttf";

// Font Sizes
#[allow(dead_code)]
pub const WIDGET_TITLE_FONT_SIZE: f32 = 24.0;
#[allow(dead_code)]
pub const WIDGET_TEXT_FONT_SIZE: f32 = 24.0;

// Widget Visual Style Constants
#[allow(dead_code)]
pub const WIDGET_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);
#[allow(dead_code)]
pub const WIDGET_BORDER_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);
#[allow(dead_code)]
pub const WIDGET_BORDER_RADIUS: f32 = 0.0;
#[allow(dead_code)]
pub const WIDGET_BORDER_WIDTH: f32 = 2.0;
#[allow(dead_code)]
pub const WIDGET_PADDING: f32 = 16.0;
#[allow(dead_code)]
pub const WIDGET_MARGIN: f32 = 24.0;
#[allow(dead_code)]
pub const WIDGET_ROW_GAP: f32 = 0.0;

// Gizmo Configuration
#[allow(dead_code)]
pub const GIZMO_LINE_WIDTH: f32 = 3.0;

// Toolbar Visual Style Constants
#[allow(dead_code)]
pub const TOOLBAR_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);
#[allow(dead_code)]
pub const TOOLBAR_ICON_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);
#[allow(dead_code)]
pub const TOOLBAR_BORDER_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);
#[allow(dead_code)]
pub const TOOLBAR_BORDER_RADIUS: f32 = 0.0;
#[allow(dead_code)]
pub const TOOLBAR_BORDER_WIDTH: f32 = 2.0;
#[allow(dead_code)]
pub const TOOLBAR_PADDING: f32 = 8.0;
#[allow(dead_code)]
pub const TOOLBAR_MARGIN: f32 = 16.0;
#[allow(dead_code)]
pub const TOOLBAR_ROW_GAP: f32 = 4.0;
#[allow(dead_code)]
pub const TOOLBAR_ITEM_SPACING: f32 = 4.0;

// Window Configuration
#[allow(dead_code)]
pub const WINDOW_TITLE: &str = "Bezy";
#[allow(dead_code)]
pub const WINDOW_WIDTH: f32 = 1024.0;
#[allow(dead_code)]
pub const WINDOW_HEIGHT: f32 = 768.0;

// Button Colors
#[allow(dead_code)]
pub const NORMAL_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);
#[allow(dead_code)]
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
#[allow(dead_code)]
pub const PRESSED_BUTTON: Color = Color::srgb(1.0, 0.4, 0.0);

// Button Outline Colors
#[allow(dead_code)]
pub const NORMAL_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
#[allow(dead_code)]
pub const HOVERED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);
#[allow(dead_code)]
pub const PRESSED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(1.0, 0.8, 0.3);
#[allow(dead_code)]
pub const PRESSED_BUTTON_ICON_COLOR: Color = Color::srgb(1.0, 0.9, 0.8);

// Glyph Point Rendering
#[allow(dead_code)]
pub const ON_CURVE_POINT_RADIUS: f32 = 4.0;
#[allow(dead_code)]
pub const OFF_CURVE_POINT_RADIUS: f32 = 4.0;

#[allow(dead_code)]
pub const ON_CURVE_POINT_COLOR: Color = Color::srgb(0.3, 1.0, 0.5);
#[allow(dead_code)]
pub const OFF_CURVE_POINT_COLOR: Color = Color::srgb(0.6, 0.4, 1.0);

// Point Layout Details
#[allow(dead_code)]
pub const ON_CURVE_SQUARE_ADJUSTMENT: f32 = 1.0;
#[allow(dead_code)]
pub const ON_CURVE_INNER_CIRCLE_RATIO: f32 = 0.5;
#[allow(dead_code)]
pub const OFF_CURVE_INNER_CIRCLE_RATIO: f32 = 0.5;

// Selection and Hover Styling
#[allow(dead_code)]
pub const SELECTION_POINT_RADIUS: f32 = 4.0;
#[allow(dead_code)]
pub const SELECTED_CIRCLE_RADIUS_MULTIPLIER: f32 = 1.0;
#[allow(dead_code)]
pub const SELECTED_CROSS_SIZE_MULTIPLIER: f32 = 1.0;
#[allow(dead_code)]
pub const SELECTED_POINT_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0);

// Hover-related constants
#[allow(dead_code)]
pub const HOVER_CIRCLE_RADIUS_MULTIPLIER: f32 = 1.0;
#[allow(dead_code)]
pub const HOVER_POINT_COLOR: Color = Color::srgba(0.3, 0.8, 1.0, 0.7);
#[allow(dead_code)]
pub const HOVER_ORANGE_COLOR: Color = Color::srgb(1.0, 0.4, 0.0);

// Handle lines connecting on-curve and off-curve points
#[allow(dead_code)]
pub const HANDLE_LINE_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.3);

#[allow(dead_code)]
pub const POINT_STROKE_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
#[allow(dead_code)]
pub const PATH_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
#[allow(dead_code)]
pub const PATH_LINE_WIDTH: f32 = 2.0;
#[allow(dead_code)]
pub const USE_SQUARE_FOR_ON_CURVE: bool = true;

// Metrics Guide
#[allow(dead_code)]
pub const METRICS_GUIDE_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 0.5);

// Debug Settings
#[allow(dead_code)]
pub const DEBUG_SHOW_ORIGIN_CROSS: bool = true;

// UI Panel Colors
#[allow(dead_code)]
pub const PANEL_BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
#[allow(dead_code)]
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
#[allow(dead_code)]
pub const BACKGROUND_COLOR: Color = Color::srgb(0.05, 0.05, 0.05);

// Checkerboard Configuration
#[allow(dead_code)]
pub const CHECKERBOARD_UNIT_SIZE: f32 = 32.0;
#[allow(dead_code)]
pub const CHECKERBOARD_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
#[allow(dead_code)]
pub const CHECKERBOARD_DARK_COLOR: Color = Color::srgb(0.05, 0.05, 0.05);

#[allow(dead_code)]
pub const CHECKERBOARD_SCALE_FACTOR: f32 = 2.0;
#[allow(dead_code)]
pub const CHECKERBOARD_MAX_ZOOM_VISIBLE: f32 = 32.0;

// Sort Configuration
#[allow(dead_code)]
pub const SORT_ACTIVE_METRICS_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 0.5);
#[allow(dead_code)]
pub const SORT_INACTIVE_METRICS_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.5);

// Knife Tool Colors
#[allow(dead_code)]
pub const KNIFE_LINE_COLOR: Color = Color::srgba(1.0, 0.3, 0.3, 0.9);
#[allow(dead_code)]
pub const KNIFE_INTERSECTION_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0);
#[allow(dead_code)]
pub const KNIFE_START_POINT_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 1.0);
#[allow(dead_code)]
pub const KNIFE_DASH_LENGTH: f32 = 8.0;
#[allow(dead_code)]
pub const KNIFE_GAP_LENGTH: f32 = 4.0;
#[allow(dead_code)]
pub const KNIFE_CROSS_SIZE: f32 = 8.0; 