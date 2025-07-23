//! Dark Mode Theme for Bezy
//!
//! This is the default theme for the Bezy font editor.

use super::BezyTheme;
use bevy::prelude::*;

/// Dark mode theme implementation
///
/// This theme provides the classic Bezy dark mode appearance with:
/// - Black background (#000000)
/// - Dark gray UI elements
/// - Bright colors for points and tools
/// - High contrast for editing clarity
pub struct DarkModeTheme;

impl BezyTheme for DarkModeTheme {
    fn name(&self) -> &'static str {
        "Dark Mode"
    }

    // TYPOGRAPHY

    fn normal_text_color(&self) -> Color {
        Color::srgb(0.9, 0.9, 0.9)
    }

    fn secondary_text_color(&self) -> Color {
        Color::srgb(0.6, 0.6, 0.6)
    }

    fn highlight_text_color(&self) -> Color {
        Color::srgb(1.0, 0.8, 0.0)
    }

    // =================================================================
    // COLORS - BACKGROUNDS & UI
    // =================================================================

    fn background_color(&self) -> Color {
        Color::srgb(0.0, 0.0, 0.0)
    }

    fn widget_background_color(&self) -> Color {
        Color::srgba(0.1, 0.1, 0.1, 1.0)
    }

    fn widget_border_color(&self) -> Color {
        Color::srgba(0.5, 0.5, 0.5, 1.0)
    }

    fn toolbar_background_color(&self) -> Color {
        Color::srgba(0.1, 0.1, 0.1, 1.0)
    }

    fn toolbar_icon_color(&self) -> Color {
        Color::srgb(0.75, 0.75, 0.75)
    }

    fn toolbar_border_color(&self) -> Color {
        Color::srgba(0.5, 0.5, 0.5, 1.0)
    }

    fn panel_background_color(&self) -> Color {
        Color::srgb(0.15, 0.15, 0.15)
    }

    fn normal_button_color(&self) -> Color {
        Color::srgb(0.1, 0.1, 0.1)
    }

    fn hovered_button_color(&self) -> Color {
        Color::srgb(0.25, 0.25, 0.25)
    }

    fn pressed_button_color(&self) -> Color {
        Color::srgb(1.0, 0.4, 0.0)
    }

    fn normal_button_outline_color(&self) -> Color {
        Color::srgb(0.5, 0.5, 0.5)
    }

    fn hovered_button_outline_color(&self) -> Color {
        Color::srgb(0.75, 0.75, 0.75)
    }

    fn pressed_button_outline_color(&self) -> Color {
        Color::srgb(1.0, 0.8, 0.3)
    }

    fn pressed_button_icon_color(&self) -> Color {
        Color::srgb(1.0, 1.0, 1.0)
    }

    fn focus_background_color(&self) -> Color {
        Color::srgb(1.0, 0.5, 0.0)
    }

    fn text_editor_background_color(&self) -> Color {
        Color::srgb(0.9, 0.9, 0.9)
    }

    // =================================================================
    // GLYPH RENDERING
    // =================================================================

    /// On-curve point colors (two-layer system)
    fn on_curve_primary_color(&self) -> Color {
        Color::srgb(0.3, 1.0, 0.5) // Bright green
    }

    fn on_curve_secondary_color(&self) -> Color {
        Color::srgb(0.1, 0.4, 0.15) // Dark green
    }

    /// Off-curve point colors (two-layer system)  
    fn off_curve_primary_color(&self) -> Color {
        Color::srgb(0.6, 0.4, 1.0) // Purple/blue
    }

    fn off_curve_secondary_color(&self) -> Color {
        Color::srgb(0.2, 0.15, 0.4) // Dark purple
    }

    fn path_line_color(&self) -> Color {
        Color::srgba(1.0, 1.0, 1.0, 1.0)
    }

    fn path_stroke_color(&self) -> Color {
        Color::srgb(0.9, 0.9, 0.9)
    }

    fn point_stroke_color(&self) -> Color {
        Color::srgba(0.1, 0.1, 0.1, 0.8)
    }

    fn handle_line_color(&self) -> Color {
        Color::srgba(0.5, 0.5, 0.5, 0.3)
    }

    // =================================================================
    // INFO COLORS - SEMANTIC COLORS
    // =================================================================

    fn error_color(&self) -> Color {
        Color::srgb(1.0, 0.0, 0.0) // Red
    }

    fn action_color(&self) -> Color {
        Color::srgb(1.0, 0.5, 0.0) // Orange
    }

    fn selected_color(&self) -> Color {
        Color::srgb(1.0, 1.0, 0.0) // Yellow
    }

    fn active_color(&self) -> Color {
        Color::srgb(0.0, 1.0, 0.0) // Green
    }

    fn helper_color(&self) -> Color {
        Color::srgb(0.0, 0.5, 1.0) // Blue
    }

    fn special_color(&self) -> Color {
        Color::srgb(0.8, 0.0, 1.0) // Purple
    }

    // =================================================================
    // SELECTION & INTERACTION
    // =================================================================

    /// Selected point colors (two-layer system for crosshairs)
    fn selected_primary_color(&self) -> Color {
        Color::srgba(1.0, 1.0, 0.0, 1.0) // Bright yellow
    }

    fn selected_secondary_color(&self) -> Color {
        Color::srgba(0.4, 0.4, 0.0, 1.0) // Dark yellow
    }

    fn hover_point_color(&self) -> Color {
        Color::srgba(0.3, 0.8, 1.0, 0.7)
    }

    fn hover_orange_color(&self) -> Color {
        Color::srgb(1.0, 0.4, 0.0)
    }

    // =================================================================
    // EDITING TOOLS
    // =================================================================

    fn knife_line_color(&self) -> Color {
        Color::srgba(1.0, 0.3, 0.3, 0.9)
    }

    fn knife_intersection_color(&self) -> Color {
        Color::srgba(1.0, 1.0, 0.0, 1.0)
    }

    fn knife_start_point_color(&self) -> Color {
        Color::srgba(0.3, 1.0, 0.5, 1.0)
    }

    fn pen_point_color(&self) -> Color {
        Color::srgb(1.0, 1.0, 0.0)
    }

    fn pen_start_point_color(&self) -> Color {
        Color::srgb(0.0, 1.0, 0.5)
    }

    fn pen_line_color(&self) -> Color {
        Color::srgba(1.0, 1.0, 1.0, 0.9)
    }

    fn hyper_point_color(&self) -> Color {
        Color::srgba(0.3, 1.0, 0.5, 1.0)
    }

    fn hyper_line_color(&self) -> Color {
        Color::srgba(0.5, 0.8, 1.0, 0.8)
    }

    fn hyper_close_indicator_color(&self) -> Color {
        Color::srgba(1.0, 1.0, 0.0, 1.0)
    }

    fn shape_preview_color(&self) -> Color {
        Color::srgba(0.8, 0.8, 0.8, 0.6)
    }

    // =================================================================
    // METABALLS
    // =================================================================

    fn metaball_gizmo_color(&self) -> Color {
        Color::srgba(0.3, 0.7, 1.0, 0.6)
    }

    fn metaball_outline_color(&self) -> Color {
        Color::srgba(1.0, 1.0, 1.0, 1.0)
    }

    fn metaball_selected_color(&self) -> Color {
        Color::srgba(1.0, 0.8, 0.0, 0.8)
    }

    // =================================================================
    // GUIDES & GRIDS
    // =================================================================

    fn metrics_guide_color(&self) -> Color {
        Color::srgba(0.3, 1.0, 0.5, 0.5)
    }

    fn checkerboard_color_1(&self) -> Color {
        Color::srgb(0.128, 0.128, 0.128)
    }

    fn checkerboard_color_2(&self) -> Color {
        Color::srgb(0.150, 0.150, 0.150)
    }

    fn checkerboard_color(&self) -> Color {
        Color::srgba(0.1, 0.1, 0.1, 0.5)
    }

    // =================================================================
    // SORTING & LAYOUT
    // =================================================================

    fn sort_active_metrics_color(&self) -> Color {
        Color::srgba(0.3, 1.0, 0.5, 0.5)
    }

    fn sort_inactive_metrics_color(&self) -> Color {
        Color::srgba(0.5, 0.5, 0.5, 0.5)
    }

    fn sort_active_outline_color(&self) -> Color {
        Color::srgb(1.0, 0.4, 0.0)
    }

    fn sort_inactive_outline_color(&self) -> Color {
        Color::srgb(0.75, 0.75, 0.75)
    }
}
