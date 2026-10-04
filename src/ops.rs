//! Collected drawing operations.
//!
//! A [`DrawingContext`](crate::DrawingContext) records into an [`OpTree`] while
//! the user's draw closure runs. `CanvasContent` then replays the tree into the
//! host's `draw::Recorder` on every commit, registering fonts and
//! images with the recording's `RecordingResources` as each is first drawn.

use alloc::{sync::Arc, vec::Vec};

use parley::FontData;
use waterui_graphics::RecordingResources;
#[cfg(feature = "image")]
use waterui_graphics::draw::Sampling;
#[cfg(feature = "image")]
use waterui_graphics::draw::kurbo::Rect;
use waterui_graphics::draw::kurbo::{Affine, BezPath, Stroke};
use waterui_graphics::draw::{
    Draw, EvenOdd, Glyph, GlyphRun, GlyphStyle, Group, Live, Paint, Recorder,
};

use crate::FillRule;
#[cfg(feature = "image")]
use crate::image::CanvasImage;

/// One collected drawing operation.
#[derive(Debug)]
pub enum Op {
    /// Fill `path` (already recorded in local space) with `paint`.
    Fill {
        /// Geometry transform applied to the whole op.
        transform: Affine,
        /// The path to fill.
        path: BezPath,
        /// Fill rule applied to `path`.
        rule: FillRule,
        /// The paint.
        paint: Live<Paint>,
    },
    /// Stroke `path` with `stroke` and `paint`.
    Stroke {
        /// Geometry transform applied to the whole op.
        transform: Affine,
        /// The path to stroke.
        path: BezPath,
        /// Stroke style.
        stroke: Stroke,
        /// The paint.
        paint: Live<Paint>,
    },
    /// Draw one glyph run.
    Glyphs {
        /// Geometry transform applied to the whole op.
        transform: Affine,
        /// The font data backing the run; resolves to a `FontId` at replay.
        font: FontData,
        /// Font size in points.
        size: f32,
        /// Variation coordinates.
        coords: Arc<[i16]>,
        /// Positioned glyphs in the run's local space.
        glyphs: Arc<[Glyph]>,
        /// Fill or stroke style of the run.
        style: GlyphStyle,
        /// The paint.
        paint: Live<Paint>,
        /// Extra alpha folded over the run (brush alpha).
        alpha: f32,
    },
    /// Draw an image's natural bounds under `transform`.
    #[cfg(feature = "image")]
    Image {
        /// Geometry transform applied to the whole op.
        transform: Affine,
        /// The source image; resolves to an `ImageId` at replay.
        image: CanvasImage,
        /// Destination rectangle in local space.
        dst: Rect,
        /// Sampling mode.
        sampling: Sampling,
    },
    /// Clip `body` to `path`, transformed by `transform`.
    Clip {
        /// Geometry transform applied to the clip shape only.
        transform: Affine,
        /// The clip path in local space.
        path: BezPath,
        /// Fill rule applied to the clip path.
        rule: FillRule,
        /// Ops inside the clip.
        body: Vec<Self>,
    },
    /// Clip `body` to `path`, then isolate it as `group`.
    Layer {
        /// Geometry transform applied to the clip shape only.
        transform: Affine,
        /// The clip path in local space.
        path: BezPath,
        /// Fill rule applied to the clip path.
        rule: FillRule,
        /// The isolated group style.
        group: Group,
        /// Ops inside the layer.
        body: Vec<Self>,
    },
}

/// A stack of nested scopes; the ops of the innermost open scope are appended
/// to its `ops`.
#[derive(Debug, Default)]
pub struct OpTree {
    frames: Vec<Frame>,
}

#[derive(Debug, Default)]
struct Frame {
    ops: Vec<Op>,
    clip: Option<ClipScope>,
}

#[derive(Debug)]
struct ClipScope {
    transform: Affine,
    path: BezPath,
    rule: FillRule,
    /// Set when the scope is also an isolated layer (`push_alpha_*`,
    /// `push_global_alpha_layer_if_needed`).
    group: Option<Group>,
}

impl OpTree {
    /// Pushes an op into the innermost open scope.
    pub fn push(&mut self, op: Op) {
        self.frames
            .last_mut()
            .expect("op pushed with no open frame")
            .ops
            .push(op);
    }

    /// Opens a clip scope.
    pub fn begin_clip(&mut self, transform: Affine, path: BezPath, rule: FillRule) {
        self.frames.push(Frame {
            clip: Some(ClipScope {
                transform,
                path,
                rule,
                group: None,
            }),
            ..Frame::default()
        });
    }

    /// Opens an isolated layer: clip to `path`, then group `body` as `group`.
    pub fn begin_layer(&mut self, transform: Affine, path: BezPath, rule: FillRule, group: Group) {
        self.frames.push(Frame {
            clip: Some(ClipScope {
                transform,
                path,
                rule,
                group: Some(group),
            }),
            ..Frame::default()
        });
    }

