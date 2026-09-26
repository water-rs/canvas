//! Type conversions between WaterUI layout types and Cherenkov's kurbo types.

use cherenkov::kurbo;
use waterui_core::layout::{Point, Rect, Size};

// ============================================================================
// Point conversions
// ============================================================================

#[inline]
pub fn point_to_kurbo(p: Point) -> kurbo::Point {
    kurbo::Point::new(f64::from(p.x), f64::from(p.y))
}

// ============================================================================
// Size conversions
// ============================================================================

#[inline]
pub fn size_to_kurbo(s: Size) -> kurbo::Size {
    kurbo::Size::new(f64::from(s.width), f64::from(s.height))
}

// ============================================================================
// Rect conversions
// ============================================================================

#[inline]
pub fn rect_to_kurbo(r: Rect) -> kurbo::Rect {
    let origin = point_to_kurbo(r.origin());
    let size = size_to_kurbo(*r.size());
    kurbo::Rect::from_origin_size(origin, size)
}

#[inline]
pub const fn kurbo_to_rect(r: kurbo::Rect) -> Rect {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "canvas coordinates are screen-scale magnitudes that f32 carries exactly enough to draw with"
    )]
    let (x, y, width, height) = (
        r.x0 as f32,
        r.y0 as f32,
        r.width() as f32,
        r.height() as f32,
    );
    Rect::new(Point::new(x, y), Size::new(width, height))
}
