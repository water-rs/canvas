//! The commands a drawing callback issues, kept in scope order until the frame
//! is recorded.
//!
//! The canvas API is imperative: `push_clip_rect` opens a scope that a later
//! `pop_layer` closes. Cherenkov's [`Draw`] scopes are closures, so the
//! context collects commands into a tree of scopes and replays that tree into
//! the recorder once the callback returns.

use cherenkov::kurbo::{Affine, Rect, Stroke};
use cherenkov::{Draw, GlyphRun, Group, ImageId, Paint, Picture, Sampling, ShapeData, StaticRecorder};

/// One command, positioned by the canvas transform in force when it was issued.
pub enum Op {
    Fill {
        transform: Affine,
        shape: ShapeData,
        paint: Paint,
    },
    Stroke {
        transform: Affine,
        shape: ShapeData,
        stroke: Stroke,
        paint: Paint,
    },
    Glyphs {
        transform: Affine,
        run: GlyphRun,
        paint: Paint,
    },
    Image {
        transform: Affine,
        image: ImageId,
        dst: Rect,
        sampling: Sampling,
    },
    Clip {
        transform: Affine,
        shape: ShapeData,
        body: Vec<Self>,
    },
    Layer {
        transform: Affine,
        shape: ShapeData,
        group: Group,
        body: Vec<Self>,
    },
}

/// An open scope: the commands issued so far and how the scope closes.
struct Frame {
    kind: FrameKind,
    body: Vec<Op>,
}

enum FrameKind {
    Root,
    Clip { transform: Affine, shape: ShapeData },
    Layer {
        transform: Affine,
        shape: ShapeData,
        group: Group,
    },
}

/// The command tree under construction.
pub struct OpTree {
    frames: Vec<Frame>,
}

impl OpTree {
    pub(crate) fn new() -> Self {
        Self {
            frames: vec![Frame {
                kind: FrameKind::Root,
                body: Vec::new(),
            }],
        }
    }

    pub(crate) fn push(&mut self, op: Op) {
        self.frames
            .last_mut()
            .expect("the root scope is never popped")
            .body
            .push(op);
    }

    pub(crate) fn begin_clip(&mut self, transform: Affine, shape: ShapeData) {
        self.frames.push(Frame {
            kind: FrameKind::Clip { transform, shape },
            body: Vec::new(),
        });
    }

    pub(crate) fn begin_layer(&mut self, transform: Affine, shape: ShapeData, group: Group) {
        self.frames.push(Frame {
            kind: FrameKind::Layer {
                transform,
                shape,
                group,
            },
            body: Vec::new(),
        });
    }

    /// Closes the innermost scope.
    ///
    /// # Panics
    /// Panics when no scope is open: a `pop_layer` without a matching push is
    /// a programming error in the drawing callback.
    pub(crate) fn end(&mut self) {
        assert!(
            self.frames.len() > 1,
            "waterui-canvas: pop_layer called with no open layer"
        );
        let frame = self.frames.pop().expect("checked above");
        let op = match frame.kind {
            FrameKind::Root => unreachable!("the root scope is never popped"),
            FrameKind::Clip { transform, shape } => Op::Clip {
                transform,
                shape,
                body: frame.body,
            },
            FrameKind::Layer {
                transform,
                shape,
                group,
            } => Op::Layer {
                transform,
                shape,
                group,
                body: frame.body,
            },
        };
        self.push(op);
    }

    /// Visits every command in issue order, entering scopes.
    #[cfg(test)]
    pub(crate) fn visit(&self, visitor: &mut dyn FnMut(&Op)) {
        fn walk(ops: &[Op], visitor: &mut dyn FnMut(&Op)) {
            for op in ops {
                visitor(op);
                match op {
                    Op::Clip { body, .. } | Op::Layer { body, .. } => walk(body, visitor),
                    _ => {}
                }
            }
        }
        for frame in &self.frames {
            walk(&frame.body, visitor);
        }
    }

    /// Replays the tree into a picture.
    ///
    /// # Panics
    /// Panics when a scope is still open: every push needs its `pop_layer`.
    pub(crate) fn finish(mut self) -> Picture {
        assert!(
            self.frames.len() == 1,
            "waterui-canvas: {} layer(s) left open at the end of the drawing callback",
            self.frames.len() - 1
        );
        let root = self.frames.pop().expect("the root scope exists");
        Picture::record(|recorder| replay_ops(recorder, root.body))
    }
}

fn replay_ops(draw: &mut StaticRecorder, ops: Vec<Op>) {
    for op in ops {
        match op {
            Op::Fill {
                transform,
                shape,
                paint,
            } => draw.transform(transform, |draw| draw.fill(shape, paint)),
            Op::Stroke {
                transform,
                shape,
                stroke,
                paint,
            } => draw.transform(transform, |draw| draw.stroke(shape, stroke, paint)),
            Op::Glyphs {
                transform,
                run,
                paint,
            } => draw.transform(transform, |draw| draw.glyphs(&run, paint)),
            Op::Image {
                transform,
                image,
                dst,
                sampling,
            } => draw.transform(transform, |draw| draw.image(image, dst, sampling)),
            Op::Clip {
                transform,
                shape,
                body,
            } => draw.transform(transform, |draw| {
                draw.clip(shape, |draw| replay_ops(draw, body));
            }),
            Op::Layer {
                transform,
                shape,
                group,
                body,
            } => draw.transform(transform, |draw| {
                draw.clip(shape, |draw| {
                    draw.group(group, |draw| replay_ops(draw, body));
                });
            }),
        }
    }
}
