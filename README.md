![void-vault-static-crt](https://github.com/user-attachments/assets/36736477-1e49-43b9-b6ad-0b9b0f2f236a)`
[![Windows Support](https://img.shields.io/badge/Windows-Should%20Work-yellowgreen?logo=windows&logoColor=white "Works on Windows but has limited testing")](docs/windows-support.md)
![Linux Support](https://img.shields.io/badge/Linux-Supported-success?logo=linux&logoColor=white)
![Status](https://img.shields.io/badge/Status-Beta-yellow)
## Zero storage. Zero trust. Zero compromise.

#### Void Vault is considered as being in beta. During beta, internal optimizations (memory management, static allocation) are being finalized, and additional features might be implemented. Once complete, Void Vault will exit beta.

**Post-beta promise**: Void Vault is designed to become *completed software*, meaning it reaches a point where the code no longer needs ongoing work. When Void Vault exits beta, the binary geometry generation will be frozen. No changes will be made to the Rust code unless a critical security vulnerability is discovered. If such a vulnerability requires updating the geometry algorithm, I will build and provide a migration tool to update your binary without losing access to your passwords.

**During beta**: If geometry algorithm changes are needed, your old binary will no longer generate the correct passwords. You'll need to reset your website passwords to temporary values, then use the updated binary to generate fresh Void Vault passwords.

The Void Vault is a component of a larger project called Starwell (TBA).
I am releasing this stripped down version as a standalone to get it tested
and picked apart by others. 

In short, Void Vault is a password manager that doesn't store passwords, it generates them
deterministically through changing what you type into geometric navigation of a unique 
7-dimensional space. It is that multi-dimensional geometry that transforms your inputs
into highly random and complex outputs.

---

## The Problem with Traditional Password Managers

**Storage = Risk.** Every password manager that stores your passwords is a target:
- Encrypted databases can be stolen and attacked offline
- Master passwords are single points of failure
- Database breaches expose all your passwords at once
- Changing passwords means updating storage everywhere

---

## What Makes Void Vault Different?

### 1. **Nothing to Steal**
Your passwords don't exist until the moment you generate them. Open the vault and you'll find exactly what the name promises: a void. No encrypted database, no stored secrets, just a unique geometric structure that transforms simple inputs into secure outputs.

### 2. **Simple Inputs, Secure Outputs**
Type "summerhouse" and get `ē₹Įő$[k₩ł∂Ʊ∫∂√≠±...`. Your input is just a navigation path through your personal 7D geometry, the security comes from your unique vault structure, not from memorizing random characters.
![recording-20251119-112008](https://github.com/user-attachments/assets/14897637-b9c0-4086-9d5b-095122767acd)

### 3. **Password Versioning Without Changing Inputs**
Need to change a password? Don't change your input phrase, just increment the version counter. Same input, completely different output:
- `summerhouse` at v0 → `ē₹Įő$[k₩ł∂Ʊ...`
- `summerhouse` at v1 → completely different password
- `summerhouse` at v2 → different again

![recording-20251119-112419](https://github.com/user-attachments/assets/294df2e0-9d56-470f-8ff2-a04ee27762c8)

**Test before you commit** with preview mode, see what the next version will be before saving it.

### 4. **Each User is Unique**
Even if you and I both use "password123" on the same website, our outputs are completely different. Your geometry is unique to you, generated during setup from a phrase you provide. No two vaults are alike.

### 5. **Automatic Per-Domain Security**
- Passwords are automatically different on every website
- Domain names are hashed and irreversible (even you can't see your domain list)
- Per-site password rules (length limits, character restrictions) handled automatically
- Attackers can't get your Netflix password from your Gmail password, they're generated from different positions in 7D space

### 6. **No Master Password**
Void Vault doesn't use a "master password." It's a **deterministic input substitution function**. Press 'S' on your keyboard, and deterministic output appears in real time. Your security comes from:
- Your unique geometry (stored in the binary)
- Your memorable input phrases (stored in your memory)
- The combination of both

---

## Key Features

- **Real-time Generation**: Type naturally, passwords appear character-by-character
- **Password Versioning**: Change passwords without changing your inputs (v0→v65535)
- **Preview Mode**: Test new password versions before committing
- **Browser Extension**: Auto-detects password fields, manages versions, applies site rules
- **Per-Domain Rules**: Automatically adapts to website requirements (max length, character types)
- **Domain Privacy**: Domain names are geometrically hashed, cannot be reversed
- **Complete Portability**: One binary file contains everything, geometry, counters, and all site rules
- **Zero Dependencies**: Pure Rust backend, no external libraries
- **Self-Modifying Binary**: Your geometry lives in the executable itself
---

## How It Works (Simple Version)

1. **Setup**: You create your unique 7D geometry by typing a long phrase (40+ characters recommended)
2. **Activation**: Click a password field, activate Void Vault (Ctrl+Shift+S)
3. **Input**: Type a simple, memorable phrase like "myfirstdog"
4. **Navigation**: Your input navigates through your unique 7D space
5. **Output**: Each step generates secure password characters in real time
6. **Different Domain**: Same input on a different site? Different position in space = different password

**Bidirectional Temporal Dependency**: Future keypresses depend on past ones, and past outputs change based on what comes after. "abc" produces completely different output than "abcd".
![recording-20251119-112827](https://github.com/user-attachments/assets/c1c38654-1bcd-4dc6-85f4-e4b59f1eb4c9)

---

## Installation

### Quick Install

**Linux / macOS:**
```bash
git clone https://github.com/Mauitron/void-vault.git
cd void-vault
./install.sh
```

**Windows:**
```bash
git clone https://github.com/Mauitron/void-vault.git
cd void-vault
.\install.bat
```
Or double-click `install.bat` in File Explorer

The installer will:
- Build the Rust binary (auto-detects your platform)
- Install to `~/void_vault/` (Linux/macOS) or `%LOCALAPPDATA%\Starwell\` (Windows)
- Set up native messaging for your browser(s)
- Guide you through extension installation
- Once the setup is done, you can close the your shell

### ⚠️ Windows Users: Important Extra Step
After setup completes, you MUST manually copy the binary from 
`target\release\void_vault.exe` to `%LOCALAPPDATA%\Starwell\`
This is due to Windows AppData folder restrictions.

---

## First-Time Setup not using the install scripts

Once you have compiled the program, run the binary to create your unique vault:

**Linux/macOS:**
```bash
~/void_vault/void_vault
```

**Windows:**
```
%LOCALAPPDATA%\Starwell\void_vault.exe
```

You'll be guided through the creation of your geometry.

### Windows Users: Extra Step Required
On Windows, after completing the setup wizard, you need to manually copy the binary:
1. Navigate to the build folder: `void-vault\target\release\`
2. Copy `void_vault.exe`
3. Paste it into: `%LOCALAPPDATA%\Starwell\`
4. Replace the existing file if prompted

This is necessary because Windows appearently restricts binary self-modification in 
the AppData folder during first setup.

### ⚠️ IMMEDIATELY BACKUP YOUR BINARY
After setup, copy your binary to a safe location:
- Linux/macOS: `~/void_vault/void_vault`
- Windows: `%LOCALAPPDATA%\Starwell\void_vault.exe`

**This binary IS your vault.** It contains:
- Your unique 7D geometry
- All domain counters (password versions)
- All site-specific rules (length limits, character types)

One file = complete portability. There is no recovery mechanism.
ideally you would make multiple backups, and put your backup(s) on a new device
or on the cloud. Point being, keep them safe.

---

## Using the Browser Extension

### Basic Usage
1. Visit any website with a password field
2. Click the password field
3. Click "Activate Vault" or press **Ctrl+Shift+S**
4. Type your memorable input phrase
5. Press **Enter** to finalize

The extension shows:
- Current domain
- Password version (v0, v1, v2, etc.)
- Preview mode indicator
- Visual feedback

### Managing Password Versions

**When you need to change a password:**

1. Activate Void Vault on the password field
2. Press **Ctrl+Shift+P** to enter Preview Mode
3. Type your input phrase, you'll see the next version (v1, v2, etc.)
4. Press **Enter** to commit the new version
5. Update your password on the website

**Or manually select a version:**
- Click the version counter in the overlay
- Select from existing versions or enter a custom number (0-65535)
- Choose "Use Temporarily" (this session only) or "Set as Default" (save it)

### Automatic Preview Mode
The extension detects "new password" or "reset password" pages and automatically enters preview mode,
suggesting you create v1 instead of using v0.

### Configuring Site-Specific Rules
Some websites have annoying restrictions (max 16 characters, no symbols, etc.):

1. Click the Void Vault extension icon
2. Click "Settings for current site"
3. Enable rules and configure:
   - Max password length
   - Allowed character types (lowercase, uppercase, digits, symbols)
4. Save

Void Vault will automatically adapt your passwords to meet these requirements.

---

## Security Model

### What Void Vault Protects Against

**Password database breaches** (nothing stored)  
**Weak master passwords** (no master password, security from geometry + input)  
**Password reuse** (automatically different per domain)  
**Mass attacks** (each user has unique geometry)  
**Brute force** (geometric approach creates massive search space)  
**Pattern analysis** (path-dependent generation resists patterns)  
**Phishing** (browser extension checks domain)  
**Password changes** (versioning system makes changes easy)

### What Void Vault Does NOT Protect Against

**Binary theft + input knowledge**: If someone gets your binary AND knows your input phrases AND knows which sites you use them on, they can generate your passwords

**Mitigation**: Don't share your binary. Use memorable but non-obvious inputs.

**OS-level keyloggers**: Captures your input as you type

**Mitigation**: They'd still need your binary for it to be useful. Use trusted devices.

**Memory dumps**: Generated passwords briefly exist in memory

**Mitigation**: Memory is zeroed after use, but brief windows exist

### The Trust Model

Void Vault's security relies on:
- **Something you have**: Your unique binary (the geometry)
- **Something you know**: Your input phrases
- **Something hidden**: Which domains you use each phrase on

An attacker needs all three to compromise your passwords. This is effectively 2-factor security by design.

---

## FAQ

### How many websites can I use this with?
Up to 512 different domains. If you have more than that... simplify your life.
Having said that, You will be able to remove domains in an upcomming version of the Void Vault

### Can I use this to replace my current password manager?
Yes! For new accounts, just use Void Vault. For existing accounts:
1. Generate a Void Vault password (v0)
2. Change your account password to it
3. Remember which input phrase you used

### What if I forget my input phrase for a domain?
There's no recovery mechanism. This is intentional, it's why Void Vault is secure. Use simple, memorable inputs. Things like "summerof45" or "myfirstdog" are fine because your security comes from the output of your unique geometry, not from the complexity of your input.

### Can I use this on multiple computers?
Yes, but you need to copy your binary to each machine. Your geometry, all domain counters (password versions), and all site-specific rules are embedded in the binary file, it's completely self-contained. Copy one file, get everything.

Future versions will support portable keys (thumb drives, etc.) for easier cross-device usage.

### How do I change a password on a website?
Don't change your input phrase! Just increment the version counter:
1. Activate Void Vault in preview mode (Ctrl+Shift+P)
2. Type the same input phrase
3. You'll see v1 (or v2, v3, etc.)
4. Press Enter to commit
5. Update your password on the website

### What happens if someone steals my binary?
Without your input phrases and knowing which sites you use them on, the binary is useless. It contains your geometry, domain hashes, password version counters, and site-specific rules, but domain hashes cannot be reversed to see which sites you have passwords for, and the counters are meaningless without knowing the domains they belong to.

### Can I see which domains I have passwords for?
No. Domain names are geometrically hashed using your vault's unique structure. Even with the binary, you can only see hash values, not domain names. This is a privacy feature, even if your binary is compromised, attackers can't see your domain list.

### Is the Windows version tested?
Yes, the Windows version is confirmed working. However, there's an extra step after setup: you need to manually copy the binary from `target\release\void_vault.exe` to `%LOCALAPPDATA%\Starwell\` due to Windows restrictions on binary self-modification in the AppData folder. Please report any other issues you find!

### Do I need to update Void Vault?
Hopefully not. This is designed to be feature-complete. Updates should only happen if critical bugs or security issues are discovered. The extension may receive updates that won't affect your passwords.

If an update is needed:
1. Back up your current binary (if you haven't already)
2. Use your old binary to temporarily reset passwords to normal ones
3. Build/download the new binary
4. Set new Void Vault passwords using the new binary

---

## Important Notes

### Beta Status
Void Vault is currently in beta. During this period, updates to the binary's geometry generation may occur. If you want to participate in these updates, it's not recommended to use Void Vault as your primary password solution yet.

### Not Metaphors
"7 dimensional", "path", "bidirectional dependency" are not marketing terms. Void Vault genuinely uses continuous movement through 7 spatial dimensions. Your binary grows by ~1.5-2 MB during setup, that's the geometry data.

### External Auditing Needed
The author makes no claims about security superiority over alternatives. The unique functionality (geometric generation, no storage) is interesting, but external security audits are needed before making strong security claims.

However: Your passwords are generated from your geometry, not stored. There's no way to extract passwords from the vault itself, it's not encryption, it's generation.

---

## Licensing

Void Vault is dual-licensed:

### Open Source (AGPL-3.0)
**FREE for:**
- Personal use
- Small businesses (<$100k revenue/year)
- Non-profits and educational institutions
- Open-source projects

**Requirements:**
- Source code modifications must be shared (AGPL-3.0 copyleft)
- Network use = distribution (must share modifications)

### Commercial License
**REQUIRED for:**
- Organizations with >$100k annual revenue
- Closed-source products integrating Void Vault
- SaaS products using Void Vault
- Commercial redistribution without source sharing

Contact: Maui_The_Magnificent@proton.me

---

## Privacy

Void Vault collects **ZERO data**:
- No telemetry
- No analytics
- No cloud services
- No network requests
- 100% offline operation

See PRIVACY.md for full policy.

---

## Contributing

***If you like Void Vault and want to support the project, please consider feeding me some [Pizza](https://buymeacoffee.com/charon0) 🍕***

Found a vulnerability or exploit? Please report it! This helps validate the security model.

---

## Disclaimer

This is beta software. While the security principles are sound,
the implementation has not undergone formal security audit.
**Use at your own risk.**

For production use, consider:
- Using strong input sequences
- Backing up your binary safely
- Reviewing the source code
- Waiting for external security audits

---

**License**: AGPL-3.0 (open source) + Commercial  
**Contact**: Maui_The_Magnificent@proton.me  
**Repository**: https://github.com/Mauitron/void-vault


