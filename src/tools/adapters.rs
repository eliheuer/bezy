//! Adapters to bridge new clean tools with the legacy toolbar system
//!
//! This allows us to use the new clean EditTool trait while still working
//! with the existing toolbar infrastructure. Eventually we'll remove these
//! adapters and fully migrate to the new system.

use super::*;
use crate::ui::toolbars::edit_mode_toolbar::{
    EditTool as LegacyEditTool, ToolId,
};
use bevy::prelude::*;

/// Adapter for the new select tool to work with legacy system
pub struct SelectToolAdapter;

impl LegacyEditTool for SelectToolAdapter {
    fn id(&self) -> ToolId {
        "select"
    }

    fn name(&self) -> &'static str {
        "Select"
    }

    fn icon(&self) -> &'static str {
        "\u{E010}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('v')
    }

    fn default_order(&self) -> i32 {
        10 // First tool in toolbar
    }

    fn description(&self) -> &'static str {
        "Select and manipulate objects"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(select::SelectModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Select);
    }

    fn on_enter(&self) {
        info!("Select tool activated");
    }

    fn on_exit(&self) {
        info!("Select tool deactivated");
    }
}

/// Adapter for the new pen tool to work with legacy system
pub struct PenToolAdapter;

impl LegacyEditTool for PenToolAdapter {
    fn id(&self) -> ToolId {
        "pen"
    }

    fn name(&self) -> &'static str {
        "Pen"
    }

    fn icon(&self) -> &'static str {
        "\u{E011}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('p')
    }

    fn default_order(&self) -> i32 {
        20 // After select
    }

    fn description(&self) -> &'static str {
        "Draw paths and contours"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(pen::PenModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Pen);
    }

    fn on_enter(&self) {
        info!("Pen tool activated");
    }

    fn on_exit(&self) {
        info!("Pen tool deactivated");
    }
}

/// Adapter for the new text tool to work with legacy system
pub struct TextToolAdapter;

impl LegacyEditTool for TextToolAdapter {
    fn id(&self) -> ToolId {
        "text"
    }

    fn name(&self) -> &'static str {
        "Text"
    }

    fn icon(&self) -> &'static str {
        "\u{E017}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('t')
    }

    fn default_order(&self) -> i32 {
        40 // After drawing tools
    }

    fn description(&self) -> &'static str {
        "Place text and create sorts (Tab for modes)"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(text::TextModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Text);
    }

    fn on_enter(&self) {
        info!("Text tool activated with submenu");
    }

    fn on_exit(&self) {
        info!("Text tool deactivated");
    }
}

/// Plugin to register all the clean tool adapters
pub struct CleanToolsPlugin;

impl Plugin for CleanToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_clean_tools);
    }
}

/// Adapter for the shapes tool to work with legacy system
pub struct ShapesToolAdapter;

impl LegacyEditTool for ShapesToolAdapter {
    fn id(&self) -> ToolId {
        "shapes"
    }

    fn name(&self) -> &'static str {
        "Shapes"
    }

    fn icon(&self) -> &'static str {
        "\u{E016}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('s')
    }

    fn default_order(&self) -> i32 {
        30 // After pen, before text
    }

    fn description(&self) -> &'static str {
        "Create geometric shapes"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(crate::core::io::input::InputMode::Shape);
    }

    fn on_enter(&self) {
        info!("Shapes tool activated");
    }

    fn on_exit(&self) {
        info!("Shapes tool deactivated");
    }
}

/// Adapter for the knife tool to work with legacy system
pub struct KnifeToolAdapter;

impl LegacyEditTool for KnifeToolAdapter {
    fn id(&self) -> ToolId {
        "knife"
    }

    fn name(&self) -> &'static str {
        "Knife"
    }

    fn icon(&self) -> &'static str {
        "\u{E012}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('k')
    }

    fn default_order(&self) -> i32 {
        110 // Advanced tool
    }

    fn description(&self) -> &'static str {
        "Cut contours at specific points"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(knife::KnifeModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Knife);
    }

    fn on_enter(&self) {
        info!("Knife tool activated");
    }

    fn on_exit(&self) {
        info!("Knife tool deactivated");
    }
}

/// Adapter for the hyper tool to work with legacy system
pub struct HyperToolAdapter;

impl LegacyEditTool for HyperToolAdapter {
    fn id(&self) -> ToolId {
        "hyper"
    }

    fn name(&self) -> &'static str {
        "Hyper"
    }

    fn icon(&self) -> &'static str {
        "\u{E018}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('h')
    }

    fn default_order(&self) -> i32 {
        100 // Advanced tool
    }

    fn description(&self) -> &'static str {
        "Draw smooth hyperbezier curves"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(hyper::HyperModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Hyper);
    }

    fn on_enter(&self) {
        info!("Hyper tool activated");
    }

    fn on_exit(&self) {
        info!("Hyper tool deactivated");
    }
}

/// Adapter for the metaballs tool to work with legacy system
pub struct MetaballsToolAdapter;

impl LegacyEditTool for MetaballsToolAdapter {
    fn id(&self) -> ToolId {
        "metaballs"
    }

    fn name(&self) -> &'static str {
        "Metaballs"
    }

    fn icon(&self) -> &'static str {
        "\u{E019}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('m')
    }

    fn default_order(&self) -> i32 {
        120 // Advanced tool
    }

    fn description(&self) -> &'static str {
        "Create organic shapes with metaball effects"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(metaballs::MetaballsModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Metaballs);
    }

    fn on_enter(&self) {
        info!("Metaballs tool activated");
    }

    fn on_exit(&self) {
        info!("Metaballs tool deactivated");
    }
}

/// System to register all clean tools with the legacy toolbar system
fn register_clean_tools(
    mut tool_registry: ResMut<
        crate::ui::toolbars::edit_mode_toolbar::ToolRegistry,
    >,
) {
    tool_registry.register_tool(Box::new(SelectToolAdapter));
    tool_registry.register_tool(Box::new(PenToolAdapter));
    // tool_registry.register_tool(Box::new(TextToolAdapter)); // Disabled - using legacy text tool with submenu
    tool_registry.register_tool(Box::new(ShapesToolAdapter));
    tool_registry.register_tool(Box::new(KnifeToolAdapter));
    tool_registry.register_tool(Box::new(HyperToolAdapter));
    tool_registry.register_tool(Box::new(MetaballsToolAdapter));

    info!("Registered clean tools with legacy toolbar system");
}
