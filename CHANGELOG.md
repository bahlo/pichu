# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1] - 2025-06-08

### Changed

- Updated `watch` function to accept multiple paths

## [0.4.0] - 2025-06-08

- Change signature of `parse`, `try_parse` to take `&PathBuf` instead of `PathBuf`
- Add strict clippy settings
- Add `watch` fn

## [0.3.2] - 2025-06-08

### Added

- Export `parse_markdown` fn, to allow people to add logic before/after in `try_parse`.
- Add missing docs and lint for it

## [0.3.1] - 2025-06-07

### Changed

- `Error` is now Sync
- `Error::Render` and `Error::Parse` no longer require the `std::error::Error` trait for more flexibility
- `Error::RenderFn` is renamed to `Error::Render`

## [0.3.0] - 2025-06-07

### Added
- `try_parse`, `try_render` and `try_render_each`, improve ergonomics of these and the non-try Fns
- Parallel sorting
- Basic test suite

### Changed
- Move markdown, sass error to their own module
- Adapt argument order of `pichu::write` to match that of `fs::write`

## [0.2.0] - 2025-06-03

### Added
- `copy_dir` fn
- Basic documentation
- Examples

## [0.1.0] - 2025-06-02

The first release! Heavily WIP and has rough edges, but usable.

[unreleased]: https://github.com/bahlo/pichu/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/bahlo/pichu/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/bahlo/pichu/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/bahlo/pichu/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/bahlo/pichu/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/bahlo/pichu/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/bahlo/pichu/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/bahlo/pichu/releases/tag/v0.1.0
