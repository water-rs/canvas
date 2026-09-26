//! Image loading and handling for Canvas.
//!
//! This module provides image loading from various sources (raw pixels, PNG, JPEG)
//! for use with the Canvas drawing API.

use alloc::rc::Rc;
use alloc::sync::Arc;
use core::cell::RefCell;
use core::fmt;
use std::error::Error;

use cherenkov::{Image, ImageData, ResourceError, Rgba8};
use waterui_core::layout::Size;
use waterui_graphics::SceneResources;
use waterui_graphics::image_decode::load_dynamic_image;

/// An image that can be drawn on the canvas.
///
/// Images can be created from raw RGBA pixels or decoded from PNG/JPEG bytes.
///
/// # Example
///
/// ```rust
/// # use waterui::prelude::*;
/// # use waterui_canvas::{CanvasImage, DrawingContext, ImageError};
/// # fn draw(ctx: &mut DrawingContext<'_>, png_data: &[u8]) -> Result<(), ImageError> {
/// // Load from PNG bytes
/// let image = CanvasImage::from_bytes(png_data)?;
///
/// // Draw at position
/// ctx.draw_image(&image, Point::new(10.0, 10.0));
///
/// // Draw scaled
/// ctx.draw_image_scaled(&image, Rect::new(Point::zero(), Size::new(200.0, 150.0)));
/// # Ok(())
/// # }
/// ```
pub struct CanvasImage {
    pixels: Arc<[u8]>,
    width: u32,
    height: u32,
    /// The handle minted by the engine this image was last drawn on.
    uploaded: RefCell<Option<Uploaded>>,
}

struct Uploaded {
    resources: Rc<dyn SceneResources>,
    handle: Image<Rgba8>,
}

impl CanvasImage {
    /// Creates an image from raw RGBA pixels.
    ///
    /// # Arguments
    /// * `width` - Width of the image in pixels
    /// * `height` - Height of the image in pixels
    /// * `pixels` - RGBA pixel data (4 bytes per pixel, must be width * height * 4 bytes)
    ///
    /// # Errors
    /// Returns an error if the pixel data length doesn't match width * height * 4.
    pub fn from_rgba_pixels(width: u32, height: u32, pixels: &[u8]) -> Result<Self, ImageError> {
        let expected_len = (width * height * 4) as usize;
        if pixels.len() != expected_len {
            return Err(ImageError::InvalidPixelData {
                expected: expected_len,
                got: pixels.len(),
            });
        }

        Ok(Self {
            pixels: Arc::from(pixels),
            width,
            height,
            uploaded: RefCell::new(None),
        })
    }

    /// Creates an image by decoding PNG or JPEG bytes.
    ///
    /// # Arguments
    /// * `bytes` - PNG or JPEG image data
    ///
    /// # Errors
    /// Returns an error if the image format is unsupported or decoding fails.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ImageError> {
        let img = load_dynamic_image(bytes).map_err(ImageError::DecodeError)?;

        // Convert to RGBA8
        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        let pixels = rgba.into_raw();

        Ok(Self {
            pixels: Arc::from(pixels),
            width,
            height,
            uploaded: RefCell::new(None),
        })
    }

    /// Returns the width of the image in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the size of the image.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub const fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    /// The engine handle for this image on `resources`, uploading it on the
    /// first draw against that engine.
    ///
    /// # Errors
    /// [`ResourceError`] when the engine rejects the upload.
    pub(crate) fn handle(
        &self,
        resources: &Rc<dyn SceneResources>,
    ) -> Result<cherenkov::ImageId, ResourceError> {
        let mut uploaded = self.uploaded.borrow_mut();
        if let Some(current) = uploaded
            .as_ref()
            .filter(|current| Rc::ptr_eq(&current.resources, resources))
        {
            return Ok(current.handle.id());
        }
        let data = ImageData::<Rgba8>::new(self.width, self.height, Arc::clone(&self.pixels))?;
        let handle = resources.image(data)?;
        let id = handle.id();
        *uploaded = Some(Uploaded {
            resources: Rc::clone(resources),
            handle,
        });
        Ok(id)
    }
}

impl fmt::Debug for CanvasImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CanvasImage")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

/// Errors that can occur when loading or creating images.
#[derive(Debug)]
pub enum ImageError {
    /// The pixel data length doesn't match the expected size.
    InvalidPixelData {
        /// The expected length of the pixel data in bytes.
        expected: usize,
        /// The actual length of the pixel data in bytes.
        got: usize,
    },
    /// Failed to decode image from bytes.
    DecodeError(image::ImageError),
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPixelData { expected, got } => {
                write!(
                    f,
                    "Invalid pixel data: expected {expected} bytes, got {got}"
                )
            }
            Self::DecodeError(err) => write!(f, "Failed to decode image: {err}"),
        }
    }
}

impl Error for ImageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPixelData { .. } => None,
            Self::DecodeError(err) => Some(err),
        }
    }
}
