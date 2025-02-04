use bevy::prelude::*;
use crate::theme::*;

#[derive(Component)]
pub struct PressedButtonText;

pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    button_text_query: Query<&Text, Without<PressedButtonText>>,
    mut center_text_query: Query<&mut Text, With<PressedButtonText>>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let button_text = button_text_query.get(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if let Ok(mut center_text) = center_text_query.get_single_mut() {
                    *center_text = button_text.clone();
                }
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
        }
    }
}
