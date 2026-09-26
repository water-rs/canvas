//! Gradient builders for Canvas.
//!
//! This module provides HTML5 Canvas-style gradient builders that use
//! WaterUI's `WorkingColor` type.

use waterui_core::layout::Point;
use waterui_graphics::WorkingColor;

// Internal imports for rendering
use cherenkov::Paint;

use super::conversions::point_to_kurbo;

/// A color stop in a gradient.
///
/// This represents a color at a specific position along the gradient.
#[derive(Debug, Clone)]
pub struct ColorStop {
    /// Position along the gradient (0.0 to 1.0).
    pub offset: f32,
    /// Color at this position.
    pub color: WorkingColor,
}

impl ColorStop {
    /// Creates a new color stop.
    #[must_use]
    pub fn new(offset: f32, color: impl Into<WorkingColor>) -> Self {
        Self {
            offset,
            color: color.into(),
        }
    }
}

/// Linear gradient builder.
///
/// Creates a gradient that transitions colors along a straight line
/// from a start point to an end point.
///
/// # Example
///
/// ```rust
/// # use waterui_canvas::DrawingContext;
/// # use waterui_graphics::color::Srgb;
/// # fn draw(ctx: &mut DrawingContext<'_>) {
/// let mut gradient = ctx.create_linear_gradient(0.0, 0.0, 100.0, 100.0);
/// gradient.add_color_stop(0.0, Srgb::new(1.0, 0.0, 0.0));
/// gradient.add_color_stop(1.0, Srgb::new(0.0, 0.0, 1.0));
/// ctx.set_fill_style(gradient);
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct LinearGradient {
    start: Point,
    end: Point,
    stops: Vec<ColorStop>,
}

impl LinearGradient {
    /// Creates a new linear gradient from (x0, y0) to (x1, y1).
    #[must_use]
    pub const fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            start: Point::new(x0, y0),
            end: Point::new(x1, y1),
            stops: Vec::new(),
        }
    }

    /// Adds a color stop to the gradient.
    ///
    /// # Arguments
    /// * `offset` - Position (0.0 to 1.0) along the gradient
    /// * `color` - Color at this position
    pub fn add_color_stop(&mut self, offset: f32, color: impl Into<WorkingColor>) {
        self.stops.push(ColorStop::new(offset, color));
    }

    /// Builds the gradient into a Cherenkov paint.
    #[must_use]
    pub(crate) fn build(&self) -> Paint {
        let mut gradient =
            cherenkov::LinearGradient::new(point_to_kurbo(self.start), point_to_kurbo(self.end));
        gradient.stops = cherenkov_stops(&self.stops);
        Paint::Linear(gradient)
    }
}

/// Radial gradient builder.
///
/// Creates a gradient that transitions colors radially from one circle to another.
///
/// # Example
///
/// ```rust
/// # use waterui_canvas::DrawingContext;
/// # use waterui_graphics::color::Srgb;
/// # fn draw(ctx: &mut DrawingContext<'_>) {
/// let mut gradient = ctx.create_radial_gradient(50.0, 50.0, 10.0, 50.0, 50.0, 50.0);
/// gradient.add_color_stop(0.0, Srgb::new(1.0, 1.0, 1.0));
/// gradient.add_color_stop(1.0, Srgb::new(0.0, 0.0, 0.0));
/// ctx.set_fill_style(gradient);
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RadialGradient {
    center0: Point,
    radius0: f32,
    center1: Point,
    radius1: f32,
    stops: Vec<ColorStop>,
}

impl RadialGradient {
    /// Creates a new radial gradient.
    ///
    /// # Arguments
    /// * `x0, y0` - Center of the start circle
    /// * `r0` - Radius of the start circle
    /// * `x1, y1` - Center of the end circle
    /// * `r1` - Radius of the end circle
    #[must_use]
    pub const fn new(x0: f32, y0: f32, r0: f32, x1: f32, y1: f32, r1: f32) -> Self {
        Self {
            center0: Point::new(x0, y0),
            radius0: r0,
            center1: Point::new(x1, y1),
            radius1: r1,
            stops: Vec::new(),
        }
    }

    /// Adds a color stop to the gradient.
    pub fn add_color_stop(&mut self, offset: f32, color: impl Into<WorkingColor>) {
        self.stops.push(ColorStop::new(offset, color));
    }

    /// Builds the gradient into a Cherenkov paint.
    #[must_use]
    pub(crate) fn build(&self) -> Paint {
        let mut gradient = cherenkov::RadialGradient::two_point(
            point_to_kurbo(self.center0),
            f64::from(self.radius0),
            point_to_kurbo(self.center1),
            f64::from(self.radius1),
        );
        gradient.stops = cherenkov_stops(&self.stops);
        Paint::Radial(gradient)
    }
}

/// Conic (sweep) gradient builder.
///
/// Creates a gradient that transitions colors in a circular sweep around a center point.
///
/// # Example
///
/// ```rust
/// # use waterui_canvas::DrawingContext;
/// # use waterui_graphics::color::Srgb;
/// # fn draw(ctx: &mut DrawingContext<'_>) {
/// let mut gradient = ctx.create_conic_gradient(0.0, 50.0, 50.0);
/// gradient.add_color_stop(0.0, Srgb::new(1.0, 0.0, 0.0));
/// gradient.add_color_stop(0.5, Srgb::new(0.0, 1.0, 0.0));
/// gradient.add_color_stop(1.0, Srgb::new(0.0, 0.0, 1.0));
/// ctx.set_fill_style(gradient);
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ConicGradient {
    center: Point,
    start_angle: f32,
    stops: Vec<ColorStop>,
}

impl ConicGradient {
    /// Creates a new conic gradient.
    ///
    /// # Arguments
    /// * `start_angle` - Starting angle in radians
    /// * `x, y` - Center point
    #[must_use]
    pub const fn new(start_angle: f32, x: f32, y: f32) -> Self {
        Self {
            center: Point::new(x, y),
            start_angle,
            stops: Vec::new(),
        }
    }

    /// Adds a color stop to the gradient.
    pub fn add_color_stop(&mut self, offset: f32, color: impl Into<WorkingColor>) {
        self.stops.push(ColorStop::new(offset, color));
    }

    /// Builds the gradient into a Cherenkov paint: one full turn from
    /// `start_angle`.
    #[must_use]
    pub(crate) fn build(&self) -> Paint {
        let start = f64::from(self.start_angle);
        let mut gradient = cherenkov::SweepGradient::new(
            point_to_kurbo(self.center),
            start,
            start + core::f64::consts::TAU,
        );
        gradient.stops = cherenkov_stops(&self.stops);
        Paint::Sweep(gradient)
    }
}

fn cherenkov_stops(stops: &[ColorStop]) -> Vec<cherenkov::ColorStop> {
    stops
        .iter()
        .map(|stop| cherenkov::ColorStop {
            offset: stop.offset,
            color: stop.color,
        })
        .collect()
}
