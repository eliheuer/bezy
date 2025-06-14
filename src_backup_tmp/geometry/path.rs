//! Path conversion utilities
//!
//! This module handles converting between different path formats used in font
//! development. Specifically, it converts from kurbo paths (used for vector
//! graphics) to norad contours (used in UFO font files).
//!
//! # What are paths and contours?
//! - A **path** is like drawing with a pen - it has moves, lines, and curves
//! - A **contour** is how font files store those shapes as a series of points
//!
//! # Why convert?
//! Different libraries use different formats to represent the same shapes.
//! This module bridges that gap so we can work with paths from graphics
//! libraries and save them in standard font file formats.

use norad::{Contour, ContourPoint, PointType};

/// Converts a kurbo path to a norad contour for UFO font files
///
/// This function takes a vector path (like you'd draw in a graphics program)
/// and converts it to the point format that UFO font files understand.
#[allow(dead_code)]
pub fn bezpath_to_contour(
    path: &kurbo::BezPath,
) -> Result<Contour, &'static str> {
    use kurbo::PathEl;

    // Set up our data structures for the conversion
    let mut points = Vec::new(); // Will store all the converted points
    let mut current_pos = None; // Track where our "pen" is currently located

    // Process each drawing command in the path
    for element in path.elements() {
        match element {
            // Move to a new position without drawing a line
            PathEl::MoveTo(point) => {
                current_pos = Some(point);
                points.push(make_point(point, PointType::Move, false));
            }

            // Draw a straight line from current position to new point
            PathEl::LineTo(point) => {
                current_pos = Some(point);
                points.push(make_point(point, PointType::Line, false));
            }

            // Draw a quadratic curve (needs conversion to cubic for UFO)
            PathEl::QuadTo(control, end) => {
                let start = current_pos
                    .ok_or("Cannot draw curve without starting point")?;
                add_quadratic_curve(&mut points, *start, *control, *end)?;
                current_pos = Some(end);
            }

            // Draw a cubic curve (UFO's native curve type)
            PathEl::CurveTo(control1, control2, end) => {
                add_cubic_curve(&mut points, *control1, *control2, *end);
                current_pos = Some(end);
            }

            // Close the path by connecting back to the start
            PathEl::ClosePath => {
                // UFO format handles path closing automatically
            }
        }
    }

    // Create and return the final contour with all our converted points
    Ok(Contour::new(points, None, None))
}

/// Converts a quadratic curve to cubic curve points
///
/// UFO fonts use cubic curves, so we need to convert quadratic curves.
/// This uses the standard mathematical conversion formula.
#[allow(dead_code)]
fn add_quadratic_curve(
    points: &mut Vec<ContourPoint>,
    start: kurbo::Point,
    control: kurbo::Point,
    end: kurbo::Point,
) -> Result<(), &'static str> {
    // Convert quadratic to cubic using the 2/3 rule:
    // - First control point is 2/3 of the way from start to quad control
    // - Second control point is 2/3 of the way from end to quad control

    let control1 = kurbo::Point::new(
        start.x + 2.0 / 3.0 * (control.x - start.x),
        start.y + 2.0 / 3.0 * (control.y - start.y),
    );

    let control2 = kurbo::Point::new(
        end.x + 2.0 / 3.0 * (control.x - end.x),
        end.y + 2.0 / 3.0 * (control.y - end.y),
    );

    // Add the three points that make up a cubic curve
    points.extend([
        make_point(&control1, PointType::OffCurve, false),
        make_point(&control2, PointType::OffCurve, false),
        make_point(&end, PointType::Curve, true),
    ]);

    Ok(())
}

/// Adds a cubic curve to the points list
#[allow(dead_code)]
fn add_cubic_curve(
    points: &mut Vec<ContourPoint>,
    control1: kurbo::Point,
    control2: kurbo::Point,
    end: kurbo::Point,
) {
    points.extend([
        make_point(&control1, PointType::OffCurve, false),
        make_point(&control2, PointType::OffCurve, false),
        make_point(&end, PointType::Curve, true),
    ]);
}

/// Creates a contour point from a kurbo point
#[allow(dead_code)]
fn make_point(
    point: &kurbo::Point,
    point_type: PointType,
    smooth: bool,
) -> ContourPoint {
    ContourPoint::new(
        point.x as f32,
        point.y as f32,
        point_type,
        smooth,
        None, // name (optional)
        None, // identifier (optional)
        None, // lib (optional metadata)
    )
}
