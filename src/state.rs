//! Canvas drawing state management.
//!
//! This module provides the state stack for HTML5 Canvas-style save/restore operations.

use waterui_graphics::Color;
use waterui_graphics::color::Srgb;

// Internal imports for rendering (not exposed to users)
use kurbo;
use waterui_graphics::draw::BlendMode;

use super::gradient::{ConicGradient, LinearGradient, RadialGradient};
use super::text::FontSpec;

// ============================================================================
// Fill and Stroke Styles
// ============================================================================

/// Fill rule used to determine the interior of self-intersecting paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FillRule {
    /// Non-zero winding rule (default).
    #[default]
    NonZero,
    /// Even-odd rule.
    EvenOdd,
}

/// Fill style for shapes - can be a solid color or gradient.
#[derive(Debug, Clone)]
pub enum FillStyle {
    /// Solid color fill.
    Color(Color),
    /// Linear gradient fill.
    LinearGradient(LinearGradient),
    /// Radial gradient fill.
    RadialGradient(RadialGradient),
    /// Conic (sweep) gradient fill.
    ConicGradient(ConicGradient),
}

impl<T> From<T> for FillStyle
where
    T: Into<Color>,
{
    fn from(color: T) -> Self {
        Self::Color(color.into())
    }
}

impl From<LinearGradient> for FillStyle {
    fn from(gradient: LinearGradient) -> Self {
        Self::LinearGradient(gradient)
    }
}

impl From<RadialGradient> for FillStyle {
    fn from(gradient: RadialGradient) -> Self {
        Self::RadialGradient(gradient)
    }
}

impl From<ConicGradient> for FillStyle {
    fn from(gradient: ConicGradient) -> Self {
        Self::ConicGradient(gradient)
    }
}

/// Stroke style for shapes - can be a solid color or gradient.
#[derive(Debug, Clone)]
pub enum StrokeStyle {
    /// Solid color stroke.
    Color(Color),
    /// Linear gradient stroke.
    LinearGradient(LinearGradient),
    /// Radial gradient stroke.
    RadialGradient(RadialGradient),
    /// Conic (sweep) gradient stroke.
    ConicGradient(ConicGradient),
}

impl<T> From<T> for StrokeStyle
where
    T: Into<Color>,
{
    fn from(color: T) -> Self {
        Self::Color(color.into())
    }
}

impl From<LinearGradient> for StrokeStyle {
    fn from(gradient: LinearGradient) -> Self {
        Self::LinearGradient(gradient)
    }
}

impl From<RadialGradient> for StrokeStyle {
    fn from(gradient: RadialGradient) -> Self {
        Self::RadialGradient(gradient)
    }
}

impl From<ConicGradient> for StrokeStyle {
    fn from(gradient: ConicGradient) -> Self {
        Self::ConicGradient(gradient)
    }
}

// ============================================================================
// Line Styling
// ============================================================================

/// Line cap style (end of strokes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineCap {
    /// Flat edge at end of line (default).
    #[default]
    Butt,
    /// Rounded end.
    Round,
    /// Square end extending beyond the endpoint.
    Square,
}

impl LineCap {
    pub(crate) const fn to_kurbo(self) -> kurbo::Cap {
        match self {
            Self::Butt => kurbo::Cap::Butt,
            Self::Round => kurbo::Cap::Round,
            Self::Square => kurbo::Cap::Square,
        }
    }
}

/// Line join style (corners).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineJoin {
    /// Miter join (sharp corner, default).
    #[default]
    Miter,
    /// Rounded corner.
    Round,
    /// Beveled corner (flattened).
    Bevel,
}

impl LineJoin {
    pub(crate) const fn to_kurbo(self) -> kurbo::Join {
        match self {
            Self::Miter => kurbo::Join::Miter,
            Self::Round => kurbo::Join::Round,
            Self::Bevel => kurbo::Join::Bevel,
        }
    }
}

// ============================================================================
// Text Styling (Phase 5)
// ============================================================================

// ============================================================================
// Drawing State
// ============================================================================

/// Complete drawing state that can be saved and restored.
///
/// This represents the HTML5 Canvas drawing state, including transforms,
/// styles, and text settings.
#[derive(Debug, Clone)]
pub(crate) struct DrawingState {
    // Transform
    pub(crate) transform: kurbo::Affine,

    // Fill and Stroke
    pub(crate) fill_style: FillStyle,
    pub(crate) stroke_style: StrokeStyle,

    // Line styling
    pub(crate) line_width: f32,
    pub(crate) line_cap: LineCap,
    pub(crate) line_join: LineJoin,
    pub(crate) miter_limit: f32,
    pub(crate) line_dash: Vec<f32>,
    pub(crate) line_dash_offset: f32,

    // Global compositing
    pub(crate) global_alpha: f32,
    pub(crate) blend_mode: BlendMode,

    // Text styling (Phase 5)
    pub(crate) font: FontSpec,

    // Fill rule
    pub(crate) fill_rule: FillRule,
}

impl Default for DrawingState {
    fn default() -> Self {
        Self {
            transform: kurbo::Affine::IDENTITY,
            fill_style: FillStyle::Color(Color::from(Srgb::BLACK)),
            stroke_style: StrokeStyle::Color(Color::from(Srgb::BLACK)),
            line_width: 1.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            miter_limit: 10.0,
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            global_alpha: 1.0,
            blend_mode: BlendMode::default(),
            font: FontSpec::default(),
            fill_rule: FillRule::NonZero,
        }
    }
}

impl DrawingState {
    /// Creates a new drawing state with default values.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Builds a [`kurbo::Stroke`] from the current line styling state.
    #[must_use]
    pub(crate) fn build_stroke(&self) -> kurbo::Stroke {
        let mut stroke = kurbo::Stroke::new(f64::from(self.line_width))
            .with_caps(self.line_cap.to_kurbo())
            .with_join(self.line_join.to_kurbo())
            .with_miter_limit(f64::from(self.miter_limit));

        if !self.line_dash.is_empty() {
            let dashes: Vec<f64> = self.line_dash.iter().map(|&d| f64::from(d)).collect();
            stroke = stroke.with_dashes(f64::from(self.line_dash_offset), dashes);
        }

        stroke
    }
}
