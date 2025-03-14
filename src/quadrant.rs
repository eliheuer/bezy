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
    pub fn all() -> &'static [Quadrant] {
        ALL_QUADRANTS
    }

    /// Return the position opposite this one; TopRight to BottomLeft, e.g.
    ///
    /// This is used when dragging a selection handle; you anchor the point
    /// opposite the selected handle.
    pub fn inverse(self) -> Quadrant {
        self.invert_y().invert_x()
    }

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

    /// Returns true if this quadrant affects the x axis when modified
    pub fn modifies_x_axis(self) -> bool {
        !matches!(self, Quadrant::Top | Quadrant::Bottom | Quadrant::Center)
    }

    /// Returns true if this quadrant affects the y axis when modified
    pub fn modifies_y_axis(self) -> bool {
        !matches!(self, Quadrant::Left | Quadrant::Right | Quadrant::Center)
    }

    /// Given a point and a size, return the quadrant containing that point.
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

    /// Given a bounds, return the point corresponding to this quadrant.
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

    /// Given a rect in *design space* (that is, y-up), return the point
    /// corresponding to this quadrant.
    pub fn point_in_design_space_rect(self, rect: Rect) -> Vec2 {
        self.invert_y().point_in_rect(rect)
    }

    /// Return the x&y suitable for transforming `rect` given a drag
    /// originating at this quadrant.
    ///
    /// This can be negative in either direction if the drag crosses the
    /// opposite quadrant point.
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

    /// When dragging from a control handle, side handles lock an axis.
    pub fn lock_delta(self, delta: Vec2) -> Vec2 {
        match self {
            Quadrant::Top | Quadrant::Bottom => Vec2::new(0.0, delta.y),
            Quadrant::Left | Quadrant::Right => Vec2::new(delta.x, 0.0),
            _ => delta,
        }
    }
}

/// Compute scale vectors based on original and current sizes.
/// Returns a Vec2 where x is the x-scale factor and y is the y-scale factor.
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
