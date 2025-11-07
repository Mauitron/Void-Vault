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
  if (event.target.type === 'password') {
    currentPasswordField = event.target;

    event.target.style.borderColor = '#4CAF50';
    event.target.style.borderWidth = '2px';

    showStarwellHint(event.target);
  }
});

document.addEventListener('focusout', (event) => {
  if (event.target.type === 'password') {
    event.target.style.borderColor = '';
    event.target.style.borderWidth = '';

    if (starwellActive) {
      deactivateStarwell();
    }
  }
});

function showStarwellHint(field) {
  const hint = document.createElement('div');
  hint.id = 'starwell-hint';
  hint.textContent = 'Press Ctrl+Shift+S to activate Void Vault';
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
    if (currentPasswordField) {
      event.preventDefault();
      if (starwellActive) {
        deactivateStarwell();
      } else {
        activateStarwell();
      }
    }
  }

  if (starwellActive && currentPasswordField === document.activeElement) {
    handleStarwellInput(event);
  }
});

function activateStarwell() {
  starwellActive = true;
  characterCount = 0;
  currentDomain = window.location.hostname;
  currentPasswordField.value = '';
  currentPasswordField.placeholder = 'Void Vault active, type your phrase...';
  currentPasswordField.style.backgroundColor = '#2c2c2c';
  currentPasswordField.style.color = '#ffffff';

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
  currentPasswordField.placeholder = '';
  currentPasswordField.style.backgroundColor = '';
  currentPasswordField.style.color = '';

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

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log('[Starwell Content] Received message type:', message.type);

  if (message.type === 'UPDATE_PASSWORD') {
    if (currentPasswordField && starwellActive) {
      let password = message.password;

      // Stupid NFC normalization for compatibility, killing my dreams
      if (message.normalize) {
        password = password.normalize('NFC');
      }

      console.log('[Starwell Content] Setting password value');
      currentPasswordField.value = password;
    }
  }

  if (message.type === 'APPEND_PASSWORD_CHARS') {
    console.log('[Starwell Content] Appending password characters');
    if (currentPasswordField && starwellActive) {
      let chars = message.characters;

      if (message.normalize) {
        chars = chars.normalize('NFC');
      }

      currentPasswordField.value += chars;
    } else {
      console.log('[Starwell Content] Cannot append field:', currentPasswordField, 'active:', starwellActive);
    }
  }
});

console.log('[Void Vault] Content script loaded');
