//! End-to-end tests for the `canvas` component.
//!
//! The scene tests mount a [`Canvas`] the way a backend does — `View::body`
//! produces a `SceneView`, the `SceneContent` inside it is what an engine
//! renders — and render it offscreen through the Cherenkov GPU engine and
//! the CPU raster engine. The PNGs land in `/tmp/waterui_canvas_e2e/`; engine
//! output is not pixel-stable across platforms, so nothing is asserted about
//! the pixels. The accessibility tests mount the view in the semantic harness
//! and inspect the tree it publishes.

use waterui_canvas::Canvas;
use waterui_core::layout::{Point, Rect, Size};
use waterui_core::{AnyView, Environment, Native, View as _};
use waterui_graphics::color::Srgb;
use waterui_graphics::{OffscreenRenderer, OffscreenSize, SceneView};

fn canvas_fill_rect_view() -> Canvas {
    Canvas::new(|ctx| {
        ctx.set_fill_style(Srgb::BLACK);
        ctx.fill_rect(Rect::from_size(Size::new(ctx.width, ctx.height)));

        ctx.set_fill_style(Srgb::new(1.0, 0.25, 0.1));
        ctx.fill_rect(Rect::new(Point::new(20.0, 16.0), Size::new(100.0, 60.0)));
    })
}

fn canvas_stroke_and_clip_view() -> Canvas {
    Canvas::new(|ctx| {
        ctx.set_fill_style(Srgb::new(0.09, 0.10, 0.15));
        ctx.fill_rect(Rect::from_size(Size::new(ctx.width, ctx.height)));

        ctx.push_clip_rect(Rect::new(Point::new(8.0, 8.0), Size::new(80.0, 80.0)));
        ctx.set_fill_style(Srgb::new(0.40, 0.85, 0.60));
        ctx.fill_circle(Point::new(48.0, 48.0), 56.0);
        ctx.pop_layer();

        ctx.set_stroke_style(Srgb::new(1.0, 0.84, 0.32));
        ctx.set_line_width(5.0);
        ctx.stroke_circle(Point::new(120.0, 50.0), 34.0);
    })
}

/// Extracts the `SceneContent` a mounted `Canvas` carries, the way a backend
/// does when it meets the `Native<SceneView>` leaf.
fn mounted_content(view: Canvas, env: &Environment) -> Box<dyn waterui_graphics::SceneContent> {
    // `Canvas::body` returns the `SceneView`; calling `body` on it wraps it as
    // the native leaf `AnyView` a backend would see.
    let leaf = view.body(env).body(env);
    let any = AnyView::new(leaf);
    let native = any
        .downcast::<Native<SceneView>>()
        .unwrap_or_else(|_| panic!("Canvas must mount as a Native<SceneView>"));
    native.into_inner().into_content()
}

fn render_on_both_engines(make_view: impl Fn() -> Canvas, name: &str) {
    use waterui_graphics::OffscreenImage;

    let directory = std::path::Path::new("/tmp/waterui_canvas_e2e");
    std::fs::create_dir_all(directory).expect("output directory must be creatable");
    let size = OffscreenSize::try_from_pixels(320, 200).expect("test size must be valid");
    let env = Environment::new();

    let save = |image: OffscreenImage, file: &str| {
        image
            .save_png(directory.join(file))
            .expect("png should be written");
    };

    let gpu = OffscreenRenderer::<waterui_graphics::cherenkov_gpu::Gpu>::new()
        .expect("scene render requires a working GPU engine");
    let mut content = mounted_content(make_view(), &env);
    // The frame that first draws a resource registers it, so one render
    // produces the complete drawing.
    save(
        gpu.render(content.as_mut(), size, 1.0)
            .expect("offscreen render should succeed"),
        &format!("{name}-gpu.png"),
    );

    let cpu = OffscreenRenderer::<waterui_graphics::cherenkov_cpu::Raster>::cpu()
        .expect("raster engine should initialise");
    let mut content = mounted_content(make_view(), &env);
    save(
        cpu.render(content.as_mut(), size, 1.0)
            .expect("offscreen render should succeed"),
        &format!("{name}-raster.png"),
    );
}

#[test]
fn canvas_fill_rect_renders_on_gpu_and_raster() {
    render_on_both_engines(canvas_fill_rect_view, "fill-rect");
}

#[test]
fn canvas_stroke_and_clip_renders_on_gpu_and_raster() {
    render_on_both_engines(canvas_stroke_and_clip_view, "stroke-clip");
}

/// A canvas whose draw closure first reaches for an image on a later frame
/// shows it in that frame: `build_scene` registers the source on demand, so
/// no mount-time registration pass has to have run for a late resource.
#[cfg(feature = "image")]
#[test]
fn canvas_draws_an_image_first_seen_on_a_later_frame() {
    let env = Environment::new();
    let frame = std::cell::Cell::new(0u32);
    let pixels: Vec<u8> = [255u8, 0, 0, 255]
        .into_iter()
        .cycle()
        .take(8 * 8 * 4)
        .collect();
    let image = waterui_canvas::CanvasImage::from_rgba_pixels(8, 8, &pixels)
        .expect("test image must be constructible");
    let view = Canvas::new(move |ctx| {
        frame.set(frame.get() + 1);
        ctx.set_fill_style(Srgb::BLACK);
        ctx.fill_rect(Rect::from_size(Size::new(ctx.width, ctx.height)));
        if frame.get() >= 2 {
            ctx.draw_image(&image, Point::new(40.0, 40.0));
        }
    });
    let mut content = mounted_content(view, &env);
    let size = OffscreenSize::try_from_pixels(160, 120).expect("test size must be valid");
    let cpu = OffscreenRenderer::<waterui_graphics::cherenkov_cpu::Raster>::cpu()
        .expect("raster engine should initialise");

    let first = cpu
        .render(content.as_mut(), size, 1.0)
        .expect("the first frame renders without the image");
    assert_eq!(
        first.pixel(44, 44),
        [0, 0, 0, 255],
        "frame one draws only the black background"
    );

    let second = cpu
        .render(content.as_mut(), size, 1.0)
        .expect("the second frame registers and draws the image");
    let [r, g, b, a] = second.pixel(44, 44);
    assert!(
        r > 200 && g < 60 && b < 60 && a > 200,
        "frame two shows the registered image at (44, 44): [{r}, {g}, {b}, {a}]"
    );
}

mod accessibility {
    use waterui::ViewExt as _;
    use waterui::accessibility::AccessibilityRole;
    use waterui::graphics::color::Srgb;
    use waterui::layout::{Point, Rect, Size};
    use waterui_canvas::Canvas;
    use waterui_testing::{Role, SemanticApp};

    fn canvas_fill_rect_view() -> impl waterui::View {
        Canvas::new(|ctx| {
            ctx.set_fill_style(Srgb::new(1.0, 0.25, 0.1));
            ctx.fill_rect(Rect::new(Point::new(20.0, 16.0), Size::new(100.0, 60.0)));
        })
        .size(160.0, 100.0)
        .background(Srgb::BLACK)
        .a11y_role(AccessibilityRole::Image)
        .a11y_label("Filled rect canvas")
    }

    #[waterui::test(canvas_fill_rect_view)]
    fn canvas_fill_rect_exposes_accessibility_node(app: &mut SemanticApp) {
        app.query()
            .role(Role::IMAGE)
            .label("Filled rect canvas")
            .assert_exists();
    }
}
