# Starwell Browser Extension

Browser extension for Starwell Password Manager with real-time password generation.

## Features

- **Real-time streaming**: Password appears as you type your phrase
- **NFC normalization**: Ensures compatibility with websites
- **Zero clipboard**: Characters streamed directly to password field
- **Keyboard shortcuts**: Ctrl+Shift+S to activate

## Installation

### 1. Build the Extension

The extension files are already created in `browser-extension/`.

### 2. Install Native Messaging Host

The native messaging host allows the extension to communicate with the Starwell binary.

**For Chrome/Brave on Linux:**

```bash
# Create the native messaging hosts directory
mkdir -p ~/.config/google-chrome/NativeMessagingHosts/
# or for Chromium:
# mkdir -p ~/.config/chromium/NativeMessagingHosts/
# or for Brave:
mkdir -p ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/

# Copy the host manifest (update the path to your binary first!)
cp browser-extension/native-host/com.starwell.password_manager.json \
   ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/
```

**Important:** Edit the JSON file to update the `path` field to point to your actual binary location:
```json
{
  "path": "/home/YOUR_USERNAME/path/to/starwell_password_manager"
}
```

### 3. Load Extension in Brave

1. Open Brave
2. Go to `brave://extensions/`
3. Enable "Developer mode" (top right toggle)
4. Click "Load unpacked"
5. Select the `browser-extension/` directory
6. Note the Extension ID shown (you'll need this)

### 4. Update Native Host Configuration

Edit `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.starwell.password_manager.json`:

Replace `EXTENSION_ID_HERE` with your actual extension ID from step 3:
```json
{
  "allowed_origins": [
    "chrome-extension://YOUR_ACTUAL_EXTENSION_ID/"
  ]
}
```

### 5. Build the Binary

```bash
cd /home/charon/Projects/starwell/starwell_password_manager
cargo build --release
```

Make sure you have at least one shape created:
```bash
./target/release/starwell_password_manager
# Follow prompts to create a new shape
```

## Usage

1. Navigate to any website with a password field
2. Click on the password field
3. Press **Ctrl+Shift+S** to activate Starwell
4. Type your memorable phrase (you won't see what you type)
5. Watch the secure password appear character-by-character
6. Press **Enter** when done, or **Esc** to cancel

## How It Works

### Architecture

```
Web Page (password field)
    ↕
Content Script (content.js)
    ↕
Background Worker (background.js)
    ↕
Native Messaging
    ↕
Starwell Binary (--stream mode)
    ↕
7D Shape System
```

### Message Flow

1. User presses Ctrl+Shift+S on password field
2. Content script activates Starwell mode
3. Background worker connects to native binary via `--stream`
4. User types character 'c'
5. Content script → Background → Binary
6. Binary generates output chars (e.g., "X9#")
7. Binary → Background → Content script
8. Content script appends "X9#" to password field (with NFC normalization)
9. Repeat for each character

### Security Features

- **No complete password in memory**: Characters streamed one-by-one
- **Immediate zeroing**: Each character zeroed after emission
- **NFC normalization**: Applied by extension, not binary (keeps backend pure)
- **Local only**: No network communication, everything local

## Troubleshooting

### Extension shows "Failed to connect to native binary"

1. Check that the binary path in the native host JSON is correct
2. Ensure the binary is executable: `chmod +x /path/to/starwell_password_manager`
3. Test the binary directly: `./target/release/starwell_password_manager --stream`
4. Check extension ID matches in native host JSON

### Password field doesn't activate

1. Make sure you're focused on a `type="password"` field
2. Check browser console for errors (F12 → Console)
3. Verify extension is enabled in brave://extensions/

### Characters appear wrong

1. Check if website normalizes Unicode differently
2. Try using ASCII-only character set (option 1 when creating shape)

## Development

### Testing without extension

You can test the streaming mode directly:

```bash
./target/release/starwell_password_manager --stream
# Type characters and watch password appear in real-time
```

### Viewing extension logs

- **Content script**: Open page, press F12, check Console
- **Background worker**: Go to brave://extensions/, click "service worker" link
- **Binary**: Check terminal output if running manually

## Files

- `manifest.json` - Extension configuration
- `scripts/content.js` - Runs on web pages, detects password fields
- `scripts/background.js` - Service worker, handles native messaging
- `scripts/popup.js` - Popup UI logic
- `popup.html` - Extension popup UI
- `native-host/com.starwell.password_manager.json` - Native messaging configuration

## TODO

- [ ] Add placeholder icons (currently missing)
- [ ] Implement site-specific normalization rules
- [ ] Add emoji filtering for sites that reject them
- [ ] Account selection UI (currently uses active/first shape)
- [ ] Password strength indicator
- [ ] Settings page for configuration
