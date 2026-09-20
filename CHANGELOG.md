# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/water-rs/canvas/compare/v0.1.1...v0.1.2) - 2026-09-11

### Fixed

- *(ci)* install the same Linux packages for the release preflight
- *(ci)* prepare release PRs only after publication

### Other

- link the test graph to the waterui 0.4 release commit
- *(deps)* waterui-graphics 0.4 (and waterui-text/-testing 0.4 where used)

## [0.1.1](https://github.com/water-rs/canvas/compare/v0.1.0...v0.1.1) - 2026-09-09

### Added

- report a path's bounding box
- gate canvas images behind a default-on `image` feature

### Fixed

- *(ci)* install native prerequisites for release verification
- make Path::arc_to actually behave like HTML5 arcTo

### Other

- resolve gpu-allocator against windows 0.62, as wgpu-hal does
- depend on the released waterui crates instead of the monorepo dev branch
- refresh the lockfile onto the rewritten waterui dev history
- update Linux package matrix and add dxc on Windows
