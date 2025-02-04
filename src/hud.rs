use crate::theme::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct PressedButtonText;

pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
            }
        }
    }
}
