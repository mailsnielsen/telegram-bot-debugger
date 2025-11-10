# Telegram Bot Debugger

[![CI](https://github.com/mailsnielsen/telegram-bot-debugger/actions/workflows/ci.yml/badge.svg)](https://github.com/mailsnielsen/telegram-bot-debugger/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![codecov](https://codecov.io/gh/mailsnielsen/telegram-bot-debugger/branch/master/graph/badge.svg)](https://codecov.io/gh/mailsnielsen/telegram-bot-debugger)

A terminal-based debugging tool for Telegram bots. Discover chats, test messages, monitor activity, and inspect raw API responses.

## Features

- Chat discovery with automatic ID extraction
- Live message monitoring
- Test message sending
- Analytics and statistics
- Raw JSON API inspector (25+ update types)

## Quick Start

```bash
# Build and run
cargo build --release
cargo run --release

# Enter your bot token from @BotFather
# Navigate: 1=Discovery, 2=Monitor, 3=Analytics, 4=Raw JSON, m=Test Message
```

## Installation

```bash
git clone <repository-url>
cd telegram-bot-debugger
cargo build --release
./target/release/telegram-bot-debugger
```

## Usage

### Navigation

| Key | Action |
|-----|--------|
| `1-4` | Switch screens |
| `m` | Send test message |
| `F5` | Toggle live monitor |
| `↑/↓` | Navigate lists |
| `Esc` | Quit / Return home |

### Screens

**Discovery (1)**: View all discovered chats with IDs. Chat types are color-coded (green=private, blue=group, yellow=channel).

**Live Monitor (2)**: Real-time message stream with timestamps and sender information.

**Analytics (3)**: Statistics on chat activity, message counts, and hourly distribution.

**Raw JSON (4)**: Complete API responses with all update types. Use arrow keys to navigate through updates.

**Test Message (m)**: Send messages to discovered chats or manual Chat IDs. Supports forum topics.

### Getting Chat IDs

**Private chats**: Send a message to your bot, check Discovery screen. Positive integer.

**Groups/Channels**: Add bot, send message, check Discovery. Negative integer starting with `-100`.

## Configuration

Bot token and cache stored in `config/cache.json` (auto-generated). To reset: `rm config/cache.json`

## Development

### Project Structure

```
src/
├── main.rs           # Entry point
├── app/              # State management
├── telegram/         # API client (types, updates, client)
├── ui/               # Terminal interface
├── storage/          # Cache handling
└── analytics/        # Statistics
```

### Testing

```bash
cargo test              # Run all tests
cargo test --lib        # Unit tests only
cargo clippy            # Linting
cargo fmt               # Formatting
```

The project includes 125+ unit tests, integration tests, and property-based tests.

## Dependencies

- **tokio** - Async runtime
- **ratatui** - Terminal UI
- **reqwest** - HTTP client
- **serde/serde_json** - Serialization
- **chrono** - Date/time handling

## Troubleshooting

**No updates**: Disable webhooks with `https://api.telegram.org/bot<TOKEN>/deleteWebhook`

**Token errors**: Verify format `123456789:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`

**Missing chats**: Ensure messages were sent and Live Monitor is running (`F5`)

## Security

- Tokens stored in plain text in `config/cache.json`
- Never commit cache files to version control
- Use `.gitignore` (already configured)

## License

MIT License. See LICENSE file for details.

## Acknowledgments

Built with [Ratatui](https://github.com/ratatui/ratatui), [Tokio](https://tokio.rs/), and the [Telegram Bot API](https://core.telegram.org/bots/api).
