//! 'Design space' is the fixed coordinate space in which we describe glyphs,
//! guides, and other entities.
//!
//! When drawing to the screen or handling mouse input, we need to translate from
//! 'screen space' to design space, taking into account things like the current
//! scroll offset and zoom level.

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use bevy::math::{Mat3, Vec2, Vec3};
use bevy::prelude::*;

/// The position of the view, relative to the design space.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ViewPort {
    /// The offset from (0, 0) in view space (the top left corner) and (0, 0) in
    /// design space, which is the intersection of the baseline and the left sidebearing.
    ///
    /// # Note:
    ///
    /// This does not account for zoom. Zoom must be applied when using this to
    /// derive a screen point.
    offset: Vec2,
    pub zoom: f32,
    /// Whether or not the y axis is inverted between view and design space.
    ///
    /// This is always `true`. It exists to make this code more readable.
    pub flipped_y: bool,
}

/// A point in design space.
///
/// This type should only be constructed through a function that has access to,
/// and takes account of, the current viewport.
#[derive(Clone, Copy, Component, PartialEq)]
pub struct DPoint {
    pub x: f32,
    pub y: f32,
}

/// A vector in design space, used for nudging & dragging
#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct DVec2 {
    pub x: f32,
    pub y: f32,
}

impl DPoint {
    #[allow(dead_code)]
    pub const ZERO: DPoint = DPoint { x: 0.0, y: 0.0 };

    /// Should only be used with inputs already in design space, such as when
    /// loaded from file.
    pub(crate) fn new(x: f32, y: f32) -> DPoint {
        assert!(
            x.is_finite()
                && y.is_finite()
                && x.fract() == 0.
                && y.fract() == 0.,
            "({}, {})",
            x,
            y
        );
        DPoint { x, y }
    }

    #[allow(dead_code)]
    pub fn from_screen(point: Vec2, vport: ViewPort) -> DPoint {
        vport.from_screen(point)
    }

    #[allow(dead_code)]
    pub fn to_screen(self, vport: ViewPort) -> Vec2 {
        vport.to_screen(self)
    }

    /// Create a new `DPoint` from a `Vec2` in design space. This should only
    /// be used to convert back to a `DPoint` after using `Vec2` to do vector
    /// math in design space.
    #[allow(dead_code)]
    pub fn from_raw(point: impl Into<Vec2>) -> DPoint {
        let point = point.into();
        DPoint::new(point.x.round(), point.y.round())
    }

    /// Convert a design point directly to a Vec2, without taking screen geometry
    /// into account.
    pub(super) fn to_raw(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Convert this `DPoint` to a `DVec2`.
    #[allow(dead_code)]
    pub fn to_dvec2(self) -> DVec2 {
        let DPoint { x, y } = self;
        DVec2 { x, y }
    }

    /// Given another point, lock whichever axis has the smallest difference
    /// between the two points to the value of that point.
    #[allow(dead_code)]
    pub(crate) fn axis_locked_to(self, other: DPoint) -> DPoint {
        let dxy = other - self;
        if dxy.x.abs() > dxy.y.abs() {
            DPoint::new(self.x, other.y)
        } else {
            DPoint::new(other.x, self.y)
        }
    }

    #[allow(dead_code)]
    pub fn lerp(self, other: DPoint, t: f32) -> DPoint {
        DPoint::from_raw(Vec2::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        ))
    }
}

impl DVec2 {
    #[allow(dead_code)]
    pub const ZERO: DVec2 = DVec2 { x: 0.0, y: 0.0 };

    fn new(x: f32, y: f32) -> DVec2 {
        assert!(
            x.is_finite()
                && y.is_finite()
                && x.fract() == 0.
                && y.fract() == 0.
        );
        DVec2 { x, y }
    }

    #[allow(dead_code)]
    pub fn from_raw(vec2: impl Into<Vec2>) -> DVec2 {
        let vec2 = vec2.into();
        DVec2::new(vec2.x.round(), vec2.y.round())
    }

