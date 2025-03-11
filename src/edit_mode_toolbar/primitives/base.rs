use crate::edit_mode_toolbar::primitives::tools::ellipse::EllipsePrimitive;
use crate::edit_mode_toolbar::primitives::tools::rectangle::RectanglePrimitive;
use crate::edit_mode_toolbar::CurrentPrimitiveType;
use crate::edit_mode_toolbar::PrimitiveType;
use bevy::prelude::*;

// Common components and systems for primitives
#[derive(Component)]
pub struct PrimitiveShape;

// Common trait for all primitive shapes
pub trait PrimitiveShapeTool: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn update(&self, commands: &mut Commands);
    fn on_enter(&self);
    fn on_exit(&self);

    // Drawing methods
    fn begin_draw(&mut self, position: Vec2);
    fn update_draw(&mut self, position: Vec2);
    fn end_draw(&mut self, position: Vec2);
    fn cancel_draw(&mut self);

    // Method to access shift_locked property
    fn set_shift_locked(&mut self, locked: bool);
}

// Active drawing state for primitives
#[derive(Resource)]
pub struct ActivePrimitiveDrawing {
    pub is_drawing: bool,
    pub tool_type: PrimitiveType,
    pub start_position: Option<Vec2>,
    pub current_position: Option<Vec2>,
}

impl Default for ActivePrimitiveDrawing {
    fn default() -> Self {
        Self {
            is_drawing: false,
            tool_type: PrimitiveType::Rectangle,
            start_position: None,
            current_position: None,
        }
    }
}

// Systems for handling primitive shape drawing
pub fn handle_primitive_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    current_primitive_type: Res<CurrentPrimitiveType>,
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut active_drawing: ResMut<ActivePrimitiveDrawing>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    // Only handle events when in primitives mode
    if current_mode.0 != crate::edit_mode_toolbar::EditMode::Primitives {
        return;
    }

    // Get the current window and cursor position
    // Find the primary camera or any camera that is the main 2D camera
    let camera_entity = camera_q.iter().find(|(camera, _)| {
        // Just use any active camera for now
        camera.is_active
    });

    // Get the main window
    let window = match windows.get_single() {
        Ok(window) => window,
        Err(e) => {
            warn!("Could not get window for primitive drawing: {}", e);
            return;
        }
    };

    // Only proceed if we found a valid camera
    let Some((camera, camera_transform)) = camera_entity else {
        warn!("No primary camera found for primitive drawing");
        return;
    };

    // Handle cursor movement
    for _cursor_moved in cursor_moved_events.read() {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            let world_position = match camera
                .viewport_to_world_2d(camera_transform, cursor_pos)
            {
                Ok(pos) => pos,
                Err(e) => {
                    debug!(
                        "Error converting cursor to world position: {:?}",
                        e
                    );
                    continue;
                }
            };

            active_drawing.current_position = Some(world_position);

            // If we're already drawing, update the current drawing
            if active_drawing.is_drawing {
                let mut tool = get_primitive_tool(current_primitive_type.0);
                tool.update_draw(world_position);
            }
        }
    }

    // Handle mouse button input
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = active_drawing.current_position {
            // Start a new drawing
            active_drawing.is_drawing = true;
            active_drawing.tool_type = current_primitive_type.0;
            active_drawing.start_position = Some(cursor_pos);

            // Create and initialize the appropriate primitive tool
            let mut tool = get_primitive_tool(current_primitive_type.0);
            tool.begin_draw(cursor_pos);
            debug!(
                "Started drawing primitive: {:?} at {:?}",
                active_drawing.tool_type, cursor_pos
            );
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        if active_drawing.is_drawing {
            if let Some(cursor_pos) = active_drawing.current_position {
                // Finish the drawing
                let mut tool = get_primitive_tool(active_drawing.tool_type);
                tool.end_draw(cursor_pos);
                debug!(
                    "Finished drawing primitive: {:?}",
                    active_drawing.tool_type
                );
            } else {
                debug!("Mouse released but no current position available");
            }

            // Reset the drawing state
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
        }
    }

    // Handle keyboard input for canceling drawing
    if keyboard.just_pressed(KeyCode::Escape) {
        if active_drawing.is_drawing {
            // Cancel the drawing
            let mut tool = get_primitive_tool(active_drawing.tool_type);
            tool.cancel_draw();

            // Reset the drawing state
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
        }
    }

    // Handle shift key for constraining shapes
    if keyboard.just_pressed(KeyCode::ShiftLeft)
        || keyboard.just_pressed(KeyCode::ShiftRight)
    {
        if active_drawing.is_drawing {
            // Get the active primitive tool and toggle shift lock
            let mut tool = get_primitive_tool(active_drawing.tool_type);
            tool.set_shift_locked(true);
        }
    } else if keyboard.just_released(KeyCode::ShiftLeft)
        || keyboard.just_released(KeyCode::ShiftRight)
    {
        if active_drawing.is_drawing {
            // Get the active primitive tool and toggle shift lock off
            let mut tool = get_primitive_tool(active_drawing.tool_type);
            tool.set_shift_locked(false);
        }
    }
}

// Trait object utilities for downcasting
pub trait AnyDowncast {
    fn downcast_ref<T: 'static>(&self) -> Option<&T>;
    fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T>;
}

impl<'a> dyn PrimitiveShapeTool + 'a {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        // This is unsafe because we're bypassing the compiler's type checking
        // It's safe only if we correctly check the type with TypeId
        unsafe {
            let type_id = std::any::TypeId::of::<T>();

            if self.type_id() == type_id {
                Some(&*(self as *const dyn PrimitiveShapeTool as *const T))
            } else {
                None
            }
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        // This is unsafe for the same reasons as above
        unsafe {
            let type_id = std::any::TypeId::of::<T>();

            if self.type_id() == type_id {
                Some(&mut *(self as *mut dyn PrimitiveShapeTool as *mut T))
            } else {
                None
            }
        }
    }

    fn type_id(&self) -> std::any::TypeId {
        // Use the name as a proxy for type_id (this is just a demonstration)
        // In a real implementation, you'd need proper type reflection
        match self.name() {
            "Rectangle" => std::any::TypeId::of::<RectanglePrimitive>(),
            "Ellipse" => std::any::TypeId::of::<EllipsePrimitive>(),
            _ => std::any::TypeId::of::<()>(), // Default for unknown types
        }
    }
}

// Util function to get current tool based on primitive type
pub fn get_primitive_tool(
    primitive_type: PrimitiveType,
) -> Box<dyn PrimitiveShapeTool> {
    match primitive_type {
        PrimitiveType::Rectangle => Box::new(RectanglePrimitive::default()),
        PrimitiveType::Ellipse => Box::new(EllipsePrimitive::default()),
    }
}
