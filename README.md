# AI Translate Tool

A cross-platform AI-powered translation tool built with Rust and egui.

## Features

- **Real-time translation with streaming support** - See translations appear as they are generated
- **Multiple language support** - 10+ languages including English, Chinese, Japanese, Korean, and more
- **Customizable UI** - Adjustable font size and theme (dark/light mode)
- **Responsive layout** - Translation window scales with app size
- **Persistent configuration** - Settings saved automatically between sessions
- **Structured logging** - Comprehensive logging with RUST_LOG support
- **Clean and intuitive UI** - Built with egui for a native feel

## Architecture

The application follows modern Rust best practices:

- **Error Handling**: Custom error types using `thiserror`
- **Logging**: Structured logging with `tracing` and `tracing-subscriber`
- **Async Runtime**: Tokio for async operations and streaming
- **Type Safety**: Strong typing throughout with Result types
- **Testing**: Comprehensive unit tests for core functionality

## Logging

The application supports the `RUST_LOG` environment variable for controlling log output:

```bash
# Default (info level)
./ai-translate

# Debug level
RUST_LOG=debug ./ai-translate

# Trace level for detailed debugging
RUST_LOG=trace ./ai-translate

# Module-specific logging
RUST_LOG=ai_translate::api=debug ./ai-translate
```

## Building from Source

### Prerequisites

- Rust 1.70 or later
- On Linux: X11 development libraries

#### Linux

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libfontconfig1-dev
cargo build --release
```

#### Windows

```bash
cargo build --release
```

## Download Pre-built Binaries

### Official Releases

Download stable releases from the [Releases](https://github.com/AnlangA/ai-T/releases) page.

#### Linux (x86_64)

```bash
wget https://github.com/AnlangA/ai-T/releases/latest/download/ai-translate-linux-x86_64.tar.gz
tar xzf ai-translate-linux-x86_64.tar.gz
./ai-translate
```

#### Windows (x86_64)

Download `ai-translate-windows-x86_64.zip` from the releases page, extract it, and run `ai-translate.exe`.

### Development Builds

Development builds from the latest code are available:
1. Go to the [Actions](https://github.com/AnlangA/ai-T/actions/workflows/release.yml) page
2. Click on the most recent successful workflow run
3. Scroll down to the "Artifacts" section
4. Download `ai-translate-linux-x86_64.tar.gz` or `ai-translate-windows-x86_64.zip`

Alternatively, manually triggered builds create draft releases that can be found in the [Releases](https://github.com/AnlangA/ai-T/releases) section.

## Usage

1. Enter your Z.AI API key in the settings panel
2. Select your target language
3. Type or paste text to translate
4. Click "Translate" to start the translation

## Configuration

Settings are automatically saved and restored between sessions:
- API Key
- Target Language
- Font Size
- Theme (Dark/Light mode)

## Development

```bash
# Run in debug mode
cargo run

# Run with logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Run linter
cargo clippy

# Run linter with auto-fix
cargo clippy --fix
```

## Testing

The project includes comprehensive unit tests:

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test config
cargo test api

# Run with verbose output
cargo test -- --nocapture --test-threads=1
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
