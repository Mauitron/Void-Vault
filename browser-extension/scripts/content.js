// Void Vault - Browser Extension (Content Script)
// Copyright (C) 2025 Starwell Project
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// Alternative commercial licensing: licensing@starwell.se

let starwellActive = false;
let currentPasswordField = null;
let characterCount = 0;
let currentDomain = '';

document.addEventListener('focusin', (event) => {
  // Check if it's a password field (type="password" or type="text" with password-related attributes)
  const isPasswordField = event.target.type === 'password' ||
                          (event.target.type === 'text' &&
                           (event.target.autocomplete === 'current-password' ||
                            event.target.autocomplete === 'new-password' ||
                            event.target.name?.toLowerCase().includes('pass') ||
                            event.target.id?.toLowerCase().includes('pass')));

  if (isPasswordField) {
    currentPasswordField = event.target;

    if (!starwellActive) {
      event.target.style.borderColor = '#4CAF50';
      event.target.style.borderWidth = '2px';
      showStarwellHint(event.target);
    }
  }
});

document.addEventListener('focusout', (event) => {
  // Don't deactivate on focus loss - user might be switching windows
  // Only deactivate on explicit actions (Escape key or Enter key)
  if (event.target === currentPasswordField && !starwellActive) {
    event.target.style.borderColor = '';
    event.target.style.borderWidth = '';
  }
});

function showStarwellHint(field) {
  const hint = document.createElement('div');
  hint.id = 'starwell-hint';
  hint.textContent = 'Press Ctrl+Shift+S to activate Void Vault (Esc to cancel)';
  hint.style.cssText = `
    position: absolute;
    background: #4CAF50;
    color: white;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    z-index: 10000;
    pointer-events: none;
  `;

  const rect = field.getBoundingClientRect();
  hint.style.top = (rect.top + window.scrollY - 30) + 'px';
  hint.style.left = (rect.left + window.scrollX) + 'px';

  document.body.appendChild(hint);

  setTimeout(() => {
    hint.remove();
  }, 3000);
}

// Ctrl+Shift+S to toggle (activate/deactivate)
document.addEventListener('keydown', (event) => {
  if (event.ctrlKey && event.shiftKey && event.key === 'S') {
    event.preventDefault();

    // If Void Vault is already active, deactivate it
    if (starwellActive) {
      deactivateStarwell();
      return;
    }

    // If a password field is already focused, activate on it
    if (currentPasswordField && document.activeElement === currentPasswordField) {
      activateStarwell();
      return;
    }

    // Otherwise, find and focus a password field automatically
    const passwordFields = document.querySelectorAll('input[type="password"], input[autocomplete="current-password"], input[autocomplete="new-password"]');

    if (passwordFields.length > 0) {
      // Focus the first password field found
      const field = passwordFields[0];
      currentPasswordField = field;
      field.focus();

      // Give it a moment to focus, then activate
      setTimeout(() => {
        activateStarwell();
      }, 50);
    } else {
      console.log('[Starwell] No password field found on page');
    }
  }

  // Only handle input if Void Vault is active and the current field is focused
  if (starwellActive && currentPasswordField && document.activeElement === currentPasswordField) {
    handleStarwellInput(event);
  }
});

function activateStarwell() {
  if (!currentPasswordField) return;

  starwellActive = true;
  characterCount = 0;
  currentDomain = window.location.hostname;
  currentPasswordField.value = '';
  currentPasswordField.placeholder = 'Void Vault active, type your phrase...';
  currentPasswordField.style.backgroundColor = '#2c2c2c';
  currentPasswordField.style.color = '#ffffff';
  currentPasswordField.style.borderColor = '#4CAF50';
  currentPasswordField.style.borderWidth = '2px';

  console.log('[Starwell Content] Sending ACTIVATE_STARWELL message');
  chrome.runtime.sendMessage({
    type: 'ACTIVATE_STARWELL',
    domain: currentDomain
  }, (response) => {
    if (chrome.runtime.lastError) {
      console.error('[Starwell Content] Error sending message:', chrome.runtime.lastError);
    } else {
      console.log('[Starwell Content] Message sent successfully, response:', response);
    }
  });

  console.log('[Starwell] Activated for', currentDomain);
}

function deactivateStarwell() {
  starwellActive = false;
  characterCount = 0;

  if (currentPasswordField) {
    currentPasswordField.placeholder = '';
    currentPasswordField.style.backgroundColor = '';
    currentPasswordField.style.color = '';
    currentPasswordField.style.borderColor = '';
    currentPasswordField.style.borderWidth = '';
  }

  chrome.runtime.sendMessage({ type: 'DEACTIVATE_STARWELL' });

  console.log('[Starwell] Deactivated');
}

