# AI Translate Tool

A cross-platform AI-powered translation tool built with Rust and egui.

## Features

- Real-time translation with streaming support
- Multiple language support (10+ languages)
- Customizable font size and theme (dark/light mode)
- Persistent configuration (settings saved automatically)
- Clean and intuitive UI built with egui

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

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
