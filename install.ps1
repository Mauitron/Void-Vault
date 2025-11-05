# Void Vault Password Generator
# PowerShell script for automated installation on Windows

#Requires -Version 5.1

# Stop on errors
$ErrorActionPreference = "Stop"

Write-Host "=============================================="
Write-Host "  Void Vault Password Manager - Installation"
Write-Host "=============================================="
Write-Host ""

# Get the absolute path to the project directory
$ProjectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$BinaryName = "void_vault.exe"
$BinaryPath = Join-Path $ProjectDir "target\release\$BinaryName"
$ExtensionPath = Join-Path $ProjectDir "browser-extension"

# Colors for output (using Write-Host -ForegroundColor)
function Write-Success { Write-Host $args[0] -ForegroundColor Green }
function Write-Warn { Write-Host $args[0] -ForegroundColor Yellow }
function Write-Err { Write-Host $args[0] -ForegroundColor Red }

Write-Host "Project directory: $ProjectDir"
Write-Host ""

# Step 1: Check for Rust/Cargo
Write-Warn "[1/5] Checking for Rust toolchain..."
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Err "Error: Rust/Cargo not found."
    Write-Host "Please install Rust first from: https://rustup.rs"
    Write-Host ""
    Write-Host "Quick install:"
    Write-Host "  Open PowerShell and run:"
    Write-Host '  Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"'
    Write-Host "  .\rustup-init.exe"
    exit 1
}
Write-Success "✓ Rust found"
Write-Host ""

# Step 2: Build the binary
Write-Warn "[2/5] Building Void Vault binary..."
Push-Location $ProjectDir
try {
    cargo build --release
    if (-not (Test-Path $BinaryPath)) {
        Write-Err "Error: Binary build failed"
        exit 1
    }
    Write-Success "✓ Binary built successfully"
} finally {
    Pop-Location
}
Write-Host ""

# Step 3: Install binary to user directory
Write-Warn "[3/5] Installing binary..."
$InstallDir = Join-Path $env:LOCALAPPDATA "Starwell"
$InstalledBinaryPath = Join-Path $InstallDir $BinaryName

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Copy-Item $BinaryPath $InstalledBinaryPath -Force
Write-Success "✓ Binary installed to: $InstalledBinaryPath"
Write-Host ""

# Step 4: Detect browsers and install native messaging hosts
Write-Warn "[4/5] Installing native messaging host..."

$Browsers = @()
$ChromePath = Join-Path $env:LOCALAPPDATA "Google\Chrome\User Data\NativeMessagingHosts"
$BravePath = Join-Path $env:LOCALAPPDATA "BraveSoftware\Brave-Browser\User Data\NativeMessagingHosts"
$EdgePath = Join-Path $env:LOCALAPPDATA "Microsoft\Edge\User Data\NativeMessagingHosts"

if (Test-Path (Join-Path $env:LOCALAPPDATA "Google\Chrome")) {
    $Browsers += @{Name="Chrome"; Path=$ChromePath}
}
if (Test-Path (Join-Path $env:LOCALAPPDATA "BraveSoftware\Brave-Browser")) {
    $Browsers += @{Name="Brave"; Path=$BravePath}
}
if (Test-Path (Join-Path $env:LOCALAPPDATA "Microsoft\Edge")) {
    $Browsers += @{Name="Edge"; Path=$EdgePath}
}

if ($Browsers.Count -eq 0) {
    Write-Err "Error: No supported browsers found"
    Write-Host "Supported: Chrome, Brave, Edge"
    exit 1
}

Write-Host "Found browsers: $($Browsers.Name -join ', ')"
Write-Host ""

# Function to create native host manifest
function Install-NativeHost {
    param($BrowserName, $ManifestPath, $ExtensionId)

    if (-not (Test-Path $ManifestPath)) {
        New-Item -ItemType Directory -Path $ManifestPath -Force | Out-Null
    }

    $ManifestFile = Join-Path $ManifestPath "com.starwell.void_vault.json"

    # Create manifest JSON
    $Manifest = @{
        name = "com.starwell.void_vault"
        description = "Void Vault Password Manager Native Host"
        path = $InstalledBinaryPath
        type = "stdio"
        allowed_origins = @("chrome-extension://$ExtensionId/")
    }

    $Manifest | ConvertTo-Json | Set-Content -Path $ManifestFile -Encoding UTF8
    Write-Success "  ✓ Installed for $BrowserName"
}

# Step 5: Browser extension installation
Write-Warn "[5/5] Browser Extension Setup"
Write-Host ""
Write-Host "The browser extension needs to be installed manually:"
Write-Host ""
Write-Host "1. Open your browser and go to:"
Write-Host "   Chrome: chrome://extensions/"
Write-Host "   Brave:  brave://extensions/"
Write-Host "   Edge:   edge://extensions/"
Write-Host ""
Write-Host "2. Enable 'Developer mode' (toggle in top-right corner)"
Write-Host ""
Write-Host "3. Click 'Load unpacked'"
Write-Host ""
Write-Host "4. Select this directory:"
Write-Host "   $ExtensionPath"
Write-Host ""
Write-Host "5. Copy the Extension ID (looks like: abcdefghijklmnopqrstuvwxyz)"
Write-Host ""

Read-Host "Press Enter after you've copied the Extension ID"
Write-Host ""

$ExtensionId = Read-Host "Paste the Extension ID here"

if ([string]::IsNullOrWhiteSpace($ExtensionId)) {
    Write-Err "Error: Extension ID cannot be empty"
    exit 1
}

# Update all browser manifests with the extension ID
Write-Host ""
Write-Host "Updating native host manifests with extension ID..."
foreach ($Browser in $Browsers) {
    Install-NativeHost -BrowserName $Browser.Name -ManifestPath $Browser.Path -ExtensionId $ExtensionId
}

Write-Host ""
Write-Success "=============================================="
Write-Success "  Installation Complete!"
Write-Success "=============================================="
Write-Host ""
Write-Host "Next steps:"
Write-Host ""
Write-Host "1. Reload the extension in your browser (click the reload icon)"
Write-Host ""
Write-Host "2. Create your first password shape:"
Write-Host "   Run: $InstalledBinaryPath"
Write-Host "   Or:  cd $InstallDir"
Write-Host "        .\$BinaryName"
Write-Host ""
Write-Host "3. Test the extension:"
Write-Host "   - Visit any website with a password field"
Write-Host "   - Focus the password field"
Write-Host "   - Press Ctrl+Shift+S to activate Void Vault"
Write-Host "   - Type your input sequence"
Write-Host "   - Watch the password generate in real-time!"
Write-Host ""
Write-Success "Enjoy using Void Vault Password Generator!"
Write-Host ""

# Optional: Add to PATH
Write-Host ""
$AddToPath = Read-Host "Add Void Vault to PATH? (y/n)"
if ($AddToPath -eq 'y' -or $AddToPath -eq 'Y') {
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir", "User")
        Write-Success "✓ Added to PATH. Restart your terminal to use 'void_vault' command."
    } else {
        Write-Host "Already in PATH."
    }
}

Write-Host ""
Write-Host "Installation log: This PowerShell session"
