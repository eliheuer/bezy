use bevy::prelude::Color;
use bevy::prelude::*;

// Font Path
pub const DEFAULT_FONT_PATH: &str = "fonts/bezy-grotesk-regular.ttf";

// UI Colors
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(1.0, 0.6, 0.0);

// Button Outline Colors
pub const NORMAL_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
pub const HOVERED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.99, 0.99, 0.99);
pub const PRESSED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(1.0, 0.6, 0.0);

// Path Drawing
#[allow(dead_code)]
pub const POINT_RADIUS: f32 = 4.0;
#[allow(dead_code)]
pub const PATH_COLOR: Color = Color::srgb(0.8, 0.0, 0.0);

// Glyph Point Rendering
pub const ON_CURVE_POINT_RADIUS: f32 = 6.0;
pub const OFF_CURVE_POINT_RADIUS: f32 = 6.0;

pub const ON_CURVE_POINT_COLOR: Color = Color::srgba(1.0, 0.75, 0.0, 1.0);
pub const OFF_CURVE_POINT_COLOR: Color = Color::srgba(0.0, 1.0, 0.4, 1.0);

#[allow(dead_code)]
pub const SELECTED_POINT_COLOR: Color = Color::srgba(1.0, 0.8, 0.0, 1.0);
#[allow(dead_code)]
pub const HOVER_POINT_COLOR: Color = Color::srgba(0.0, 1.0, 0.8, 1.0);
#[allow(dead_code)]
pub const POINT_STROKE_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
pub const PATH_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
#[allow(dead_code)]
pub const PATH_LINE_WIDTH: f32 = 2.0;
pub const USE_SQUARE_FOR_ON_CURVE: bool = true;

// Background Color
pub const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

// Checkerboard Configuration
pub const CHECKERBOARD_UNIT_SIZE: f32 = 8.0; // Width and height of each square in pixels
pub const CHECKERBOARD_COLOR: Color = Color::srgb(0.15, 0.15, 0.15); // Single color for checkerboard squares

// Button Styling
pub const BUTTON_BORDER_RADIUS: f32 = 8.0;

pub fn get_default_text_style(asset_server: &Res<AssetServer>) -> TextFont {
    TextFont {
        font: asset_server.load(DEFAULT_FONT_PATH),
        font_size: 32.0,
        ..default()
    }
}
