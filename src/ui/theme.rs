//! UI Theme file
//!
//! This file contains all the constants for visual styling of the UI.
//! Non-visual constants are in the `core::settings` file.
//! You can create a custome theme for bevy by copying this file and
//! modifying the constants to your liking.

#![allow(unused_variables)]

use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::ui::prelude::*;

// Font Paths
#[allow(dead_code)]
pub const GROTESK_FONT_PATH: &str = "fonts/bezy-grotesk-regular.ttf";
#[allow(dead_code)]
pub const MONO_FONT_PATH: &str = "fonts/HasubiMono-Regular.ttf";

// Font Sizes
#[allow(dead_code)]
pub const WIDGET_TITLE_FONT_SIZE: f32 = 24.0;
#[allow(dead_code)]
pub const WIDGET_TEXT_FONT_SIZE: f32 = 24.0;
#[allow(dead_code)]
pub const DEFAULT_FONT_SIZE: f32 = 16.0;
#[allow(dead_code)]
pub const SMALL_FONT_SIZE: f32 = 12.0;
#[allow(dead_code)]
pub const LARGE_FONT_SIZE: f32 = 20.0;

// Widget Visual Style Constants
#[allow(dead_code)]
pub const WIDGET_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);
#[allow(dead_code)]
pub const WIDGET_BG_COLOR: Color = WIDGET_BACKGROUND_COLOR; // Alias for compatibility
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

// Text color constants
#[allow(dead_code)]
pub const NORMAL_TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
#[allow(dead_code)]
pub const SECONDARY_TEXT_COLOR_THEME: Color = Color::srgb(0.6, 0.6, 0.6);
#[allow(dead_code)]
pub const HIGHLIGHT_TEXT_COLOR: Color = Color::srgb(1.0, 0.8, 0.0);

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

pub const TOOLBAR_PADDING: f32 = 0.0;
pub const TOOLBAR_CONTAINER_MARGIN: f32 = 16.0;
pub const TOOLBAR_ITEM_SPACING: f32 = 4.0;

// Window Configuration
#[allow(dead_code)]
pub const WINDOW_TITLE: &str = "Bezy";
#[allow(dead_code)]
pub const WINDOW_WIDTH: f32 = 1024.0;
#[allow(dead_code)]
pub const WINDOW_HEIGHT: f32 = 768.0;

// Button Colors
pub const NORMAL_BUTTON_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
pub const HOVERED_BUTTON_COLOR: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON_COLOR: Color = Color::srgb(1.0, 0.4, 0.0);

// Button Outline Colors
pub const NORMAL_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
pub const HOVERED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);
pub const PRESSED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(1.0, 0.8, 0.3);
pub const PRESSED_BUTTON_ICON_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

// Toolbar Button Sizes
pub const TOOLBAR_BUTTON_SIZE: f32 = 64.0;
pub const BUTTON_ICON_SIZE: f32 = 48.0;

// Glyph Point Rendering
pub const ON_CURVE_POINT_RADIUS: f32 = 4.0;
pub const OFF_CURVE_POINT_RADIUS: f32 = 4.0;
pub const ON_CURVE_POINT_COLOR: Color = Color::srgb(0.3, 1.0, 0.5);
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
pub const DEBUG_SHOW_ORIGIN_CROSS: bool = false;

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
pub const PATH_STROKE_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

// Background Color
#[allow(dead_code)]
pub const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);

// Checkerboard grid configuration
pub const CHECKERBOARD_ENABLED_BY_DEFAULT: bool = true;
pub const CHECKERBOARD_DEFAULT_UNIT_SIZE: f32 = 32.0;
#[allow(dead_code)]
pub const CHECKERBOARD_COLOR_1: Color = Color::srgb(0.128, 0.128, 0.128); // Dark squares
#[allow(dead_code)]
pub const CHECKERBOARD_COLOR_2: Color = Color::srgb(0.150, 0.150, 0.150); // Light squares
#[allow(dead_code)]
pub const CHECKERBOARD_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.5); // Subtle gray for design space grid
#[allow(dead_code)]
pub const CHECKERBOARD_SCALE_FACTOR: f32 = 2.0;
#[allow(dead_code)]
pub const CHECKERBOARD_MAX_ZOOM_VISIBLE: f32 = 32.0;

// Sort Configuration
#[allow(dead_code)]
pub const SORT_ACTIVE_METRICS_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 0.5);
#[allow(dead_code)]
pub const SORT_INACTIVE_METRICS_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.5);
#[allow(dead_code)]
pub const SORT_ACTIVE_OUTLINE_COLOR: Color = Color::srgb(1.0, 0.4, 0.0);
#[allow(dead_code)]
pub const SORT_INACTIVE_OUTLINE_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);

// Sort Spacing - consistent gaps between sorts both horizontally and vertically
#[allow(dead_code)]
pub const SORT_HORIZONTAL_PADDING: f32 = 256.0;
#[allow(dead_code)]
pub const SORT_VERTICAL_PADDING: f32 = 256.0;

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

// Metaballs Configuration
#[allow(dead_code)]
pub const METABALL_GIZMO_COLOR: Color = Color::srgba(0.3, 0.7, 1.0, 0.6);
#[allow(dead_code)]
pub const METABALL_OUTLINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
#[allow(dead_code)]
pub const METABALL_SELECTED_COLOR: Color = Color::srgba(1.0, 0.8, 0.0, 0.8);

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
) -> Text {
    Text(text.to_string())
}

/// Creates a label (dim) and value (bright) text pair for a widget row
#[allow(dead_code)]
pub fn create_widget_label_value_pair(
    asset_server: &Res<AssetServer>,
    label: &str,
    value: &str,
) -> (Node, Text) {
    (
        Node {
            // Set fields directly here
            ..Default::default()
        },
        Text(format!("{} {}", label, value))
    )
}

/// Create a default text style with the given font
/// Returns a Text component ready for use in UI
#[allow(dead_code)]
pub fn get_default_text_style(asset_server: &Res<AssetServer>) -> Text {
    Text::new("")
}

/// Create a text style with custom content and font size
#[allow(dead_code)]
pub fn create_text_style(text: &str, font_size: f32) -> Text {
    Text::new(text)
}

pub const LINE_LEADING: f32 = 0.0; 
