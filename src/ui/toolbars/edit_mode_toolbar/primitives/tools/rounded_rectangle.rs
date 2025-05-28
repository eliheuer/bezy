use crate::ui::toolbars::edit_mode_toolbar::primitives::base::PrimitiveShapeTool;
use bevy::prelude::*;

/// State for the rounded rectangle drawing tool
#[derive(Debug)]
pub struct RoundedRectanglePrimitive {
    gesture_state: GestureState,
    pub shift_locked: bool,
    pub corner_radius: i32,
}

/// The state of the rounded rectangle drawing gesture
#[derive(Default, Debug)]
enum GestureState {
    #[default]
    Ready,
    Down(Vec2),
    Drawing {
        start: Vec2,
        current: Vec2,
    },
    Finished,
}

impl RoundedRectanglePrimitive {
    /// Get the starting and current points for the rectangle
    fn rect_points(&self) -> Option<(Vec2, Vec2)> {
        match self.gesture_state {
            GestureState::Drawing { start, current } => {
                let mut end = current;

                // If shift is held, make a perfect square
                if self.shift_locked {
                    let width = (current.x - start.x).abs();
                    let height = (current.y - start.y).abs();
                    let size = width.max(height);

                    end.x = start.x + size * (current.x - start.x).signum();
                    end.y = start.y + size * (current.y - start.y).signum();
                }

                Some((start, end))
            }
            _ => None,
        }
    }

    /// Create a rectangle for drawing
    fn current_rect(&self) -> Option<Rect> {
        self.rect_points().map(|(start, end)| {
            let min_x = start.x.min(end.x);
            let min_y = start.y.min(end.y);
            let max_x = start.x.max(end.x);
            let max_y = start.y.max(end.y);

            Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            }
        })
    }

    /// Get the corner radius in a reasonable range
    #[allow(dead_code)]
    fn get_adjusted_radius(&self, rect: &Rect) -> f32 {
        // Get the minimum dimension of the rectangle
        let min_dimension = rect.width().min(rect.height());

        // Ensure the radius doesn't exceed half of the smallest dimension
        (self.corner_radius as f32).min(min_dimension / 2.0)
    }
}

impl PrimitiveShapeTool for RoundedRectanglePrimitive {
    fn name(&self) -> &'static str {
        "RoundedRectangle"
    }

    fn update(&self, _commands: &mut Commands) {
        // Update logic will be implemented here
    }

    fn on_enter(&self) {
        info!("Rounded Rectangle tool activated");
    }

    fn on_exit(&self) {
        info!("Rounded Rectangle tool deactivated");
    }

    fn begin_draw(&mut self, position: Vec2) {
        self.gesture_state = GestureState::Down(position);
    }

    fn update_draw(&mut self, position: Vec2) {
        if let GestureState::Down(start) = self.gesture_state {
            self.gesture_state = GestureState::Drawing {
                start,
                current: position,
            };
        } else if let GestureState::Drawing { start, .. } = self.gesture_state {
            self.gesture_state = GestureState::Drawing {
                start,
                current: position,
            };
        }
    }

    fn end_draw(&mut self, _position: Vec2) {
        if let Some(rect) = self.current_rect() {
            // Log the rounded rectangle creation
            info!(
                "Rounded Rectangle drawing completed: {:?}, radius: {}",
                rect, self.corner_radius
            );
        }

        self.gesture_state = GestureState::Finished;
    }

    fn cancel_draw(&mut self) {
        self.gesture_state = GestureState::Ready;
    }

    fn set_shift_locked(&mut self, locked: bool) {
        self.shift_locked = locked;
    }
}

impl Default for RoundedRectanglePrimitive {
    fn default() -> Self {
        Self {
            gesture_state: GestureState::Ready,
            shift_locked: false,
            corner_radius: 32, // Default corner radius
        }
    }
}
