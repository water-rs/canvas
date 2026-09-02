# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/water-rs/canvas/releases/tag/v0.1.0) - 2026-09-02

### Added

- *(graphics)* render scenes on the CPU/GPU split engine where compute is missing

### Fixed

- *(release)* verify registry-only package graph
- *(canvas)* [**breaking**] repair draw_image_sub, cache measure_text, drop dead shadow API

### Other

- update Linux package matrix and add dxc on Windows
- setup standalone crate files, CI workflows, and release-plz
- *(deps)* drop `image`'s AVIF encoder from every consumer
- [**breaking**] ungate Scene2D from the GPU stack and drop its Vello escape hatches
- ship the licence texts in every published crate
- prepare publishable dependency graph
- *(canvas)* compile every doc example
- Format the workspace
- Remove premature heap allocations
- upgrade workspace dependencies
- achieve zero clippy warnings across the workspace
- clean up clippy warnings across the workspace
- Dew on ESP32-S3: parley libm path, firmware demo, Xtensa miscompile triage
- Lean dependency graph for embedded: gpu/widgets/gestures features
- Restore WaterUI CI gates and reactive map API
- reorganize the project