function applyDomainShift(inputChar, domain, inputPosition) {
  const domainLetters = domain.split('').filter(ch => ch !== '.');
  if (domainLetters.length === 0) return inputChar;

  const domainChar = domainLetters[inputPosition % domainLetters.length];
  const domainByte = domainChar.charCodeAt(0);
  const shiftAmount = domainByte % 26;
  const isEven = (domainByte % 2) === 0;

  let charCode = inputChar.charCodeAt(0);
  charCode = isEven ? charCode + shiftAmount : charCode - shiftAmount;

  return String.fromCharCode(charCode);
}

function handleStarwellInput(event) {
  if (event.key.length === 1 && !event.ctrlKey && !event.altKey) {
    event.preventDefault();

    const shiftedChar = applyDomainShift(event.key, currentDomain, characterCount);
    characterCount++;

    console.log('[Starwell Content] Key pressed, count:', characterCount);

    console.log('[Starwell Content] Sending character to background');
    chrome.runtime.sendMessage({
      type: 'STARWELL_INPUT',
      character: shiftedChar
    });

  } else if (event.key === 'Backspace') {
    event.preventDefault();

    console.log('[Starwell Content] Backspace - resetting everything');

    characterCount = 0;
    currentPasswordField.value = '';

    chrome.runtime.sendMessage({
      type: 'STARWELL_RESET'
    });

  } else if (event.key === 'Enter') {
    event.preventDefault();

    chrome.runtime.sendMessage({
      type: 'STARWELL_FINALIZE'
    });

    deactivateStarwell();
  } else if (event.key === 'Escape') {
    event.preventDefault();

    // Cancel
    currentPasswordField.value = '';
    deactivateStarwell();
  }
}

chrome.runtime.onMessage.addListener(async (message, sender, sendResponse) => {
  console.log('[Starwell Content] Received message type:', message.type);

  if (message.type === 'UPDATE_PASSWORD') {
    if (currentPasswordField && starwellActive) {
      let password = message.password;

      // Stupid NFC normalization for compatibility, killing my dreams
      if (message.normalize) {
        password = password.normalize('NFC');
      }

      // Look up rules for THIS domain from storage
      chrome.storage.local.get(['domainRules'], (result) => {
        const allRules = result.domainRules || {};
        const domainRules = allRules[currentDomain] || null;

        console.log('[Starwell Content] Looking up rules for domain:', currentDomain);
        console.log('[Starwell Content] Rules found:', domainRules ? 'yes' : 'no');

        // Apply domain-specific normalization if rules exist
        if (domainRules && domainRules.enabled) {
          password = normalizePassword(password, domainRules);
          console.log('[Starwell Content] Applied normalization rules');
        }

        console.log('[Starwell Content] Setting password value');
        currentPasswordField.value = password;

        // Visual feedback for minimum length (count actual characters, not code units)
        const passwordLength = Array.from(password).length;

        // Always update border color based on current state
        if (domainRules && domainRules.minLength) {
          if (passwordLength < domainRules.minLength) {
            // Password is under minimum - show red border
            console.log('[Starwell Content] Password too short:', passwordLength, '/', domainRules.minLength);
            currentPasswordField.style.borderColor = '#f44336';
            currentPasswordField.style.borderWidth = '2px';
          } else {
            // Password meets minimum - show green border
            console.log('[Starwell Content] Password meets minimum:', passwordLength, '>=', domainRules.minLength);
            currentPasswordField.style.borderColor = '#4CAF50';
            currentPasswordField.style.borderWidth = '2px';
          }
        } else {
          // No minimum set - show green border
          currentPasswordField.style.borderColor = '#4CAF50';
          currentPasswordField.style.borderWidth = '2px';
        }
      });
    }
  } else if (message.type === 'RULES_UPDATED') {
    // Rules were updated - trigger reset if Void Vault is active
    console.log('[Starwell Content] Rules updated for domain:', message.domain);
    if (message.domain === currentDomain && starwellActive && currentPasswordField) {
      console.log('[Starwell Content] Void Vault active - resetting to apply new rules');
      // Clear the field so user can retype with new rules
      currentPasswordField.value = '';
      // Send reset to background to clear state
      chrome.runtime.sendMessage({ type: 'STARWELL_RESET' });
    }
  }
});

console.log('[Void Vault] Content script loaded');
