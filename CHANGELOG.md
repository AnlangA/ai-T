# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive contributing guidelines (CONTRIBUTING.md)
- Changelog to track version history
- Build optimization configurations in `.cargo/config.toml`
- Rust cache in GitHub Actions CI/CD workflows
- Release profile optimizations in Cargo.toml

### Changed
- Fixed Cargo.toml edition from "2024" to "2021"
- Renamed `buid_info.md` to `build_info.md` (fixed typo)
- Enhanced CI/CD workflows with caching for faster builds

### Fixed
- Corrected invalid Rust edition in Cargo.toml

## [0.0.5] - 2024-XX-XX

### Added
- Text-to-Speech (TTS) functionality
- Audio playback controls
- TTS configuration options (voice, speed, volume)
- Audio caching for improved performance

### Changed
- Refactored control flow and string formatting
- Improved error handling

## [0.1.0] - Initial Release

### Added
- Real-time AI translation with streaming support
- Multiple language support (10+ languages)
- Integration with Z.AI API (GLM-4.7 model)
- Customizable UI with adjustable font size
- Dark/light theme support
- Persistent configuration storage
- Translation caching for improved performance
- Structured logging with tracing
- Cross-platform support (Linux, Windows)
- Comprehensive unit tests
- CI/CD pipeline with GitHub Actions
- Automated releases for Linux and Windows

### Features
- **Translation**: Stream translations in real-time
- **Languages**: English, Chinese, Japanese, Korean, French, German, Spanish, Portuguese, Russian, Italian
- **UI**: Clean egui-based interface with responsive layout
- **Performance**: Local caching to avoid redundant API calls
- **Logging**: RUST_LOG environment variable support
- **Testing**: 25+ unit tests covering core functionality

[Unreleased]: https://github.com/AnlangA/ai-T/compare/v0.0.5...HEAD
[0.0.5]: https://github.com/AnlangA/ai-T/releases/tag/v0.0.5
