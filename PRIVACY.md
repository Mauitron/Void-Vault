# Privacy Policy for Void Vault Password Manager

**Last Updated:** January 3, 2025

## Overview

Void Vault is designed with privacy as the core principle.
This privacy policy explains our commitment to protecting your data.

**Short version:** We don't collect any data and we don't have any way to
do so. Everything happens on your device.

## Data Collection

**Void Vault collects ZERO data.**

We do not:
- Collect any personal information
- Track your usage
- Send analytics or telemetry
- Use cookies or tracking pixels
- Connect to any servers
- Store data in the cloud
- Have user accounts or authentication

## Data Storage

**All data is stored locally on your device only.**

### What is stored:
- **Password geometry:** Stored in the binary file on your computer
- **Browser extension state:** Temporary state in browser local storage
  (cleared when browser closes)

### Where data is stored:
- **Linux:** `~/.local/bin/void_vault` or in the project directory
- **macOS:** `~/Library/Application Support/Starwell/void_vault` or in the project directory
- **Windows:** `%LOCALAPPDATA%\Starwell\void_vault.exe` or in the project directory

### What is NOT stored:
-  No passwords are ever stored (they're generated on-the-fly)
-  No input sequences are stored
-  No browsing history or website data
-  No keystroke data (streams inputs)

## Network Access

**Void Vault has ZERO network access.**

The application:
-  Works completely offline
-  Does not connect to any servers
-  Does not send or receive any data over the internet
-  Does not check for updates automatically (you must manually update)

You can verify this by:
- Disconnecting from the internet and using Void Vault normally
- Reviewing the source code (it's open source)
- Using network monitoring tools (you'll see zero traffic)

## Browser Extension Permissions

The Void Vault browser extension requests these permissions:

### `nativeMessaging`
**Purpose:** Communicate with the local Void Vault binary to generate passwords
**Data sent:** Only the characters you type in the password field
**Data received:** Only the generated password characters
**Where it goes:** Only to the local binary on your computer 

### `activeTab`
**Purpose:** Detect password fields on the current webpage.
**Data accessed:** Only the current tab's password input fields.
**What we do with it:** Inject generated passwords into password fields.
**What we DON'T do:** Read other form data, track which sites you visit.

### `storage`
**Purpose:** Remember setup completion state.
**Data stored:** A single boolean flag (`setupComplete: true`) in browser local storage.
**What we do with it:** Prevent showing setup wizard repeatedly.

### `tabs`
**Purpose:** Open the setup page when the extension is first installed.
**What we do with it:** Open `setup.html` once after installation.
**What we DON'T do:** We do not track your tabs or monitor your browsing.

### Content Script (`<all_urls>`)
**Purpose:** Detect password fields on any website you visit.
**What it does:** Listens for Ctrl+Shift+S hotkey and detects password input fields.
**What it does NOT do:** Does not read other data on the page.

## Third-Party Services

**We do not currently integrate with any third-party services.**

Void Vault does not integrate with:
-  Analytics services (Google Analytics, Mixpanel, etc.)
-  Crash reporting services (Sentry, Bugsnag, etc.)
-  Cloud storage providers
-  Authentication providers
-  Payment processors (if you purchase a commercial license, you interact directly with us)
-  CDNs or external resources

## GDPR Compliance (European Users)

Void Vault is fully compliant with GDPR because:

1. **No personal data collected** - We cannot violate GDPR if we have no data
2. **No data processing** - All processing happens locally on your device
3. **No data transfers** - No data crosses borders because no data leaves your device
4. **Right to access** - All data is already on your device
5. **Right to deletion** - Delete the binary file to remove all data
6. **Right to portability** - Your binary file is the data, you can copy it anywhere

**Data Controller:** Starwell Project
**Contact:** Maui_The_Magnificent@proton.me

## Open Source Transparency

Void Vault is open source (AGPL-3.0 license). You can:
-  Review all source code: https://github.com/Mauitron/void-vault
-  Verify no data collection occurs
-  Audit the security yourself
-  Build from source if you don't trust pre-compiled binaries

## Security

### How we protect your data:
-  All data stays on your device (we can't lose what we don't have)
-  No cloud = no cloud breaches
-  No servers = no server hacks
-  No accounts = no credential leaks
-  Passwords never stored = nothing to steal

### Your responsibilities:
-  Keep your binary file secure (it contains your password structure)
-  Back up your binary file to a secure location
-  Don't share your binary file with others

## Data Retention

**We retain zero data because we collect zero data.**

Data on your device:
- **Password shapes:** Retained until you delete the binary file
- **Browser extension state:** Cleared when you uninstall the extension

## Changes to This Privacy Policy

We may update this privacy policy from time to time. Changes will be posted at:
- https://github.com/Mauitron/void-vault/blob/main/PRIVACY.md

Significant changes will be announced via GitHub releases.

## Contact Us

If you have questions about this privacy policy:

**Email:** Maui_The_Magnificent@proton.me
**GitHub Issues:** https://github.com/Mauitron/void-vault/issues
**Project Website:** https://github.com/Mauitron/void-vault

## Legal Basis (GDPR)

Our legal basis for processing (the minimal local processing that occurs):

- **Legitimate Interest:** You have a legitimate interest in generating
  secure passwords locally on your device
- **Consent:** By using Void Vault, you consent to local password generation
- **No special categories:** We do not process special categories of personal data

## Summary

| Question | Answer |
|----------|--------|
| Do you collect data? | No |
| Do you use analytics? | No |
| Do you connect to servers? | No |
| Do you use cookies? | No (browser extension uses localStorage only for setup flag) |
| Do you sell data? | No (we have no data to sell) |
| Do you share data with third parties? | No (we have no data to share) |
| Can you see my passwords? | No (passwords are generated locally and never stored) |
| Can you see which sites I visit? | No (no tracking whatsoever) |
| Is it safe to use? | Yes (open source, auditable, offline) |

---

**TL;DR:** Void Vault is a local-only, offline, zero-data-collection password
manager. Your privacy is absolute because we literally cannot access your data.
