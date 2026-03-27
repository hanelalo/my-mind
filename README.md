# My Mind

A lightweight, intelligent voice-to-text desktop application for macOS. Press a hotkey, speak, and your words are automatically transcribed, polished by an LLM, and pasted into the active application.

## How It Works

1. Press **Option+Space** to activate
2. Speak into your microphone
3. Release the key (push-to-talk mode) or press again (toggle mode)
4. Audio is sent to an ASR service for transcription
5. The raw transcript is refined by an LLM (removes filler words, fixes errors, adds punctuation)
6. The final text is automatically pasted into the previously focused application

## Features

- **Global Hotkey** - Trigger recording from anywhere with Option+Space, no need to switch windows
- **Push-to-Talk & Toggle Modes** - Hold to record or press to start/stop
- **Online ASR** - Whisper API-compatible speech recognition, works with OpenAI, SiliconFlow, and other providers
- **LLM Post-Processing** - Intelligent cleanup of ASR output powered by OpenAI or Anthropic-compatible APIs
  - Removes filler words, stuttering, and self-corrections
  - Context-driven error correction (homophones, letter-number confusion, etc.)
  - Adds proper punctuation and formatting
  - Adaptive output: casual chat stays concise, substantive content gets structured with lists and highlights
- **Custom Prompt** - Customize the LLM post-processing prompt in Settings, or use the built-in default
- **Auto-Paste** - Automatically pastes the result into the previously active application (WeChat, browser, editor, etc.)
- **Error Feedback** - Overlay shows error messages on failure; text stays in clipboard for manual paste
- **Focus Management** - Captures the frontmost app before showing the overlay, restores focus after processing
- **History** - All transcriptions are saved locally (SQLite). Browse, search, copy, and delete past records from the History window
- **Settings UI** - Configure ASR, LLM, prompt, shortcuts, and output preferences from the system tray
- **macOS Native** - HudWindow overlay effect, system tray integration, accessibility permission auto-detection, CGEvent-based input simulation

## Tech Stack

- **Backend**: Rust + [Tauri v2](https://tauri.app/)
- **Frontend**: [SolidJS](https://www.solidjs.com/) + [Tailwind CSS](https://tailwindcss.com/) + TypeScript
- **Audio**: CPAL (capture) + Rubato (resampling) + Hound (WAV encoding)
- **ASR**: Whisper API-compatible (REST, multipart upload)
- **LLM**: OpenAI / Anthropic API-compatible
- **Storage**: SQLite via rusqlite (bundled, no system dependency)

## Getting Started

### Prerequisites

- macOS 10.15+
- Rust 1.85+ (via [rustup](https://rustup.rs/))
- Node.js 18+ and [pnpm](https://pnpm.io/)
- Tauri CLI: `cargo install tauri-cli`

### Configuration

Create `~/.config/my-mind/config.toml`:

```toml
[asr.online]
api_key = "your-asr-api-key"
api_base_url = "https://api.siliconflow.cn/v1"  # or any Whisper-compatible endpoint
model = "TeleAI/TeleSpeechASR"                  # optional

[llm]
provider = "openai"
api_key = "your-llm-api-key"
api_base_url = "https://api.moonshot.cn/v1"      # or any OpenAI-compatible endpoint
model = "kimi-k2-turbo-preview"                  # optional, default: gpt-4o-mini
temperature = 0.3
enabled = true
prompt = ""  # leave empty for built-in default, or set your own system prompt

[shortcuts]
record = "Alt+Space"
mode = "push_to_talk"  # or "toggle"

[output]
auto_paste = true
```

You can also configure all settings from the system tray menu (**Settings**).

### Build & Run

```bash
# Development
pnpm install
cargo tauri dev

# Production build
cargo tauri build
```

The built application will be at `target/release/bundle/macos/My Mind.app`.

### Permissions

On first launch, macOS will ask for:
- **Microphone** access (for audio recording)
- **Accessibility** access (for simulating paste via Cmd+V)

## Project Structure

```
my-mind/
├── crates/
│   ├── my-mind-core/        # Core library: audio, ASR, LLM, pipeline, history storage
│   └── my-mind-tauri/       # Tauri commands, state, events
├── packages/
│   └── web/                 # SolidJS frontend (overlay + settings + history UI)
├── src-tauri/               # Tauri app entry, setup, tray, shortcuts
└── Cargo.toml               # Rust workspace root
```

## Data Storage

- **Config**: `~/.config/my-mind/config.toml`
- **History**: `~/.config/my-mind/history.db` (SQLite)

## Roadmap

- [ ] Offline ASR support (local Whisper model)
- [ ] Shortcut hot-reload (apply without restart)
- [ ] Streaming ASR for real-time transcription display
- [ ] Multi-language auto-detection
- [ ] Windows and Linux support
- [ ] History search and export

## License

[MIT](LICENSE)