    #[allow(dead_code)]
    #[inline]
    pub(super) fn to_raw(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn length(self) -> f32 {
        self.to_raw().length()
    }

    /// The vector snapped to the closest axis.
    #[allow(dead_code)]
    pub fn axis_locked(self) -> DVec2 {
        if self.x.abs() > self.y.abs() {
            DVec2::new(self.x, 0.0)
        } else {
            DVec2::new(0.0, self.y)
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn zero_x(self) -> DVec2 {
        DVec2::new(0.0, self.y)
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn zero_y(self) -> DVec2 {
        DVec2::new(self.x, 0.0)
    }
}

impl ViewPort {
    #[allow(dead_code)]
    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    #[allow(dead_code)]
    pub fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    pub fn transform_matrix(&self) -> Mat3 {
        let y_scale = if self.flipped_y {
            self.zoom
        } else {
            -self.zoom
        };
        Mat3::from_cols(
            Vec3::new(self.zoom, 0.0, 0.0),
            Vec3::new(0.0, y_scale, 0.0),
            Vec3::new(
                self.offset.x * self.zoom,
                self.offset.y * self.zoom,
                1.0,
            ),
        )
    }

    pub fn inverse_transform_matrix(&self) -> Mat3 {
        self.transform_matrix().inverse()
    }

    pub fn from_screen(&self, point: impl Into<Vec2>) -> DPoint {
        let point = point.into();
        let transformed =
            self.inverse_transform_matrix().transform_point2(point);
        DPoint::new(transformed.x.round(), transformed.y.round())
    }

    pub fn to_screen(&self, point: impl Into<DPoint>) -> Vec2 {
        let point = point.into().to_raw();
        self.transform_matrix().transform_point2(point)
    }

    // rects get special treatment because they can't be transformed with a matrix directly
    #[allow(dead_code)]
    pub fn rect_to_screen(&self, rect: Rect) -> Rect {
        let p0 = self.to_screen(DPoint::from_raw(rect.min));
        let p1 = self.to_screen(DPoint::from_raw(rect.max));
        Rect::from_corners(p0, p1)
    }

    #[allow(dead_code)]
    pub fn get_color(&self) -> Color {
        Color::srgba(1.0, 0.0, 0.0, 1.0)
    }

    #[allow(dead_code)]
    pub fn get_hover_color(&self) -> Color {
        Color::srgba(1.0, 0.0, 0.0, 1.0)
    }
}

impl Add<DVec2> for DPoint {
    type Output = DPoint;

    #[inline]
    fn add(self, other: DVec2) -> Self {
        DPoint::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub<DVec2> for DPoint {
    type Output = DPoint;

    #[inline]
    fn sub(self, other: DVec2) -> Self {
        DPoint::new(self.x - other.x, self.y - other.y)
    }
}

impl Sub<DPoint> for DPoint {
    type Output = DVec2;

    #[inline]
    fn sub(self, other: DPoint) -> DVec2 {
        DVec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Add for DVec2 {
    type Output = DVec2;

    #[inline]
    fn add(self, other: DVec2) -> DVec2 {
        DVec2::new((self.x + other.x).round(), (self.y + other.y).round())
    }
}

impl AddAssign for DVec2 {
    fn add_assign(&mut self, rhs: DVec2) {
        *self = *self + rhs
    }
}

impl Sub for DVec2 {
    type Output = DVec2;

    #[inline]
    fn sub(self, other: DVec2) -> DVec2 {
        DVec2::new(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign for DVec2 {
    fn sub_assign(&mut self, rhs: DVec2) {
        *self = *self - rhs
    }
}

impl From<(f32, f32)> for DPoint {
    fn from(src: (f32, f32)) -> DPoint {
        DPoint::new(src.0.round(), src.1.round())
    }
}

impl fmt::Debug for DPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "D({:?}, {:?})", self.x, self.y)
    }
}

impl fmt::Display for DPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "D(")?;
        fmt::Display::fmt(&self.x, formatter)?;
        write!(formatter, ", ")?;
        fmt::Display::fmt(&self.y, formatter)?;
        write!(formatter, ")")
    }
}

impl fmt::Display for DVec2 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "D𝐯=(")?;
        fmt::Display::fmt(&self.x, formatter)?;
        write!(formatter, ", ")?;
        fmt::Display::fmt(&self.y, formatter)?;
        write!(formatter, ")")
    }
}

impl Default for ViewPort {
    fn default() -> Self {
        ViewPort {
            offset: Vec2::ZERO,
            zoom: 1.0,
            flipped_y: true,
        }
    }
}

/// System that draws debug coordinate lines in the design space
pub fn debug_coordinates(mut gizmos: Gizmos) {
    use crate::theme::DEBUG_SHOW_ORIGIN_CROSS;

    // Only draw the debug cross if enabled in theme settings
    if DEBUG_SHOW_ORIGIN_CROSS {
        // Draw a cross at (0,0)
        gizmos.line_2d(
            Vec2::new(-10.0, 0.0),
            Vec2::new(10.0, 0.0),
            Color::srgba(1.0, 0.0, 0.0, 1.0),
        );
        gizmos.line_2d(
            Vec2::new(0.0, -10.0),
            Vec2::new(0.0, 10.0),
            Color::srgba(1.0, 0.0, 0.0, 1.0),
        );
    }
}

/// Plugin that sets up the design space systems
pub struct DesignSpacePlugin;

impl Plugin for DesignSpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_coordinates);
    }
}