    /// Closes the innermost open scope, folding it back into its parent.
    pub fn end(&mut self) {
        let frame = self.frames.pop().expect("pop_layer with no open scope");
        let Some(clip) = frame.clip else {
            unreachable!("scope opened without a clip");
        };
        let op = match clip.group {
            Some(group) => Op::Layer {
                transform: clip.transform,
                path: clip.path,
                rule: clip.rule,
                group,
                body: frame.ops,
            },
            None => Op::Clip {
                transform: clip.transform,
                path: clip.path,
                rule: clip.rule,
                body: frame.ops,
            },
        };
        self.push(op);
    }

    /// Takes the collected ops, leaving the tree ready for the next frame.
    pub fn take(&mut self) -> Vec<Op> {
        debug_assert!(self.frames.len() <= 1, "unbalanced save/restore");
        let frame = self.frames.pop().unwrap_or_default();
        frame.ops
    }

    /// Opens the root frame. Must be called before the draw closure runs.
    pub fn begin_frame(&mut self) {
        debug_assert!(self.frames.is_empty());
        self.frames.push(Frame::default());
    }
}

/// Replays `ops` into `recorder`, resolving fonts and images through `state`
/// and registering sources the target has not seen yet with `names` — the
/// op that first draws a resource records its id in the same frame.
pub fn replay(
    ops: Vec<Op>,
    recorder: &mut Recorder,
    state: &mut crate::resources::Resources,
    names: &mut RecordingResources<'_>,
) {
    for op in ops {
        replay_op(op, recorder, state, names);
    }
}

/// Opens `shape` (pre-transformed by the op's transform) as a clip on
/// `recorder`, honouring the fill rule, then runs `body` inside it.
fn clip_with(
    recorder: &mut Recorder,
    path: BezPath,
    rule: FillRule,
    body: impl FnOnce(&mut Recorder),
) {
    match rule {
        FillRule::EvenOdd => recorder.clip(EvenOdd(path), body),
        FillRule::NonZero => recorder.clip(path, body),
    }
}

fn replay_op(
    op: Op,
    recorder: &mut Recorder,
    state: &mut crate::resources::Resources,
    names: &mut RecordingResources<'_>,
) {
    match op {
        Op::Fill {
            transform,
            path,
            rule,
            paint,
        } => {
            draw_shaped(recorder, transform, |r| fill_path(r, path, rule, paint));
        }
        Op::Stroke {
            transform,
            path,
            stroke,
            paint,
        } => {
            draw_shaped(recorder, transform, |r| {
                r.stroke(path, stroke, paint);
            });
        }
        Op::Glyphs {
            transform,
            font,
            size,
            coords,
            glyphs,
            style,
            paint,
            alpha,
        } => {
            replay_glyphs(
                recorder, transform, &font, size, coords, glyphs, style, paint, alpha, state, names,
            );
        }
        #[cfg(feature = "image")]
        Op::Image {
            transform,
            image,
            dst,
            sampling,
        } => {
            if let Some(id) = state.image(&image, names) {
                draw_shaped(recorder, transform, |r| {
                    r.image(id, dst, sampling);
                });
            }
        }
        Op::Clip {
            transform,
            path,
            rule,
            body,
        } => {
            let path = transform * path;
            clip_with(recorder, path, rule, |r| {
                for op in body {
                    replay_op(op, r, state, names);
                }
            });
        }
        Op::Layer {
            transform,
            path,
            rule,
            group,
            body,
        } => {
            let path = transform * path;
            clip_with(recorder, path, rule, |r| {
                r.group(group, |r| {
                    for op in body {
                        replay_op(op, r, state, names);
                    }
                });
            });
        }
    }
}

/// Draws a glyph run, folding the brush alpha into an isolated group when the
/// canvas painted the text at less than full opacity.
#[expect(
    clippy::too_many_arguments,
    reason = "the fn mirrors Op::Glyphs' fields; boxing them into a struct would rename the same tuple"
)]
fn replay_glyphs(
    recorder: &mut Recorder,
    transform: Affine,
    font: &FontData,
    size: f32,
    coords: Arc<[i16]>,
    glyphs: Arc<[Glyph]>,
    style: GlyphStyle,
    paint: Live<Paint>,
    alpha: f32,
    state: &mut crate::resources::Resources,
    names: &mut RecordingResources<'_>,
) {
    let Some(font_id) = state.font(font, names) else {
        return;
    };
    draw_shaped(recorder, transform, |r| {
        let run = GlyphRun {
            font: font_id,
            size,
            coords,
            glyphs,
            style,
        };
        if alpha < 1.0 {
            r.group(Group::new().opacity(alpha), |r| {
                r.glyphs(run, paint);
            });
        } else {
            r.glyphs(run, paint);
        }
    });
}

fn draw_shaped(recorder: &mut Recorder, transform: Affine, body: impl FnOnce(&mut Recorder)) {
    if transform == Affine::IDENTITY {
        body(recorder);
    } else {
        recorder.transform(transform, body);
    }
}

fn fill_path(
    recorder: &mut Recorder,
    path: BezPath,
    rule: FillRule,
    paint: impl Into<Live<Paint>>,
) {
    match rule {
        FillRule::EvenOdd => recorder.fill(EvenOdd(path), paint),
        FillRule::NonZero => recorder.fill(path, paint),
    }
}
