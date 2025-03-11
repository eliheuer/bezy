use crate::edit_mode_toolbar::primitives::base::PrimitiveShapeTool;
use bevy::prelude::*;

/// State for the ellipse drawing tool
#[derive(Default, Debug)]
pub struct EllipsePrimitive {
    gesture_state: GestureState,
    pub shift_locked: bool,
}

/// The state of the ellipse drawing gesture
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

impl EllipsePrimitive {
    /// Get the starting and current points for the ellipse bounding rectangle
    fn rect_points(&self) -> Option<(Vec2, Vec2)> {
        match self.gesture_state {
            GestureState::Drawing { start, current } => {
                let mut end = current;

                // If shift is held, make a perfect circle
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

    /// Create a rectangle for drawing the ellipse's bounding box
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
}

impl PrimitiveShapeTool for EllipsePrimitive {
    fn name(&self) -> &'static str {
        "Ellipse"
    }

    fn update(&self, _commands: &mut Commands) {
        // Update logic will be implemented here
    }

    fn on_enter(&self) {
        info!("Ellipse tool activated");
    }

    fn on_exit(&self) {
        info!("Ellipse tool deactivated");
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
            // Create the actual ellipse shape entity
            // This will be implemented when the drawing system is ready
            info!("Created ellipse with bounding rect: {:?}", rect);
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
