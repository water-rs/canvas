# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/water-rs/canvas/compare/v0.2.0...v0.3.0) - 2026-11-14

### Added

- port to the render-target-neutral recording contract ([#44](https://github.com/water-rs/canvas/pull/44))
- implement SceneContent::rebuild_for_engine engine-recovery contract
- follow the r4 scene contract
- [**breaking**] port the canvas recorder to the Cherenkov engine API

### Fixed

- repin hydrolysis to 14d5e231, m3 to fcb8b9e8; patch all registry waterkit crates to 123b0581
- *(ci)* pin the toolchain without disarming the MSRV leg ([#36](https://github.com/water-rs/canvas/pull/36))

### Other

- switch water-rs git dependencies and workflow refs from dev to main ([#60](https://github.com/water-rs/canvas/pull/60))
- release from main and point sibling deps to main ([#56](https://github.com/water-rs/canvas/pull/56))
- restore the Hydrolysis test fonts from the Actions cache ([#54](https://github.com/water-rs/canvas/pull/54))
- *(deps)* point migrated Batch 1 git dependencies to main ([#51](https://github.com/water-rs/canvas/pull/51)) ([#52](https://github.com/water-rs/canvas/pull/52))
- consume the framework from its dev branch
- lock the framework at its rebased nami 0.13 head
- lock the sibling components at their nami 0.13 heads
- [**breaking**] move to nami 0.13 and the migrated framework
- pass the release pull request's gate without CI
- [**breaking**] move the patch table to nami 0.12
- require nami 0.12
- *(deps)* pin waterui to dev at 46552dd30 ([#45](https://github.com/water-rs/canvas/pull/45))
- install Hydrolysis test fonts before tests
- *(deps)* repin water-rs/waterui to 1e359bca
- *(deps)* repin the engine and framework onto in-tree Cherenkov
- *(deps)* pin hydrolysis, hydrolysis-m3 and video-gpu at the repinned heads
- pin cherenkov to 52236561
- install SPIRV-Tools through the shared Linux deps action
- install SPIRV-Tools for cherenkov-gpu's shader build on non-Apple legs
- repin to the cherenkov cutover base
- repin to waterui f3576aa, cherenkov 8d00976e, waterkit 2f1e359
- name RecordingResources in the resources module
- repin cherenkov to its rewritten history
- repin the vello fork to the glifo canonical-location fix
- repin water-rs dependencies to merged dev heads
- repin water-rs dependencies to unified migration heads
- repin hydrolysis to accessibility import fix
- refresh lockfile for final branch revs
- repin migrated dependency branch heads
- pin migrated dependency branch revs
- [**breaking**] migrate to nami snapshot API
- consume the 0.5.0 framework wave from crates.io ([#35](https://github.com/water-rs/canvas/pull/35))

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
