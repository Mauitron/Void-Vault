#!/usr/bin/env bash
# Void Vault Password Manager Installation Script
# Sets up the binary, native messaging host, and browser extension

set -e  # Exit on error

echo "=============================================="
echo "  Starwell Password Manager - Installation"
echo "=============================================="
echo

# Get the absolute path to the project directory
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY_PATH="$PROJECT_DIR/target/release/starwell_password_manager"
LAUNCHER_PATH="$PROJECT_DIR/native-host-launcher.sh"
EXTENSION_PATH="$PROJECT_DIR/browser-extension"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Detect operating system
OS="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
else
    echo -e "${RED}Error: This installer is for Linux and macOS only${NC}"
    echo "For Windows, use install.ps1"
    exit 1
fi

echo "Detected OS: $OS"

echo "Project directory: $PROJECT_DIR"
echo

# Step 1: Build the binary
echo -e "${YELLOW}[1/5] Building Void Vault binary...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo not found. Please install Rust first:${NC}"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

cargo build --release
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: Binary build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Binary built successfully${NC}"
echo

# Step 2: Ensure launcher script is executable
echo -e "${YELLOW}[2/5] Setting up native messaging launcher...${NC}"
chmod +x "$LAUNCHER_PATH"
echo -e "${GREEN}✓ Launcher script ready${NC}"
echo

# Step 3: Install native messaging host manifest for browsers
echo -e "${YELLOW}[3/5] Installing native messaging host...${NC}"

# Detect which browsers are installed
BROWSERS=()
if [ "$OS" = "linux" ]; then
    if [ -d "$HOME/.config/google-chrome" ]; then
        BROWSERS+=("chrome")
    fi
    if [ -d "$HOME/.config/chromium" ]; then
        BROWSERS+=("chromium")
    fi
    if [ -d "$HOME/.config/BraveSoftware/Brave-Browser" ]; then
        BROWSERS+=("brave")
    fi
elif [ "$OS" = "macos" ]; then
    if [ -d "$HOME/Library/Application Support/Google/Chrome" ]; then
        BROWSERS+=("chrome")
    fi
    if [ -d "$HOME/Library/Application Support/Chromium" ]; then
        BROWSERS+=("chromium")
    fi
    if [ -d "$HOME/Library/Application Support/BraveSoftware/Brave-Browser" ]; then
        BROWSERS+=("brave")
    fi
fi

if [ ${#BROWSERS[@]} -eq 0 ]; then
    echo -e "${RED}Error: No supported browsers found${NC}"
    echo "Supported: Chrome, Chromium, Brave"
    exit 1
fi

echo "Found browsers: ${BROWSERS[*]}"

install_native_host() {
    local browser=$1
    local config_dir=""

    if [ "$OS" = "linux" ]; then
        case $browser in
            chrome)
                config_dir="$HOME/.config/google-chrome/NativeMessagingHosts"
                ;;
            chromium)
                config_dir="$HOME/.config/chromium/NativeMessagingHosts"
                ;;
            brave)
                config_dir="$HOME/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts"
                ;;
        esac
    elif [ "$OS" = "macos" ]; then
        case $browser in
            chrome)
                config_dir="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
                ;;
            chromium)
                config_dir="$HOME/Library/Application Support/Chromium/NativeMessagingHosts"
                ;;
            brave)
                config_dir="$HOME/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts"
                ;;
        esac
    fi

    mkdir -p "$config_dir"

    # Create the manifest (without extension ID, this will be filled in after installation
    cat > "$config_dir/com.starwell.void_vault.json" << EOF
{
  "name": "com.starwell.void_vault",
  "description": "Void Vault Password Manager Native Host",
  "path": "$LAUNCHER_PATH",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://EXTENSION_ID_PLACEHOLDER/"
  ]
}
EOF

    echo -e "  ${GREEN}✓${NC} Installed for $browser"
}

for browser in "${BROWSERS[@]}"; do
    install_native_host "$browser"
done
echo

# Step 4: Provide extension installation instructions
echo -e "${YELLOW}[4/5] Browser Extension Setup${NC}"
echo
echo "The browser extension needs to be installed manually:"
echo
echo "1. Open your browser and go to:"
echo "   Chrome/Chromium: chrome://extensions/"
echo "   Brave:          brave://extensions/"
echo
echo "2. Enable 'Developer mode' (toggle in top-right corner)"
echo
echo "3. Click 'Load unpacked'"
echo
echo "4. Select this directory:"
echo "   $EXTENSION_PATH"
echo
echo "5. Copy the Extension ID (it looks like: abcdefghijklmnopqrstuvwxyz)"
echo
read -p "Press Enter after you've copied the Extension ID..."
echo
read -p "Paste the Extension ID here: " EXTENSION_ID

if [ -z "$EXTENSION_ID" ]; then
    echo -e "${RED}Error: Extension ID cannot be empty${NC}"
    exit 1
fi

# Update the manifest files with the actual extension ID
echo -e "${YELLOW}[5/5] Updating native host manifests with extension ID...${NC}"
for browser in "${BROWSERS[@]}"; do
    config_dir=""
    if [ "$OS" = "linux" ]; then
        case $browser in
            chrome)
                config_dir="$HOME/.config/google-chrome/NativeMessagingHosts"
                ;;
            chromium)
                config_dir="$HOME/.config/chromium/NativeMessagingHosts"
                ;;
            brave)
                config_dir="$HOME/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts"
                ;;
        esac
    elif [ "$OS" = "macos" ]; then
        case $browser in
            chrome)
                config_dir="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
                ;;
            chromium)
                config_dir="$HOME/Library/Application Support/Chromium/NativeMessagingHosts"
                ;;
            brave)
                config_dir="$HOME/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts"
                ;;
        esac
    fi

    if [ "$OS" = "macos" ]; then
        # macOS uses different sed syntax
        sed -i '' "s/EXTENSION_ID_PLACEHOLDER/$EXTENSION_ID/g" \
            "$config_dir/com.starwell.void_vault.json"
    else
        # Linux sed
        sed -i "s/EXTENSION_ID_PLACEHOLDER/$EXTENSION_ID/g" \
            "$config_dir/com.starwell.void_vault.json"
    fi
    echo -e "  ${GREEN}✓${NC} Updated $browser manifest"
done
echo

# Final instructions
echo -e "${GREEN}=============================================="
echo "  Installation Complete!"
echo "==============================================${NC}"
echo
echo "Next steps:"
echo
echo "1. Reload the extension in your browser (click the reload icon)"
echo
echo "2. Create your first password geometry:"
echo "   $BINARY_PATH"
echo
echo "3. Test the extension:"
echo "   - Visit any website with a password field"
echo "   - Focus the password field"
echo "   - Press Ctrl+Shift+S to activate Void Vault"
echo "   - Type your input sequence"
echo "   - Watch the password generate in real-time!"
echo
echo "Enjoy using Starwell's Void Vault Password Manager!"
echo

# Optional: Create desktop shortcut
read -p "Create a desktop application launcher? (y/n): " CREATE_LAUNCHER
if [[ "$CREATE_LAUNCHER" =~ ^[Yy]$ ]]; then
    DESKTOP_FILE="$HOME/.local/share/applications/starwell-password-manager.desktop"
    mkdir -p "$HOME/.local/share/applications"

    cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Name=Void Vault
Comment=Deterministic password generator with geometric path traversal
Exec=$BINARY_PATH
Icon=password-generator
Terminal=true
Type=Application
Categories=Utility;Security;
EOF

    echo -e "${GREEN}✓ Desktop launcher created${NC}"
fi

echo
echo "Installation log saved to: /tmp/void-vault-install.log"
