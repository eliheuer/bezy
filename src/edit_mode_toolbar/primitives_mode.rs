use crate::edit_mode_toolbar::EditModeSystem;
use crate::theme::*;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;

// Import primitive shapes modules directly
use crate::edit_mode_toolbar::primitives::base;

// An enum to track which primitive type is currently selected
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum PrimitiveType {
    #[default]
    Rectangle,
    Ellipse,
    RoundedRectangle,
    // Future primitive types can be added here (stars, hexagons, etc.)
}

impl PrimitiveType {
    // Get the icon for each primitive type
    pub fn get_icon(&self) -> &'static str {
        match self {
            // Using different icons for each primitive type
            PrimitiveType::Rectangle => "\u{E018}",
            PrimitiveType::Ellipse => "\u{E019}",
            PrimitiveType::RoundedRectangle => "\u{E020}",
        }
    }

    // Get the display name for each primitive type
    pub fn display_name(&self) -> &'static str {
        match self {
            PrimitiveType::Rectangle => "Rectangle",
            PrimitiveType::Ellipse => "Ellipse",
            PrimitiveType::RoundedRectangle => "Rounded Rectangle",
        }
    }
}

// Component to mark primitive sub-menu buttons
#[derive(Component)]
pub struct PrimitiveSubMenuButton;

// Component to associate a button with its primitive type
#[derive(Component)]
pub struct PrimitiveTypeButton(pub PrimitiveType);

// Resource to track the currently selected primitive type
#[derive(Resource, Default)]
pub struct CurrentPrimitiveType(pub PrimitiveType);

pub struct PrimitivesMode;

impl EditModeSystem for PrimitivesMode {
    fn update(&self, commands: &mut Commands) {
        // Disable selection mode while in primitives mode
        commands.insert_resource(
            crate::edit_mode_toolbar::select::SelectModeActive(false),
        );

        // The actual implementation will need to access the current primitive type through a system parameter
        // For now, we'll just use the default rectangle
        let primitive_tool = base::get_primitive_tool(PrimitiveType::Rectangle);
        primitive_tool.update(commands);
    }

    fn on_enter(&self) {
        // Called when entering primitives mode - will show the sub-menu
        info!("Entered primitives mode");
    }

    fn on_exit(&self) {
        // Called when exiting primitives mode - will hide the sub-menu
        info!("Exited primitives mode");
    }
}

// Now add a system that will handle the active primitive tool
pub fn handle_active_primitive_tool(
    current_primitive_type: Res<CurrentPrimitiveType>,
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut commands: Commands,
) {
    // Only update when in primitives mode
    if current_mode.0 == crate::edit_mode_toolbar::EditMode::Primitives {
        let tool = base::get_primitive_tool(current_primitive_type.0);
        tool.update(&mut commands);
    }
}

// System to spawn the primitives sub-menu
pub fn spawn_primitives_submenu(
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
                padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),  // Use theme padding
                margin: UiRect::all(Val::ZERO),  // Set to ZERO since we're using absolute positioning
                row_gap: Val::Px(TOOLBAR_ROW_GAP),  // Use theme row gap
                ..default()
            },
            Name::new("PrimitivesSubMenu"),
            // Start as hidden until primitives mode is selected
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

// Helper function to spawn a single primitive type button
fn spawn_primitive_button(
    parent: &mut ChildBuilder,
    primitive_type: &PrimitiveType,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            margin: UiRect::all(Val::Px(TOOLBAR_ITEM_SPACING)),  // Use theme spacing
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
                        padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),  // Use theme padding
                        border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)),  // Use theme border width
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(TOOLBAR_BORDER_COLOR),  // Use theme border color
                    BorderRadius::all(Val::Px(TOOLBAR_BORDER_RADIUS)),  // Use theme border radius
                    BackgroundColor(TOOLBAR_BACKGROUND_COLOR),  // Use theme background color
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(primitive_type.get_icon().to_string()),
                        TextFont {
                            font: asset_server.load(DEFAULT_FONT_PATH),  // Use theme font path
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(TOOLBAR_ICON_COLOR),
                    ));
                });
        });
}

// System to handle primitive type selection
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
    // First, check if any button was clicked
    let mut selection_changed = false;

    {
        let interaction_query = button_queries.p0();
        for (interaction, primitive_button) in interaction_query.iter() {
            if *interaction == Interaction::Pressed {
                // Update the current primitive type
                current_type.0 = primitive_button.0;
                info!("Selected primitive type: {:?}", current_type.0);
                selection_changed = true;
            }
        }
    }

    // If a button was clicked or during startup, update all buttons
    if selection_changed {
        // Update the appearance of all buttons to reflect the current selection
        let mut all_buttons_query = button_queries.p1();

        for (
            interaction,
            mut color,
            mut border_color,
            primitive_button,
            entity,
        ) in all_buttons_query.iter_mut()
        {
            // Update button appearance based on current selection
            let is_current_type = current_type.0 == primitive_button.0;

            // Update button colors
            if is_current_type {
                *color = PRESSED_BUTTON.into();
                border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
            } else if *interaction == Interaction::Hovered {
                *color = HOVERED_BUTTON.into();
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            } else {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
            }

            // Update text color for this button
            for (parent, mut text_color) in &mut text_query {
                if parent.get() == entity {
                    text_color.0 = if is_current_type {
                        PRESSED_BUTTON_ICON_COLOR
                    } else {
                        TOOLBAR_ICON_COLOR
                    };
                }
            }
        }
    } else {
        // Just update hovered effects for buttons where interaction changed
        let mut all_buttons_query = button_queries.p1();

        for (interaction, mut color, mut border_color, primitive_button, _) in
            all_buttons_query.iter_mut()
        {
            let is_current_type = current_type.0 == primitive_button.0;

            // Only update hover effects, don't change selection state
            if !is_current_type && *interaction == Interaction::Hovered {
                *color = HOVERED_BUTTON.into();
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            } else if !is_current_type && *interaction == Interaction::None {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
            }
        }
    }
}

// System to show/hide the primitive sub-menu based on the current edit mode
pub fn toggle_primitive_submenu_visibility(
    current_edit_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut submenu_query: Query<(&mut Visibility, &Name)>,
) {
    // Find the primitives submenu by name
    for (mut visibility, name) in submenu_query.iter_mut() {
        if name.as_str() == "PrimitivesSubMenu" {
            // Show the submenu only when in primitives mode
            *visibility = if current_edit_mode.0
                == crate::edit_mode_toolbar::EditMode::Primitives
            {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
