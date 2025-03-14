// General purpose debugging functions

use bevy::prelude::*;
use crate::theme::{TEXT_COLOR, ARABIC_DEBUG_FONT_PATH};

#[allow(dead_code)]
pub fn green_text(text: String) -> String {
    format!("\x1b[32m{}\x1b[0m", text)
}

#[allow(dead_code)]
fn red_text(text: String) -> String {
    format!("\x1b[31m{}\x1b[0m", text)
}

#[allow(dead_code)]
fn yellow_text(text: String) -> String {
    format!("\x1b[33m{}\x1b[0m", text)
}

/// Plugin for the Arabic debug text display in the corner of the screen
pub struct ArabicDebugPlugin;

impl Plugin for ArabicDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArabicDebugTextState>()
           .add_systems(Startup, setup_arabic_debug_text)
           .add_systems(Update, ensure_debug_text_visible);
    }
}

/// Component marker for the debug text entity
#[derive(Component)]
pub struct ArabicDebugText;

/// Resource to track if the text has been created
#[derive(Resource, Default)]
pub struct ArabicDebugTextState {
    text_entity: Option<Entity>,
}

/// System to set up the Arabic debug text (runs once during startup)
fn setup_arabic_debug_text(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut state: ResMut<ArabicDebugTextState>
) {
    // Create the debug text UI element
    let entity = create_debug_text(&mut commands, &asset_server);
    state.text_entity = Some(entity);
    
    info!("Arabic debug text initialized");
}

/// Creates the Arabic debug text UI element
fn create_debug_text(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    // Arabic text to display
    let text = "إشهد يا الهي بانك خلقتني";
    
    // Create the text entity directly, positioned in the lower right corner
    commands
        .spawn((
            // Place the text in the lower right corner
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(32.0),
                right: Val::Px(32.0),
                ..default()
            },
            Text::new(text),
            TextFont {
                font: asset_server.load(ARABIC_DEBUG_FONT_PATH),
                font_size: 64.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
            ArabicDebugText,
        ))
        .id()
}

/// System that ensures the Arabic debug text is visible
fn ensure_debug_text_visible(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut text_state: ResMut<ArabicDebugTextState>,
    text_query: Query<Entity, With<ArabicDebugText>>,
) {
    // Check if the text exists
    let text_exists = match text_state.text_entity {
        Some(entity) => text_query.contains(entity),
        None => false,
    };
    
    // If the text doesn't exist, create it
    if !text_exists {
        // Clean up any existing text entities first (just in case)
        for entity in text_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        // Create a new text entity and store its entity
        let text_entity = create_debug_text(&mut commands, &asset_server);
        text_state.text_entity = Some(text_entity);
        
        info!("Arabic debug text restored");
    }
}
