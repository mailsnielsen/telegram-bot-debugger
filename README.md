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
- Webhook management (get, set, delete)

## Quick Start

```bash
# Build and run
cargo build --release
cargo run --release

# Enter your bot token from @BotFather
# Navigate: 1=Discovery, 2=Monitor, 3=Analytics, 4=Raw JSON, 5=Webhooks, m=Test Message
```

## Installation

### From Binary (Recommended)

Download the latest release for your platform from the [Releases page](https://github.com/mailsnielsen/telegram-bot-debugger/releases).

**Linux**:
```bash
wget https://github.com/mailsnielsen/telegram-bot-debugger/releases/latest/download/telegram-bot-debugger-linux-x86_64.tar.gz
tar xzf telegram-bot-debugger-linux-x86_64.tar.gz
./telegram-bot-debugger
```

**macOS**:
```bash
# Download and extract
curl -LO https://github.com/mailsnielsen/telegram-bot-debugger/releases/latest/download/telegram-bot-debugger-macos-x86_64.tar.gz
tar xzf telegram-bot-debugger-macos-x86_64.tar.gz

# Remove quarantine attribute (required for unsigned binaries)
xattr -d com.apple.quarantine telegram-bot-debugger

# Run
./telegram-bot-debugger
```

**Windows**: Download the `.zip`, extract, and run `telegram-bot-debugger.exe`

### Verify Download (Recommended)

```bash
# Download checksum file
wget https://github.com/mailsnielsen/telegram-bot-debugger/releases/latest/download/telegram-bot-debugger-linux-x86_64.tar.gz.sha256

# Verify integrity
sha256sum -c telegram-bot-debugger-linux-x86_64.tar.gz.sha256
```

### From Source

```bash
git clone https://github.com/mailsnielsen/telegram-bot-debugger.git
cd telegram-bot-debugger
cargo build --release
./target/release/telegram-bot-debugger
```

## Usage

### Navigation

| Key | Action |
|-----|--------|
| `1-5` | Switch screens |
| `m` | Send test message |
| `F5` | Toggle live monitor |
| `↑/↓` | Navigate lists |
| `Esc` | Quit / Return home |

### Screens

**Discovery (1)**: View all discovered chats with IDs. Chat types are color-coded (green=private, blue=group, yellow=channel).

**Live Monitor (2)**: Real-time message stream with timestamps and sender information.

**Analytics (3)**: Statistics on chat activity, message counts, and hourly distribution.

**Raw JSON (4)**: Complete API responses with all update types. Use arrow keys to navigate through updates.

**Webhook Management (5)**: Configure webhooks and polling mode. View current webhook status, set new webhooks, or delete webhooks to enable polling.
  - Press `i` to get webhook info
  - Enter HTTPS URL and press `Enter` to set webhook
  - Press `d` to delete webhook and enable polling

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

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for:
- How to report bugs and suggest features
- Development setup and workflow
- Code standards and testing requirements
- Pull request process

## Dependencies

- **tokio** - Async runtime
- **ratatui** - Terminal UI
- **reqwest** - HTTP client
- **serde/serde_json** - Serialization
- **chrono** - Date/time handling

## Troubleshooting

**No updates**: If the bot isn't receiving updates, you likely have a webhook configured. Use the **Webhook Management** screen (press `5`) to:
- View current webhook status (press `i`)
- Delete the webhook to enable polling (press `d`)

Alternatively, manage webhooks programmatically:

```rust
use telegram_bot_debugger::telegram::TelegramClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = TelegramClient::new("YOUR_BOT_TOKEN".to_string());
    
    // Check current webhook status
    let info = client.get_webhook_info().await?;
    println!("Webhook URL: {}", info.result.unwrap().url);
    
    // Delete webhook to enable polling
    client.delete_webhook(Some(true)).await?;
    println!("Webhook deleted, polling enabled");
    
    Ok(())
}
```

Or use curl: `https://api.telegram.org/bot<TOKEN>/deleteWebhook`

**Token errors**: Verify format `123456789:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`

**Missing chats**: Ensure messages were sent and Live Monitor is running (`F5`)

**macOS security warning**: Remove quarantine with `xattr -d com.apple.quarantine telegram-bot-debugger` or allow in System Settings → Privacy & Security

**Windows SmartScreen**: Click "More info" → "Run anyway"

## Security

- Tokens stored in plain text in `config/cache.json`
- Never commit cache files to version control
- Use `.gitignore` (already configured)

## License

MIT License. See LICENSE file for details.

## Acknowledgments

Built with [Ratatui](https://github.com/ratatui/ratatui), [Tokio](https://tokio.rs/), and the [Telegram Bot API](https://core.telegram.org/bots/api).
