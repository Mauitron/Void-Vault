![void-vault-static-crt](https://github.com/user-attachments/assets/4116b3b7-2b20-4b33-bdb2-54b4f463b4f5)

**Zero storage. Zero trust. Zero compromise.**

The Void Vault is a component of a larger project called Starwell (TBA).
I am releasing this stripped down version as a standalone for a handfull
of reasons. 

Firstly, one of the goals of the **Void Vault** is to make ultra secure
passwords approachable and easy to use for people who are vulnerable.
By turning easy to remember inputs into super secure outputs, elderly
and memory deficient people (like me) can feel safe and protected without
having to remember multiple passwords, or changing passwords and forgetting them 

Secondly, to allow you to find any vulnerablities or exploits that I might have
missed during development. To validate the 'unique' security model Void Vault
provides. 

Thirdly, Simlpy to get feedback on the user experience and suggestions.

As far as I can tell the Void Vault is new solution to password management
that uses **geometric path traversal** to generate high-entropy passwords from
memorable input sequences. solving password management not through encrypted
storage and hidden secrets, but through never storing anything at all.
The Void Vault is by its very nature 2-factor, using something you have
(your binary) and somthing you know (your inputs) to generate your ultra secure
passwords on demand instead of storing and copying them. 

**NOTE:** There will be a fair amount of repitition, this can be annoying
          but it is very much intentional. Void Vault does not work like a
          normal password manager, and I want to be super clear on the points
          I feel are important. I find that explaining new things in multiple
          but slightly different ways increases the chance that people will
          remember and understand. 
          
## What makes Void Vault different?

Unlike traditional password managers, Void Vault does not **hide your passwords**,
instead it generates them deterministically through navigating your own unique
multidimensional geometry created during setup.

## Features
- **Security from Non-existence**: Just like it's namesake, when you open
  the vault, there are no passwords to be found. Nothing to steal.  
- **Weak Input into Strong Output**: Simple inputs like "password123" produce
  ultra-secure passwords.
- **Zero Dependencies**: Back-end is pure Rust standard library only. making
  it completely auditable.
- **Ment to be yours**: No APIs or cloud service. Each user has their own
  unique geometry, locally on their device (no mass breach risk).
  **IMPORTANT** Make sure you make a backup of the binary to a safe location.
  there is no way to recover your passwords if you delete your binary and
  don't have it backed up anywhere. 
- **Browser Extension**: Real-time password generation for use in the browser.
  allowing you to toggle on Void Vault and start outputting through it.
- **Domain-Specific Passwords**: The extension makes different passwords on
  different websites
- **Deterministic Generation**: Same input + geometry = same password (always)
- **Real-time Streaming**: Passwords generated character-by-character.

### Why Simple Inputs Work

Also unlike traditional password managers, simple inputs like "summerhouse" 
are actually secure because:

1. **Your geometry is unique**: Even "password123" through your binary 
   produces outputs no one else can generate

2. **No mass breach risk**: Attackers must target you individually, 
   which isn't economical for random users

3. **No master password**: Nothing to crack or phish


## Installation

Void Vault works on **Linux**, should work on **Windows**, will work on **macOS**!

### Quick Install

#### Linux / macOS
```bash
git clone https://github.com/YOUR_USERNAME/void-vault.git
cd void-vault
./install.sh
```

#### Windows
```powershell
git clone https://github.com/YOUR_USERNAME/void-vault.git
cd void-vault
.\install.bat
```
*Or double-click `install.bat` in File Explorer*

The installer will:
1. Build the Rust binary (auto-detects your platform)
2. Install to system location (`~/.local/bin`, `~/Library/...`, or `%LOCALAPPDATA%`)
3. Set up native messaging for your browser(s)
4. Guide you through extension installation
5. Hopefully, configure everything automatically
6. If this does not work, you can follow the manual steps below
### Manual Installation

<details>
<summary>Click to expand manual steps</summary>

#### 1. Build the Binary

```bash
cargo build --release
```

The binary will be at: `target/release/void_vault`
(or `void_vault.exe` on Windows)

#### 2. Install Native Messaging Host

**Linux - Brave:**
```bash
mkdir -p ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts
cp browser-extension/native-host/com.starwell.void_vault.json ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/
```

**Linux - Chrome:**
```bash
mkdir -p ~/.config/google-chrome/NativeMessagingHosts
cp browser-extension/native-host/com.starwell.void_vault.json ~/.config/google-chrome/NativeMessagingHosts/
```

**macOS - Brave:**
```bash
mkdir -p ~/Library/Application\ Support/BraveSoftware/Brave-Browser/NativeMessagingHosts
cp browser-extension/native-host/com.starwell.void_vault.json ~/Library/Application\ Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/
```

**Windows - Chrome:**
```
C:\Users\<USERNAME>\AppData\Local\Google\Chrome\User Data\NativeMessagingHosts\
```

Edit the manifest file to:
1. Point to your binary location (update the "path" field)
2. Add your extension ID (see next step)

#### 3. Install Browser Extension

1. Open browser: `brave://extensions/` or `chrome://extensions/`
2. Enable "Developer mode" (top right)
3. Click "Load unpacked"
4. Select the `browser-extension/` directory
5. Copy the Extension ID shown
6. Edit the native host manifest and add the extension ID to the
   "allowed_origins" array, like so: `chrome-extension://[PUT IT HERE]/`


</details>

## Usage

### First-Time Setup

When you first run the binary, it will guide you through a simple 3-step setup:

```bash
./target/release/void_vault
```

**Step 1: Choose Password Style** (Can not be changed once set)
- Option 1: For shorter input passwords (1-8 characters) → Longer passwords
- Option 2: For longer input passwords (9-16 characters) → Balanced passwords
  NOTE: Most websites have a max allowed password length of 64-124 characters
        If you are to use Void Vault for general use, I recommend choosing
        option 2, as it will give you more freedom to use longer inputs
        without exceeding 64 bytes. 
        
**Step 2: Choose Character Set**
- Standard (95 chars): ASCII printable characters `!` to `~`
           Should work everywhere, but low pool size results in less password
           strength.

- Extended (300+ chars): Includes common special characters. (recommended)
           should work in most places and allows for very secure passwords.

- Full (5000+ chars): Maximum Unicode coverage, includes everything.
                      This is the least supported option, most websites will
                      reject emojis and such. If you are planning on using this
                      application for a domain you know supports the full UTF8
                      character set, this option will generate very secure
                      passwords.

**Step 3: Create Your Structure**
- Type a phrase, any phrase will do (40+ characters recommended)
  NOTE: As a rule of thumb, the longer the phrase the better
- The system uses this phrase to help make the Void Vault **Unique to You**
- **Type naturally**, write a story if you want!

You're now ready to generate deterministic passwords!
**BUT MAKE A BACKUP BEFORE YOU DO!**

### Binary Self-Modification

When you complete the setup, Void Vault modifies its own executable file to 
embed your unique geometry within itself. This is why:
- Backing up the binary is so important, its backing up your vault
- The binary file grows slightly after setup
- A .bak file is automatically created during saves to the binary

### Interactive Password Generation

After setup, running the binary gives you a simple interface:

```bash
./target/release/void_vault
```

```
=== VOID VAULT ===
Active configuration: main

Enter your password phrase (or 'exit' to quit):
```

Type any common input sequence (e.g., "Maui is super pretty") and it will
generate a unique password:

```
Generated password (N characters):
ē₹Įő$[k₩ł∂Ʊ...
```

I know I am repeating myself but the fundamental security model is important to
understand. the same input will always produce the same output.
Different inputs produce completely different outputs.
Different binaries produce a completely different output even if the input is
the same.

### Using the Browser Extension

1. **Visit any website** with a password field (e.g., gmail.com)

2. **Focus the password field** (click on it)

3. **Press `Ctrl+Shift+S`** to activate Void Vault
   - Field turns green
   - Shows "Void Vault active - type your phrase..."

4. **Type your input sequence** (e.g., "myPassword")
   - Password generates in real-time as you type
   - Each character you type produces multiple output characters
   - Domain tracking ensures gmail.com gets a completel different password
     than facebook.com, even if you use the same input.

5. **Press Enter** to finalize (Ctrl+Shift+S again or Escape to cancel)

**Important**: as mentioned above, the same input on different domains produces
               different passwords:
- "mypass" on gmail.com → `ē₹Įő$[k₩ł∂Ʊ...`
- "mypass" on facebook.com → `₿Ψ∞≠±∑∆√∫...`

### Command-Line Modes

#### Interactive Mode (Default)
```bash
./target/release/void_vault
```
Simple prompt for entering passwords. Type input, get password.

#### Terminal Demo Mode
```bash
./target/release/void_vault --term
```
Shows character-by-character generation in real-time. Great for seeing the
system in action! And for you to experiment / analyze the output

#### I/O Mode (for scripts)
```bash
echo "myinput" | ./target/release/void_vault --io
```
Pipe input, get password on stdout.

#### JSON I/O Mode (for native messaging)
```bash
./target/release/void_vault --json-io
```
Chrome native messaging protocol (used automatically by browser extension).

## How It Works

### Password Generation Flow

1. **User types input**: "password123"
2. **Domain specific modification applied** (browser extension only):
3. **Geometry navigation**:
   - System starts at origin of your structure
   - Each input character triggers navigation through the geometry
   - Path is deterministic but very complex and path dependant
4. **Character generation**:
   - Each input produces 4-8 output characters (depending on your setup choice)
   - Output generated from continuous position and its previous movements 
5. **Path-dependent**:
   - Each character input affects all subsequent characters (avalanche effect)
   - Hash-like behavior: small input changes results in completely different
     outputs

### More Technical Security Properties (Repeating myself)

- **Entropy Amplification**: Weak input + structure configuration +
  multiple outputs results in a much stronger final passwords
- **Path Dependency**: Character N depends on ALL previous characters (hash-like)
- **No Reversibility**: Cannot derive input from output (one-way function)
- **Behavioral Uniqueness**: Your instance of Void Vault is unique to you!
- **Domain Separation**: Same input produces different passwords per website
- **Single-Target Architecture**: Each user has owns their own Void Vault (no mass breach risk)
- **Zero Storage**: Passwords never stored, as the name suggest, all the vault
  stores is a void. It does not care or remember your passwords, they are only generated
  on-demand, when you need them. And are based on something you have
  (your instance of the application), something you know (your inputs/targets)
- **Memory Zeroing**: Sensitive data is both fragmented and transitional, it's
  cleared from memory after use

### Why This Is Secure

**Traditional password managers** rely on:
- Master password strength (user responsibility)
- Encryption implementation (trust required)
- Storage security (cloud/local breach risk)

**Void Vault** relies on:
- Your unique structure generated during setup
- That you make backup(s) of your binary once setup is complete
- Deterministic outputs based on movement through an extremely complex structure

### Communication Protocol

Browser extension sends characters one at a time:
```json
{"char": "a"}
{"char": "b"}
{"char": "c"}
{"type": "FINALIZE"}
```

Binary responds with generated characters:
```json
{"output": "ē₹Įő$"}
{"output": "k₩ł∂Ʊ"}
{"output": "∫∂√≠±"}
```

## Files

```
void-vault/
├── src/
│   ├── main.rs              Core password generation engine
├── browser-extension/
│   ├── manifest.json       Extension configuration
│   ├── popup.html          Extension UI
│   ├── setup.html          Setup wizard
│   └── scripts/
│       ├── background.js    Native messaging handler
│       ├── content.js       Password field injection
│       ├── popup.js         Popup logic
│       ├── setup.js         Setup wizard logic
│       └── domain-shift.js  Domain-based input shifting
├── install.sh              Linux/macOS installer
├── install.ps1             Windows PowerShell installer
├── install.bat             Windows batch wrapper
├── native-host-launcher.sh Chrome native messaging launcher
└── Cargo.toml              Rust project configuration
```

## Security Considerations

### What Void Vault Protects Against
-  **Password database breaches** (nothing stored)
-  **Weak master passwords** (security from geometry, not only from input)
-  **Password reuse** (different password per domain)
-  **Brute force attacks** (The geometric approach creates huge variation)
-  **Pattern analysis** (path-dependent generation)
-  **Mass attacks** (each user has unique shape)

### What Void Vault Does NOT Protect Against
- **Binary theft**
  attacker with your binary could generate your output given they
  know your input and target.
- Mitigation: Don't give your vault to someone else, and then tell them all
  your input passwords and the domains they target. 

- **Memory dumps**
  portions of the generated output is briefly in memory during generation
- Mitigation: Memory is zeroed after activation, but brief windows exists

- **OS-level keyloggers** (captures your input as you type)
- Mitigation: None yet (they would need you vault for it to be usefull though)

- **Phishing (Propably)** (user enters password on fake site)
- Mitigation: Browser extension checks domain so it should not get the 
  password to the real website.

### Threat Model

**Strong security assumptions:**
- Your computer is not compromised (no malware, no keyloggers)
- Your binary file is private (stored on a local disk + backup elsewhere)
- You need to be targeted individually as the vault is unique to you,
  it is not on the cloud unless you but it there yourself.
- You use the browser extension (provides domain verification)

**Weaker assumptions:**
- Your input sequences can be simple (security comes from geometry)
- You sure at least a 8-12 char input password / phrase
- You don't need to remember complex passwords

## Licensing

Void Vault is **dual-licensed**:

### Open Source (AGPL-3.0)
**FREE** for:
- Personal use
- Small businesses (<$100k revenue/year)
- Non-profits and educational institutions
- Open-source projects

**Requirements**:
- Source code modifications must be shared (AGPL-3.0 copyleft)
- Network use = distribution (must share modifications)

### Commercial License
**REQUIRED** for:
- Organizations with >$100k annual revenue
- Closed-source products integrating Void Vault
- SaaS products using Void Vault
- Commercial redistribution without source code sharing

**Pricing**: See [COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md)

**Contact**: licensing@starwell.se

## Privacy

Void Vault collects **ZERO data**. See [PRIVACY.md](PRIVACY.md) for full policy.

- No telemetry
- No analytics
- No cloud services
- No network requests
- 100% offline operation

## FAQ

### Can I use this to replace my current password manager?

Yes! Use the browser extension for new accounts. For existing accounts, you can:
1. Generate a Void Vault password
2. Change your account password to the generated one
3. Remember your input sequence

### What if I forget my input sequence?

There's no recovery mechanism. The security model relies on you remembering
simple inputs (things like "summerof45" or "mynameisjohn" ). Strong inputs are
of course better if your binary is stolen, same is true with long inputs, but
choose inputs memorable to you! that is what the Void Vault is built for. 

### Can I use this on multiple computers?

Yes, but currently not easily. Your geometry is stored in the binary.
To use on another computer you would need to copy the binary to the other
machine. In a future version of Void Vault you will however be able to make
any normal storage device a key that you can bring with you. like a thumb drive
for example.

### Is the Windows version tested?

The code includes full Windows support, with Console API for terminal, native
messaging detection and such, but has not been extensively tested.
Please report issues you might find!

### How do I update Void Vault?
This is a slimmed down version of the original Void Vault, it is designed to
become completed. What this means for you is that updates to the actual backend
will only happen if absolutely needed. development will always trend towards
a state where it would never need to be updated again.
The full version of Void Vault is part of a larger project called Starwell.

With all that said, if bugs or security related problems arise and an update
is needed, I then recommend that you reset your passwords temporarily using
your current Void Vault, to simple normal passwords and then install the new
version of the vault.

Updates are more likely to happen on the extension side of the application
which will not impact your passwords at all.

But again, if it needs to be done, these are the condensed recommended steps:
1. If you don't have a backup, copy the binary to a safe location
2. use your old binary to reset passwords to normal ones temporarily
3. Build/download new binary
4. Use the new binary to set new secure Void Vault passwords.
5. Live long and prosper 


## Disclaimer

This is **beta software**. While the security principles are sound,
the implementation has not undergone any formal security audit.
Use at your own risk.

For production use, consider:
- Using strong input sequences
- Backing up your binary safely
- Reviewing the source code

---

**License**: AGPL-3.0 (open source) + Commercial
**Contact**: Maui_The_Magnificent@proton.me

