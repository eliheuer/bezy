use bevy::prelude::*;

/// Divisions of a 2D plane.
///
/// These correspond to nine anchor points, and are used for things like
/// calculating the position of selection handles, as well as in the coordinate panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum Quadrant {
    #[default]
    Center,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

#[allow(dead_code)]
static ALL_QUADRANTS: &[Quadrant] = &[
    Quadrant::TopLeft,
    Quadrant::Top,
    Quadrant::TopRight,
    Quadrant::Left,
    Quadrant::Center,
    Quadrant::Right,
    Quadrant::BottomLeft,
    Quadrant::Bottom,
    Quadrant::BottomRight,
];

impl Quadrant {
    /// Return all `Quadrant`s, suitable for iterating.
    #[allow(dead_code)]
    pub fn all() -> &'static [Quadrant] {
        ALL_QUADRANTS
    }

    /// Return the position opposite this one; TopRight to BottomLeft, e.g.
    ///
    /// This is used when dragging a selection handle; you anchor the point
    /// opposite the selected handle.
    #[allow(dead_code)]
    pub fn inverse(self) -> Quadrant {
        self.invert_y().invert_x()
    }

    /// Return the quadrant that is horizontally opposite. This preserves the Y
    /// coordinate.
    #[allow(dead_code)]
    fn invert_y(self) -> Quadrant {
        match self {
            Quadrant::TopRight => Quadrant::BottomRight,
            Quadrant::TopLeft => Quadrant::BottomLeft,
            Quadrant::Top => Quadrant::Bottom,
            Quadrant::BottomRight => Quadrant::TopRight,
            Quadrant::BottomLeft => Quadrant::TopLeft,
            Quadrant::Bottom => Quadrant::Top,
            other => other,
        }
    }

    /// Return the quadrant that is vertically opposite. This preserves the X
    /// coordinate.
    #[allow(dead_code)]
    fn invert_x(self) -> Quadrant {
        match self {
            Quadrant::TopRight => Quadrant::TopLeft,
            Quadrant::TopLeft => Quadrant::TopRight,
            Quadrant::Left => Quadrant::Right,
            Quadrant::Right => Quadrant::Left,
            Quadrant::BottomRight => Quadrant::BottomLeft,
            Quadrant::BottomLeft => Quadrant::BottomRight,
            other => other,
        }
    }

    /// Returns `true` if this quadrant modifies the x axis of a rectangle.
    #[allow(dead_code)]
    pub fn modifies_x_axis(self) -> bool {
        !matches!(self, Quadrant::Top | Quadrant::Bottom | Quadrant::Center)
    }

    /// Returns `true` if this quadrant modifies the y axis of a rectangle.
    #[allow(dead_code)]
    pub fn modifies_y_axis(self) -> bool {
        !matches!(self, Quadrant::Left | Quadrant::Right | Quadrant::Center)
    }

    /// Returns the quadrant that contains the given point, when overlaid on a
    /// rectangle of the given size.
    #[allow(dead_code)]
    pub fn for_point_in_bounds(pt: Vec2, size: Vec2) -> Self {
        let zone_x = size.x / 3.0;
        let zone_y = size.y / 3.0;
        let mouse_x = match pt.x {
            x if x < zone_x => 0,
            x if x >= zone_x && x < zone_x * 2.0 => 1,
            x if x >= zone_x * 2.0 => 2,
            _ => unreachable!(),
        };

        let mouse_y = match pt.y {
            y if y < zone_y => 0,
            y if y >= zone_y && y < zone_y * 2.0 => 1,
            y if y >= zone_y * 2.0 => 2,
            _ => unreachable!(),
        };

        match (mouse_x, mouse_y) {
            (0, 0) => Quadrant::TopLeft,
            (1, 0) => Quadrant::Top,
            (2, 0) => Quadrant::TopRight,
            (0, 1) => Quadrant::Left,
            (1, 1) => Quadrant::Center,
            (2, 1) => Quadrant::Right,
            (0, 2) => Quadrant::BottomLeft,
            (1, 2) => Quadrant::Bottom,
            (2, 2) => Quadrant::BottomRight,
            _ => unreachable!(),
        }
    }

    /// Given a rect's bounds, return a point representing the position of this
    /// quadrant in that rect.
    #[allow(dead_code)]
    pub fn point_in_rect(self, bounds: Rect) -> Vec2 {
        let size = Vec2::new(bounds.width(), bounds.height());
        let origin = Vec2::new(bounds.min.x, bounds.min.y);

        let rel_point = match self {
            Quadrant::TopLeft => Vec2::new(0.0, 0.0),
            Quadrant::Top => Vec2::new(size.x / 2.0, 0.0),
            Quadrant::TopRight => Vec2::new(size.x, 0.0),
            Quadrant::Left => Vec2::new(0.0, size.y / 2.0),
            Quadrant::Center => Vec2::new(size.x / 2.0, size.y / 2.0),
            Quadrant::Right => Vec2::new(size.x, size.y / 2.0),
            Quadrant::BottomLeft => Vec2::new(0.0, size.y),
            Quadrant::Bottom => Vec2::new(size.x / 2.0, size.y),
            Quadrant::BottomRight => Vec2::new(size.x, size.y),
        };

        origin + rel_point
    }

    /// Given a rect, return a point representing the position of this quadrant, in the design space.
    ///
    /// This is a variant of `point_in_rect` with the coordinate system of our design space, (UL is 0,0).
    /// Our Bevy coordinate system has 0,0 at bottom-left, UI is top-left, so we need to handle the conversion.
    #[allow(dead_code)]
    pub fn point_in_design_space_rect(self, rect: Rect) -> Vec2 {
        self.invert_y().point_in_rect(rect)
    }

    /// Given a rectangle and a drag vector, return a new drag vector that scales the rectangle.
    ///
    /// The rectangle is anchored at the opposite quadrant (for resize handles),
    /// and the returned Vec2 keeps the origin fixed and scales the rectangle to include the drag point.
    #[allow(dead_code)]
    pub fn scale_design_space_rect(self, rect: Rect, drag: Vec2) -> Vec2 {
        // axis locking should have already happened
        assert_eq!(drag, self.lock_delta(drag));

        let start_point = self.point_in_design_space_rect(rect);
        let origin_point = self.inverse().point_in_design_space_rect(rect);
        let origin_delta = origin_point - start_point;
        let cur_delta = origin_point - (start_point + drag);

        compute_scale(
            Vec2::new(origin_delta.x.abs(), origin_delta.y.abs()),
            Vec2::new(cur_delta.x.abs(), cur_delta.y.abs()),
        )
    }

    /// Lock a motion delta to only one dimension, as needed for the given quadrant.
    ///
    /// For example, if the quadrant is `Top`, only the Y component of the delta is preserved.
    #[allow(dead_code)]
    pub fn lock_delta(self, delta: Vec2) -> Vec2 {
        match self {
            Quadrant::Top | Quadrant::Bottom => Vec2::new(0.0, delta.y),
            Quadrant::Left | Quadrant::Right => Vec2::new(delta.x, 0.0),
            _ => delta,
        }
    }
}

/// Calculate how much a rectangle would need to scale to have its origin at the origin point
/// and a corner at the current point
#[allow(dead_code)]
fn compute_scale(origin: Vec2, current: Vec2) -> Vec2 {
    let mut scale = Vec2::ONE;

    if origin.x != 0.0 {
        scale.x = current.x / origin.x;
    }

    if origin.y != 0.0 {
        scale.y = current.y / origin.y;
    }

    scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrant_pos() {
        let rect =
            Rect::from_corners(Vec2::new(10.0, 10.0), Vec2::new(100.0, 100.0));

        assert_eq!(
            Quadrant::BottomLeft.point_in_design_space_rect(rect),
            Vec2::new(10.0, 10.0)
        );

        assert_eq!(
            Quadrant::Center.point_in_design_space_rect(rect),
            Vec2::new(55.0, 55.0)
        );

        assert_eq!(
            Quadrant::TopRight.point_in_design_space_rect(rect),
            Vec2::new(100.0, 100.0)
        );

        assert_eq!(
            Quadrant::Top.point_in_design_space_rect(rect),
            Vec2::new(55.0, 100.0)
        );
    }
}
