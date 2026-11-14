//! Image data for the canvas.

use alloc::sync::Arc;

use waterui_core::layout::Size;

/// A decoded image the canvas can draw.
///
/// Pixel data is sRGB, straight (non-premultiplied) RGBA8. The same
/// `CanvasImage` is registered with the engine once per mount and then drawn
/// by id, so reuse it across frames rather than rebuilding it.
#[derive(Debug, Clone)]
pub struct CanvasImage {
    /// The RGBA8 pixel buffer.
    pub(crate) pixels: Arc<[u8]>,
    /// Width in pixels.
    pub(crate) width: u32,
    /// Height in pixels.
    pub(crate) height: u32,
}

impl CanvasImage {
    /// Creates an image from RGBA8 pixels (4 bytes per pixel, row-major).
    ///
    /// # Errors
    ///
    /// [`ImageError::InvalidPixelData`] when `pixels.len()` is not exactly
    /// `width * height * 4`, or either dimension is zero.
    pub fn from_rgba_pixels(width: u32, height: u32, pixels: &[u8]) -> Result<Self, ImageError> {
        let expected = width as usize * height as usize * 4;
        if width == 0 || height == 0 || pixels.len() != expected {
            return Err(ImageError::InvalidPixelData {
                expected,
                got: pixels.len(),
            });
        }
        Ok(Self {
            pixels: Arc::from(pixels),
            width,
            height,
        })
    }

    /// Decodes an image from encoded bytes (PNG, JPEG, TIFF).
    ///
    /// Requires the `image` feature.
    ///
    /// # Errors
    ///
    /// [`ImageError::Decode`] when the bytes cannot be decoded.
    #[cfg(feature = "image")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ImageError> {
        let dynamic = waterui_graphics::image_decode::load_dynamic_image(bytes)
            .map_err(ImageError::DecodeError)?;
        let rgba = dynamic.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(Self {
            pixels: Arc::from(rgba.into_raw().as_slice()),
            width,
            height,
        })
    }

    /// Width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The image's natural size.
    ///
    /// Dimensions above 2^24 pixels lose precision in `f32`; no decoder or
    /// engine surface allocates near that bound.
    #[allow(
        clippy::cast_precision_loss,
        reason = "pixel extents up to 2^24 are exact in f32 and far past any drawable surface"
    )]
    #[must_use]
    pub const fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }
}

/// Errors produced when loading a [`CanvasImage`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ImageError {
    /// The pixel data length doesn't match the expected size.
    InvalidPixelData {
        /// The expected length of the pixel data in bytes.
        expected: usize,
        /// The actual length of the pixel data in bytes.
        got: usize,
    },
    /// Failed to decode image from bytes.
    #[cfg(feature = "image")]
    DecodeError(::image::ImageError),
}

impl core::fmt::Display for ImageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidPixelData { expected, got } => {
                write!(
                    f,
                    "invalid pixel data: expected {expected} bytes, got {got}"
                )
            }
            #[cfg(feature = "image")]
            Self::DecodeError(error) => write!(f, "failed to decode image: {error}"),
        }
    }
}

impl core::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidPixelData { .. } => None,
            #[cfg(feature = "image")]
            Self::DecodeError(error) => Some(error),
        }
    }
}
