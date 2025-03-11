use norad::{Contour, ContourPoint, PointType};

/// Convert a kurbo::BezPath to a norad::Contour
pub fn bezpath_to_contour(path: &kurbo::BezPath) -> Result<norad::Contour, &'static str> {
    use kurbo::PathEl;
    
    let mut points = Vec::new();
    let mut current_point = None;
    
    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => {
                current_point = Some(p);
                points.push(create_point(p.x as f32, p.y as f32, PointType::Move, false));
            }
            PathEl::LineTo(p) => {
                current_point = Some(p);
                points.push(create_point(p.x as f32, p.y as f32, PointType::Line, false));
            }
            PathEl::QuadTo(p1, p2) => {
                // Convert quadratic bezier to cubic (not ideal but works for now)
                if let Some(p0) = current_point {
                    let cp1 = kurbo::Point::new(
                        p0.x + 2.0/3.0 * (p1.x - p0.x),
                        p0.y + 2.0/3.0 * (p1.y - p0.y),
                    );
                    let cp2 = kurbo::Point::new(
                        p2.x + 2.0/3.0 * (p1.x - p2.x),
                        p2.y + 2.0/3.0 * (p1.y - p2.y),
                    );
                    
                    points.push(create_point(cp1.x as f32, cp1.y as f32, PointType::OffCurve, false));
                    points.push(create_point(cp2.x as f32, cp2.y as f32, PointType::OffCurve, false));
                    points.push(create_point(p2.x as f32, p2.y as f32, PointType::Curve, true));
                    
                    current_point = Some(p2);
                } else {
                    return Err("QuadTo without a current point");
                }
            }
            PathEl::CurveTo(p1, p2, p3) => {
                points.push(create_point(p1.x as f32, p1.y as f32, PointType::OffCurve, false));
                points.push(create_point(p2.x as f32, p2.y as f32, PointType::OffCurve, false));
                points.push(create_point(p3.x as f32, p3.y as f32, PointType::Curve, true));
                
                current_point = Some(p3);
            }
            PathEl::ClosePath => {
                // No need to add a point for close path
            }
        }
    }
    
    // Create the contour with the points
    let contour = Contour::new(points, None, None);
    
    Ok(contour)
}

/// Helper function to create a ContourPoint
fn create_point(x: f32, y: f32, typ: PointType, smooth: bool) -> ContourPoint {
    ContourPoint::new(x, y, typ, smooth, None, None, None)
} 