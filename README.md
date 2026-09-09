# waterui-canvas

Immediate-mode 2D drawing canvas component for WaterUI using Vello.

## Cargo features

| Feature | Default | What it adds |
| --- | --- | --- |
| `image` | yes | Raster images on the canvas: `CanvasImage`, `ImageError`, `DrawingContext::draw_image`, `draw_image_scaled`, `draw_image_sub`, and the `CanvasResource::Image` arm. |

Decoding a PNG/JPEG/TIFF goes through `waterui-graphics`'s `image_decode`,
which that crate gates behind its own `gpu` feature, so `image` is what brings
the `wgpu` stack into the dependency graph. Vector drawing — paths, shapes,
gradients, text, clipping and transforms — needs none of it.

A consumer that never draws a decoded image, such as a CPU-only renderer like a
dew firmware build, turns the feature off and links neither the image codecs nor
`wgpu`:

```toml
[dependencies]
waterui-canvas = { version = "0.1", default-features = false }
```

`cargo tree -e normal --no-default-features` shows no `wgpu` in that
configuration.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
