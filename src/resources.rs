//! Engine resource handles owned by a mounted canvas.
//!
//! Fonts and images become drawable once the engine registers them; the
//! recorder then names them by id. `build_scene` hands every frame the
//! host's `SceneResources` table, so a source the drawing first reaches is
//! registered in the very frame that draws it — a font or image the closure
//! reaches for on its tenth commit is recorded with a live id, never queued
//! for a mount-time hook that has already run.
//!
//! Each lookup keeps the `Registered` handle the table returns, keyed by its
//! source. A recording names the registration behind the handle, so handles
//! are held exactly as long as the current frame draws them: a source a
//! later frame does not use is released on `end_frame`, while the content is
//! still mounted.

use alloc::sync::Arc;
use std::collections::{HashMap, HashSet};

use cherenkov::{Font, FontId, FontSource};
#[cfg(feature = "image")]
use cherenkov::{Image, ImageId, Rgba8};
use parley::FontData;
use waterui_graphics::{RecordingResources, Registered};

#[cfg(feature = "image")]
use crate::image::CanvasImage;

/// Dedup key for a font: the font collection's blob id plus the face index.
type FontKey = (u64, u32);

/// Dedup key for an image: the pixel buffer's allocation identity and extents.
#[cfg(feature = "image")]
type ImageKey = (usize, u32, u32);

fn font_key(font: &FontData) -> FontKey {
    (font.data.id(), font.index)
}

#[cfg(feature = "image")]
fn image_key(image: &CanvasImage) -> ImageKey {
    (
        Arc::as_ptr(&image.pixels).cast::<u8>() as usize,
        image.width,
        image.height,
    )
}

/// Registered handles kept for as long as the current frame draws them.
#[derive(Debug, Default)]
pub struct Resources {
    fonts: HashMap<FontKey, Registered<Font>>,
    #[cfg(feature = "image")]
    images: HashMap<ImageKey, Registered<Image<Rgba8>>>,
    used_fonts: HashSet<FontKey>,
    #[cfg(feature = "image")]
    used_images: HashSet<ImageKey>,
    failed_fonts: HashSet<FontKey>,
    #[cfg(feature = "image")]
    failed_images: HashSet<ImageKey>,
}

impl Resources {
    /// The `FontId` for `font`, registering it with `table` on first use.
    ///
    /// A source the engine rejects is marked failed so it is skipped rather
    /// than re-registered on every commit.
    pub fn font(&mut self, font: &FontData, names: &mut RecordingResources<'_>) -> Option<FontId> {
        let key = font_key(font);
        if let Some(handle) = self.fonts.get(&key) {
            self.used_fonts.insert(key);
            return Some(names.name(handle));
        }
        if self.failed_fonts.contains(&key) {
            return None;
        }
        let source = FontSource::bytes(Arc::<[u8]>::from(font.data.data())).with_index(font.index);
        match names.font(source) {
            Ok(handle) => {
                let id = names.name(&handle);
                self.fonts.insert(key, handle);
                self.used_fonts.insert(key);
                Some(id)
            }
            Err(error) => {
                tracing::warn!("canvas font registration failed: {error}");
                self.failed_fonts.insert(key);
                None
            }
        }
    }

    /// The `ImageId` for `image`, registering it with `table` on first use.
    #[cfg(feature = "image")]
    pub fn image(
        &mut self,
        image: &CanvasImage,
        names: &mut RecordingResources<'_>,
    ) -> Option<ImageId> {
        let key = image_key(image);
        if let Some(handle) = self.images.get(&key) {
            self.used_images.insert(key);
            return Some(names.name(handle));
        }
        if self.failed_images.contains(&key) {
            return None;
        }
        let data =
            cherenkov::ImageData::<Rgba8>::new(image.width, image.height, image.pixels.clone());
        match data.and_then(|data| names.image(data)) {
            Ok(handle) => {
                let id = names.name(&handle);
                self.images.insert(key, handle);
                self.used_images.insert(key);
                Some(id)
            }
            Err(error) => {
                tracing::warn!("canvas image registration failed: {error}");
                self.failed_images.insert(key);
                None
            }
        }
    }

    /// Releases every registration the frame that just recorded did not
    /// name; the host installs the recording before the engine renders
    /// again, so nothing still drawn loses its resource.
    pub fn end_frame(&mut self) {
        let used = std::mem::take(&mut self.used_fonts);
        self.fonts.retain(|key, _| used.contains(key));
        #[cfg(feature = "image")]
        {
            let used = std::mem::take(&mut self.used_images);
            self.images.retain(|key, _| used.contains(key));
        }
    }
}
