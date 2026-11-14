//! Gradient builders for Canvas.
//!
//! This module provides HTML5 Canvas-style gradient builders whose colour
//! stops are `waterui_graphics::Color` values resolved against the view's
//! environment at draw time.

use std::rc::Rc;

use nami::watcher::{Context, WatcherGuard};
use nami::{Computed, Signal};
use waterui_core::layout::Point;
use waterui_graphics::Color;
use waterui_graphics::draw::{self, Paint, WorkingColor};

use crate::{DrawingContext, StylePaint};

/// A paint signal that rebuilds a gradient when any stop's colour changes.
///
/// The recorder's paint operand takes the whole `Paint` as one live value —
/// the record layer names no per-stop operand — so a gradient whose stops are signals
/// rides as a `Signal<Output = Paint>` and a stop change lands as an operand
/// update, not a scene re-record.
#[derive(Clone)]
struct LiveStops<F> {
    stops: Vec<(f32, Computed<WorkingColor>)>,
    build: F,
}

impl<F> Signal for LiveStops<F>
where
    F: Fn(Vec<(f32, WorkingColor)>) -> Paint + Clone + 'static,
{
    type Output = Paint;
    type Guard = StopsGuard;

    fn snapshot(&self) -> Paint {
        let colors = self
            .stops
            .iter()
            .map(|(offset, color)| (*offset, color.snapshot()))
            .collect::<Vec<_>>();
        (self.build)(colors)
    }

    fn watch(&self, watcher: impl Fn(Context<Paint>) + 'static) -> StopsGuard {
        let watcher = Rc::new(watcher);
        let guards = self
            .stops
            .iter()
            .map(|(_, color)| {
                let signal = self.clone();
                let watcher = Rc::clone(&watcher);
                color.watch(move |_| watcher(Context::from(signal.snapshot())))
            })
            .collect();
        StopsGuard(guards)
    }
}

/// Keeps every stop subscription alive until the operand drops it.
struct StopsGuard(
    #[expect(dead_code, reason = "dropping the guard releases the subscriptions")]
    Vec<nami::watcher::BoxWatcherGuard>,
);
impl WatcherGuard for StopsGuard {}

/// Resolves every stop's colour to its signal and returns a paint signal that
/// rebuilds the gradient whenever one of them changes.
fn live_stops<G>(
    ctx: &mut DrawingContext,
    stops: &[ColorStop],
    build: impl Fn(Vec<(f32, WorkingColor)>) -> G + Clone + 'static,
) -> Computed<Paint>
where
    G: Into<Paint> + 'static,
{
    let stops = stops
        .iter()
        .map(|stop| (stop.offset, ctx.resolve_color_signal(&stop.color)))
        .collect();
    Computed::new(LiveStops {
        stops,
        build: move |colors| build(colors).into(),
    })
}

/// A color stop in a gradient.
///
/// This represents a color at a specific position along the gradient.
#[derive(Debug, Clone)]
pub struct ColorStop {
    /// Position along the gradient (0.0 to 1.0).
    pub offset: f32,
    /// Color at this position.
    pub color: Color,
}

impl ColorStop {
    /// Creates a new color stop.
    #[must_use]
    pub fn new(offset: f32, color: impl Into<Color>) -> Self {
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
    pub fn add_color_stop(&mut self, offset: f32, color: impl Into<Color>) {
        self.stops.push(ColorStop::new(offset, color));
    }

    /// Resolves the gradient stops into a live engine paint.
    pub(crate) fn build(&self, ctx: &mut DrawingContext) -> StylePaint {
        let start = super::conversions::point_to_kurbo(self.start);
        let end = super::conversions::point_to_kurbo(self.end);
        StylePaint::Signal(live_stops(ctx, &self.stops, move |colors| {
            let mut gradient = draw::LinearGradient::new(start, end);
            for (offset, color) in colors {
                gradient = gradient.stop(offset, color);
            }
            gradient
        }))
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
    pub fn add_color_stop(&mut self, offset: f32, color: impl Into<Color>) {
        self.stops.push(ColorStop::new(offset, color));
    }

    /// Resolves the gradient stops into a live engine paint.
    pub(crate) fn build(&self, ctx: &mut DrawingContext) -> StylePaint {
        let center0 = super::conversions::point_to_kurbo(self.center0);
        let radius0 = f64::from(self.radius0);
        let center1 = super::conversions::point_to_kurbo(self.center1);
        let radius1 = f64::from(self.radius1);
        StylePaint::Signal(live_stops(ctx, &self.stops, move |colors| {
            let mut gradient = draw::RadialGradient::two_point(center0, radius0, center1, radius1);
            for (offset, color) in colors {
                gradient = gradient.stop(offset, color);
            }
            gradient
        }))
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
    pub fn add_color_stop(&mut self, offset: f32, color: impl Into<Color>) {
        self.stops.push(ColorStop::new(offset, color));
    }

    /// Resolves the gradient stops into a live engine paint.
    pub(crate) fn build(&self, ctx: &mut DrawingContext) -> StylePaint {
        let center = super::conversions::point_to_kurbo(self.center);
        let start_angle = f64::from(self.start_angle);
        StylePaint::Signal(live_stops(ctx, &self.stops, move |colors| {
            // Conic stops sweep a full turn from `start_angle`.
            let mut gradient =
                draw::SweepGradient::new(center, start_angle, start_angle + core::f64::consts::TAU);
            for (offset, color) in colors {
                gradient = gradient.stop(offset, color);
            }
            gradient
        }))
    }
}
