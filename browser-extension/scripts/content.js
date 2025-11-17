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
// Alternative commercial licensing: Maui_The_Magnificent@proton.me

let starwellActive = false;
let currentPasswordField = null;
let characterCount = 0;
let currentDomain = '';
let savedCounter = 0;
let activeCounter = 0;
let isPreviewMode = false;

document.addEventListener('focusin', (event) => {
  const isPasswordField = event.target.type === 'password' ||
                          (event.target.type === 'text' &&
                           (event.target.autocomplete === 'current-password' ||
                            event.target.autocomplete === 'new-password' ||
                            event.target.name?.toLowerCase().includes('pass') ||
                            event.target.id?.toLowerCase().includes('pass')));

  if (isPasswordField) {
    if (!starwellActive) {
      currentPasswordField = event.target;
      event.target.style.borderColor = '#4CAF50';
      event.target.style.borderWidth = '2px';
      showStarwellHint(event.target);
    }
  }
});

document.addEventListener('focusout', (event) => {
  if (event.target === currentPasswordField && !starwellActive) {
    event.target.style.borderColor = '';
    event.target.style.borderWidth = '';
  }
});

document.addEventListener('click', (event) => {
  const isPasswordField = event.target.type === 'password' ||
                          (event.target.type === 'text' &&
                           (event.target.autocomplete === 'current-password' ||
                            event.target.autocomplete === 'new-password' ||
                            event.target.name?.toLowerCase().includes('pass') ||
                            event.target.id?.toLowerCase().includes('pass')));

  if (isPasswordField && !starwellActive) {
    currentPasswordField = event.target;
    showActivationPopup(event.target);
  }
});

function showStarwellHint(field) {
  const hint = document.createElement('div');
  hint.id = 'starwell-hint';
  hint.innerHTML = '<span style="color: #c4b5fd;">✧</span> Press Ctrl+Shift+S to activate Void Vault (Esc to cancel)';
  hint.style.cssText = `
    position: absolute;
    background: linear-gradient(135deg, #7c3aed, #6d28d9);
    color: white;
    padding: 0.375rem 0.625rem;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    font-weight: 500;
    z-index: 10000;
    pointer-events: none;
    box-shadow: 0 0 0.75rem rgba(124, 58, 237, 0.4), 0 0.125rem 0.5rem rgba(0,0,0,0.3);
    border: 0.0625rem solid #a78bfa;
    font-family: Arial, Helvetica, sans-serif;
  `;

  const rect = field.getBoundingClientRect();
  hint.style.top = (rect.top + window.scrollY - 32) + 'px';
  hint.style.left = (rect.left + window.scrollX) + 'px';

  document.body.appendChild(hint);

  setTimeout(() => {
    hint.remove();
  }, 3000);
}

function showActivationPopup(field) {
  const existing = document.getElementById('vv-activation-popup');
  if (existing) existing.remove();

  const popup = document.createElement('div');
  popup.id = 'vv-activation-popup';
  popup.style.cssText = `
    position: absolute;
    background: linear-gradient(135deg, #1e1b4b, #1a1a1a);
    border: 0.125rem solid #7c3aed;
    border-radius: 0.5rem;
    padding: 0.75rem;
    z-index: 999999;
    box-shadow: 0 0 1.25rem rgba(124, 58, 237, 0.3), 0 0.25rem 1rem rgba(0,0,0,0.5);
    min-width: 12.5rem;
    overflow: hidden;
  `;

  const rect = field.getBoundingClientRect();
  popup.style.top = (rect.bottom + window.scrollY + 8) + 'px';
  popup.style.left = (rect.left + window.scrollX) + 'px';

  const starBg = document.createElement('div');
  starBg.style.cssText = `
    position: absolute;
    font-size: 3rem;
    color: rgba(124, 58, 237, 0.15);
    top: 50%;
    right: -0.5rem;
    transform: translateY(-50%) rotate(-15deg);
    pointer-events: none;
    z-index: 0;
  `;
  starBg.textContent = '✧';
  popup.appendChild(starBg);

  const content = document.createElement('div');
  content.style.cssText = 'position: relative; z-index: 1;';
  content.innerHTML = `
    <div style="color: white; font-size: 0.8125rem; font-weight: bold; margin-bottom: 0.5rem; font-family: Arial, Helvetica, sans-serif; display: flex; align-items: center; gap: 0.375rem;">
      <span style="color: #a78bfa; font-size: 1rem;"></span> Void Vault
    </div>
    <div style="display: flex; flex-direction: column; gap: 0.375rem;">
      <button id="vv-quick-activate" class="vv-quick-btn" style="padding: 0.5rem 0.75rem; background: #7c3aed; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem; font-weight: 500; transition: none; position: relative;">
        <span style="display: flex; align-items: center; justify-content: center; gap: 0.375rem;">
          ▶ Activate Vault
        </span>
      </button>
      <button id="vv-quick-preview" class="vv-quick-btn" style="padding: 0.5rem 0.75rem; background: #a855f7; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem; font-weight: 500; transition: none; position: relative;">
        <span style="display: flex; align-items: center; justify-content: center; gap: 0.375rem;">
          🔄 Activate Preview
        </span>
      </button>
    </div>
    <div style="color: #999; font-size: 0.625rem; margin-top: 0.5rem; text-align: center; font-family: Arial, Helvetica, sans-serif;">
      Or press Ctrl+Shift+S / Ctrl+Shift+P
    </div>
  `;

  popup.appendChild(content);

  document.body.appendChild(popup);

  // Add tooltips
  const activateBtn = document.getElementById('vv-quick-activate');
  const previewBtn = document.getElementById('vv-quick-preview');

  let tooltip = null;

  const showTooltip = (btn, text) => {
    tooltip = document.createElement('div');
    tooltip.className = 'vv-tooltip';
    tooltip.textContent = text;
    tooltip.style.cssText = `
      position: absolute;
      background: linear-gradient(135deg, #1e1b4b, #1a1a1a);
      color: white;
      padding: 0.375rem 0.625rem;
      border-radius: 0.25rem;
      font-size: 0.6875rem;
      z-index: 10000000;
      pointer-events: none;
      white-space: nowrap;
      box-shadow: 0 0 0.5rem rgba(124, 58, 237, 0.4), 0 0.125rem 0.5rem rgba(0,0,0,0.4);
      border: 0.0625rem solid #7c3aed;
      font-family: Arial, Helvetica, sans-serif;
    `;

    const btnRect = btn.getBoundingClientRect();
    tooltip.style.top = (btnRect.top + window.scrollY - 32) + 'px';
    tooltip.style.left = (btnRect.left + window.scrollX + btnRect.width / 2) + 'px';
    tooltip.style.transform = 'translateX(-50%)';

    document.body.appendChild(tooltip);
  };

  const hideTooltip = () => {
    if (tooltip) {
      tooltip.remove();
      tooltip = null;
    }
  };

  activateBtn.addEventListener('mouseenter', () => {
    activateBtn.style.background = '#45a049';
    activateBtn.style.transform = 'translateY(-1px)';
    showTooltip(activateBtn, 'Use current saved version (Ctrl+Shift+S)');
  });
  activateBtn.addEventListener('mouseleave', () => {
    activateBtn.style.background = '#4CAF50';
    activateBtn.style.transform = 'translateY(0)';
    hideTooltip();
  });

  previewBtn.addEventListener('mouseenter', () => {
    previewBtn.style.background = '#f57c00';
    previewBtn.style.transform = 'translateY(-1px)';
    showTooltip(previewBtn, 'Try new version +1 without saving (Ctrl+Shift+P)');
  });
  previewBtn.addEventListener('mouseleave', () => {
    previewBtn.style.background = '#ff9800';
    previewBtn.style.transform = 'translateY(0)';
    hideTooltip();
  });

  activateBtn.addEventListener('click', (e) => {
    e.stopPropagation(); // Prevent click-outside handler from firing
    popup.remove();
    hideTooltip();
    activateStarwell();
    setTimeout(() => {
      if (currentPasswordField) {
        currentPasswordField.focus();
      }
    }, 50);
  });

  previewBtn.addEventListener('click', (e) => {
    e.stopPropagation(); // Prevent click-outside handler from firing
    popup.remove();
    hideTooltip();
    activateStarwell();
    setTimeout(() => {
      if (starwellActive) {
        chrome.runtime.sendMessage({
          type: 'ACTIVATE_PREVIEW',
          domain: currentDomain
        });
        characterCount = 0;
      }
      if (currentPasswordField) {
        currentPasswordField.focus();
      }
    }, 50);
  });

  setTimeout(() => {
    const closeOnClickOutside = (e) => {
      if (!popup.contains(e.target) && e.target !== field) {
        popup.remove();
        hideTooltip();
        document.removeEventListener('click', closeOnClickOutside);
      }
    };
    document.addEventListener('click', closeOnClickOutside);
  }, 100);
}

// Ctrl+Shift+S to toggle (activate/deactivate)
// Ctrl+Shift+P to activate preview mode
document.addEventListener('keydown', (event) => {
  if (event.ctrlKey && event.shiftKey && event.key === 'S') {
    event.preventDefault();

    if (starwellActive) {
      deactivateStarwell();
      return;
    }

    if (currentPasswordField && document.activeElement === currentPasswordField) {
      activateStarwell();
      return;
    }

    const passwordFields = document.querySelectorAll('input[type="password"], input[autocomplete="current-password"], input[autocomplete="new-password"]');

    if (passwordFields.length > 0) {
      // Focus the first password field found
      const field = passwordFields[0];
      currentPasswordField = field;
      field.focus();

      setTimeout(() => {
        activateStarwell();
      }, 50);
    } else {
      console.log('[Starwell] No password field found on page');
    }
  }

  if (event.ctrlKey && event.shiftKey && event.key === 'P') {
    event.preventDefault();

    if (starwellActive) {
      if (isPreviewMode) {
        chrome.runtime.sendMessage({
          type: 'CANCEL_PREVIEW'
        });
        activeCounter = savedCounter;
        isPreviewMode = false;
        characterCount = 0;
        updateOverlay();
      } else {
        chrome.runtime.sendMessage({
          type: 'ACTIVATE_PREVIEW',
          domain: currentDomain
        });
        characterCount = 0;
      }
    } else {
    }
  }

  // press ESC key to close overlay and deactivate Void Vault
  if (event.key === 'Escape') {
    const overlay = document.getElementById('starwell-counter-overlay');
    if (overlay) {
      event.preventDefault();
      overlay.remove();
    }

    if (starwellActive) {
      event.preventDefault();
      deactivateStarwell();
      console.log('[Starwell] Void Vault deactivated via ESC');
    }
  }

  if (starwellActive && currentPasswordField && document.activeElement === currentPasswordField) {
    handleStarwellInput(event);
  }
});

function analyzePasswordField(field) {
  const autocomplete = field.getAttribute('autocomplete');
  const isNewPassword = autocomplete === 'new-password';

  const id = (field.id || '').toLowerCase();
  const name = (field.name || '').toLowerCase();
  const hasNewInName = id.includes('new') ||
                       id.includes('confirm') ||
                       name.includes('new') ||
                       name.includes('confirm');

  const url = window.location.href.toLowerCase();
  const isPasswordReset = url.includes('reset') ||
                          url.includes('change') ||
                          url.includes('password') && (url.includes('new') || url.includes('create'));

  return {
    isNewPassword: isNewPassword || hasNewInName,
    isPasswordReset: isPasswordReset,
    isLogin: !isNewPassword && !hasNewInName && !isPasswordReset
  };
}

function activateStarwell() {
  if (!currentPasswordField) return;

  const activationPopup = document.getElementById('vv-activation-popup');
  if (activationPopup) {
    activationPopup.remove();
  }

  starwellActive = true;
  characterCount = 0;
  currentDomain = window.location.hostname;
  currentPasswordField.placeholder = 'Void Vault active, type your phrase...';
  currentPasswordField.style.backgroundColor = '#2c2c2c';
  currentPasswordField.style.color = '#ffffff';
  currentPasswordField.style.borderColor = '#4CAF50';
  currentPasswordField.style.borderWidth = '2px';

  currentPasswordField.setAttribute('data-original-autocomplete', currentPasswordField.getAttribute('autocomplete') || '');
  currentPasswordField.setAttribute('autocomplete', 'off');

  const tempOverlay = document.createElement('div');
  tempOverlay.id = 'starwell-counter-overlay';
  tempOverlay.style.cssText = `
    position: fixed;
    top: 10px;
    right: 10px;
    background: #4CAF50;
    color: white;
    padding: 12px 16px;
    border-radius: 8px;
    font-family: monospace;
    font-size: 14px;
    z-index: 999999;
    box-shadow: 0 4px 8px rgba(0,0,0,0.3);
    min-width: 200px;
  `;
  tempOverlay.innerHTML = '<div style="font-weight: bold;">Void Vault</div><div style="font-size: 12px;">Activating...</div>';
  document.body.appendChild(tempOverlay);

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
  isPreviewMode = false;
  removeOverlay();

  if (currentPasswordField) {
    currentPasswordField.placeholder = '';
    currentPasswordField.style.backgroundColor = '';
    currentPasswordField.style.color = '';
    currentPasswordField.style.borderColor = '';
    currentPasswordField.style.borderWidth = '';

    const originalAutocomplete = currentPasswordField.getAttribute('data-original-autocomplete');
    if (originalAutocomplete !== null) {
      if (originalAutocomplete === '') {
        currentPasswordField.removeAttribute('autocomplete');
      } else {
        currentPasswordField.setAttribute('autocomplete', originalAutocomplete);
      }
      currentPasswordField.removeAttribute('data-original-autocomplete');
    }
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

    chrome.runtime.sendMessage({
      type: 'STARWELL_INPUT',
      character: shiftedChar
    });

  } else if (event.key === 'Backspace') {
    event.preventDefault();

    characterCount = 0;

    chrome.runtime.sendMessage({
      type: 'STARWELL_RESET'
    });

  } else if (event.key === 'Enter') {
    event.preventDefault();

    if (isPreviewMode && activeCounter !== savedCounter) {
      (async () => {
        const confirmed = await showConfirmDialog(activeCounter);

        if (confirmed) {
          chrome.runtime.sendMessage({
            type: 'COMMIT_INCREMENT',
            domain: currentDomain
          });
          savedCounter = activeCounter;

          chrome.runtime.sendMessage({
            type: 'STARWELL_FINALIZE'
          });
          deactivateStarwell();
          showNotification(`Password updated to v${activeCounter}`);
        } else {
          chrome.runtime.sendMessage({
            type: 'CANCEL_PREVIEW'
          });
          characterCount = 0;
        }
      })();
    } else {
      chrome.runtime.sendMessage({
        type: 'STARWELL_FINALIZE'
      });
      deactivateStarwell();
    }
  } else if (event.key === 'Escape') {
    event.preventDefault();

    if (isPreviewMode) {
      chrome.runtime.sendMessage({
        type: 'CANCEL_PREVIEW'
      });
    }

    deactivateStarwell();
  }
}

function createOverlay() {
  const existing = document.getElementById('starwell-counter-overlay');
  if (existing) existing.remove();

  const overlay = document.createElement('div');
  overlay.id = 'starwell-counter-overlay';
  overlay.className = 'vv-overlay';
  overlay.style.cssText = `
    position: fixed;
    top: 0.625rem;
    right: 0.625rem;
    background: linear-gradient(135deg, ${isPreviewMode ? '#a855f7' : '#1e1b4b'}, ${isPreviewMode ? '#7e22ce' : '#1a1a1a'});
    color: white;
    padding: 1rem;
    border-radius: 0.75rem;
    font-family: Arial, Helvetica, sans-serif;
    font-size: 0.875rem;
    z-index: 999999;
    box-shadow: ${isPreviewMode ? '0 0 1.25rem rgba(168, 85, 247, 0.4), 0 0.5rem 1.5rem rgba(0,0,0,0.6)' : '0 0 1.25rem rgba(124, 58, 237, 0.3), 0 0.5rem 1.5rem rgba(0,0,0,0.6)'};
    min-width: 16.25rem;
    border: 0.125rem solid ${isPreviewMode ? '#c084fc' : '#7c3aed'};
    animation: fadeIn 0.2s ease-out;
    overflow: hidden;
  `;

  const starBg = document.createElement('div');
  starBg.style.cssText = `
    content: '✧';
    position: absolute;
    font-size: 4rem;
    color: ${isPreviewMode ? 'rgba(192, 132, 252, 0.28)' : 'rgba(124, 58, 237, 0.28)'};
    top: 43%;
    left: 80%;
    transform: translate(-50%, -50%);
    pointer-events: none;
    z-index: 0;
  `;
  starBg.textContent = '✧';
  overlay.appendChild(starBg);

  const header = document.createElement('div');
  header.className = 'vv-header';
  header.style.cssText = 'font-weight: bold; margin-bottom: 0.625rem; font-size: 1rem; display: flex; align-items: center; position: relative; z-index: 1;';
  header.innerHTML = `<span style="color: #a78bfa;"></span><span>Void Vault</span><span style="font-size: 0.75rem; opacity: 0.7;">:  ${currentDomain}</span>`;

  const body = document.createElement('div');
  body.className = 'vv-body';
  body.style.cssText = 'margin-bottom: 0.75rem; position: relative; z-index: 1;';

  const isFirstTime = (savedCounter === 0 && activeCounter === 0 && !isPreviewMode);

  if (isFirstTime) {
    body.innerHTML = `
      <div style="color: #a78bfa; font-weight: bold; margin-bottom: 0.25rem;">First time here</div>
      <div style="font-size: 0.75rem; opacity: 0.9;">Creating v0</div>
    `;
  } else if (isPreviewMode) {
    body.innerHTML = `
      <div style="font-size: 0.75rem; opacity: 0.8;">Saved: v${savedCounter}</div>
      <div style="color: #e9d5ff; font-weight: bold; font-size: 0.875rem; margin: 0.25rem 0;">Preview: v${activeCounter}</div>
      <div style="font-size: 0.6875rem; color: #e9d5ff; margin-top: 0.375rem;">⚠️ Not saved yet</div>
      <div style="font-size: 0.6875rem; opacity: 0.8; margin-top: 0.25rem;">Press Enter to confirm or Esc to cancel</div>
    `;
  } else {
    body.innerHTML = `<div style="font-size: 0.875rem;">Version: <strong style="color: #a78bfa;">v${activeCounter}</strong></div>`;
  }

  const footer = document.createElement('div');
  footer.className = 'vv-footer';
  footer.style.cssText = 'display: flex; gap: 0.5rem; margin-top: 0.75rem; position: relative; z-index: 1;';

  if (isPreviewMode) {
    footer.innerHTML = `
      <button id="vv-cancel" style="flex: 1; padding: 0.375rem 0.75rem; background: #555; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;">Cancel</button>
      <button id="vv-history" style="flex: 1; padding: 0.375rem 0.75rem; background: #7e22ce; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;">History</button>
      <button id="vv-settings" style="flex: 1; padding: 0.375rem 0.75rem; background: #7e22ce; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;">Settings</button>
    `;
  } else {
    footer.innerHTML = `
      <button id="vv-new" style="flex: 1; padding: 0.375rem 0.75rem; background: #a855f7; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;">Preview</button>
      <button id="vv-history" style="flex: 1; padding: 0.375rem 0.75rem; background: #6d28d9; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;">History</button>
      <button id="vv-settings" style="flex: 1; padding: 0.375rem 0.75rem; background: #6d28d9; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.75rem;">Settings</button>
    `;
  }

  overlay.appendChild(header);
  overlay.appendChild(body);
  overlay.appendChild(footer);
  document.body.appendChild(overlay);

  attachOverlayListeners();

  setTimeout(() => {
    const closeOnClickOutside = (e) => {
      if (!overlay.contains(e.target) && (!currentPasswordField || !currentPasswordField.contains(e.target))) {
        deactivateStarwell();
        document.removeEventListener('click', closeOnClickOutside);
      }
    };
    document.addEventListener('click', closeOnClickOutside);
  }, 100);

  return overlay;
}

function attachOverlayListeners() {
  const cancelBtn = document.getElementById('vv-cancel');
  if (cancelBtn) {
    cancelBtn.addEventListener('click', (e) => {
      e.stopPropagation(); // Prevent click-outside handler from firing
      if (isPreviewMode) {
        chrome.runtime.sendMessage({
          type: 'CANCEL_PREVIEW'
        });
        activeCounter = savedCounter;
        isPreviewMode = false;
        characterCount = 0;
        updateOverlay();
      } else {
        deactivateStarwell();
      }
    });
  }

  const newBtn = document.getElementById('vv-new');
  if (newBtn) {
    newBtn.addEventListener('click', (e) => {
      e.stopPropagation(); // Prevent click-outside handler from firing
      chrome.runtime.sendMessage({
        type: 'ACTIVATE_PREVIEW',
        domain: currentDomain
      });
      characterCount = 0;
    });
  }

  const historyBtn = document.getElementById('vv-history');
  if (historyBtn) {
    historyBtn.addEventListener('click', (e) => {
      e.stopPropagation(); // Prevent click-outside handler from firing
      showHistory();
    });
  }

  const settingsBtn = document.getElementById('vv-settings');
  if (settingsBtn) {
    settingsBtn.addEventListener('click', (e) => {
      e.stopPropagation(); // Prevent click-outside handler from firing
      chrome.runtime.sendMessage({
        type: 'OPEN_SETTINGS'
      });
    });
  }
}

function updateOverlay() {
  if (starwellActive) {
    createOverlay();
  }
}

// Show history/version selector UI
function showHistory() {
  const existing = document.getElementById('starwell-counter-overlay');
  if (existing) existing.remove();

  const overlay = document.createElement('div');
  overlay.id = 'starwell-counter-overlay';
  overlay.className = 'vv-overlay';
  overlay.style.cssText = `
    position: fixed;
    top: 0.625rem;
    right: 0.625rem;
    background: linear-gradient(135deg, #1e1b4b, #1a1a1a);
    color: white;
    padding: 1rem;
    border-radius: 0.75rem;
    font-family: Arial, Helvetica, sans-serif;
    font-size: 0.875rem;
    z-index: 999999;
    box-shadow: 0 0 1.25rem rgba(124, 58, 237, 0.3), 0 0.5rem 1.5rem rgba(0,0,0,0.6);
    min-width: 20rem;
    max-width: 25rem;
    border: 0.125rem solid #7c3aed;
    animation: fadeIn 0.2s ease-out;
    overflow: hidden;
  `;

  const starBg = document.createElement('div');
  starBg.style.cssText = `
    position: absolute;
    font-size: 6rem;
    color: rgba(124, 58, 237, 0.12);
    bottom: -1rem;
    right: -1rem;
    transform: rotate(-25deg);
    pointer-events: none;
    z-index: 0;
  `;
  starBg.textContent = '✧';
  overlay.appendChild(starBg);

  const header = document.createElement('div');
  header.style.cssText = 'font-weight: bold; margin-bottom: 0.75rem; font-size: 1rem; position: relative; z-index: 1; display: flex; align-items: center; gap: 0.375rem;';
  header.innerHTML = `<span style="color: #a78bfa;">✧</span> Void Vault - ${currentDomain}`;

  const maxVersion = Math.max(savedCounter, activeCounter, 5); 

  const versionsTitle = document.createElement('div');
  versionsTitle.style.cssText = 'font-weight: bold; margin-bottom: 0.5rem; font-size: 0.8125rem; position: relative; z-index: 1;';
  versionsTitle.textContent = 'Password Versions:';

  const versionsList = document.createElement('div');
  versionsList.style.cssText = 'max-height: 12.5rem; overflow-y: auto; margin-bottom: 0.75rem; padding: 0.5rem; background: rgba(0,0,0,0.2); border-radius: 0.375rem; position: relative; z-index: 1;';

  versionsList.addEventListener('click', (e) => {
    e.stopPropagation();
  });

  for (let i = maxVersion; i >= 0; i--) {
    const isSaved = (i === savedCounter);
    const isActive = (i === activeCounter);

    let label = `v${i}`;
    if (i === 0) label += ' [original]';
    if (isSaved) label += ' (saved)';
    if (isActive && isPreviewMode) label += ' (preview)';

    const versionItem = document.createElement('label');

    const updateItemStyle = (isHovered, isChecked) => {
      let bg = 'rgba(255,255,255,0.05)';
      let border = '0.0625rem solid transparent';
      let transform = 'translateX(0)';

      if (isChecked) {
        bg = 'rgba(124, 58, 237, 0.2)';
        border = '0.0625rem solid rgba(124, 58, 237, 0.5)';
      }

      if (isHovered) {
        bg = isChecked ? 'rgba(124, 58, 237, 0.3)' : 'rgba(255,255,255,0.15)';
        transform = 'translateX(0.25rem)';
      }

      versionItem.style.background = bg;
      versionItem.style.border = border;
      versionItem.style.transform = transform;
    };

    versionItem.style.cssText = 'display: block; padding: 0.5rem 0.625rem; margin: 0.1875rem 0; cursor: pointer; border-radius: 0.375rem; transition: all 0.2s ease; user-select: none;';
    updateItemStyle(false, isActive);

    versionItem.innerHTML = `
      <input type="radio" name="version" value="${i}" ${isActive ? 'checked' : ''} style="margin-right: 0.625rem; cursor: pointer; pointer-events: auto; accent-color: #7c3aed;">
      <span style="${isSaved ? 'font-weight: bold; color: #a78bfa;' : ''}">${label}</span>
    `;

    versionItem.addEventListener('click', (e) => {
      e.stopPropagation();
    });

    versionItem.addEventListener('mouseenter', () => {
      const radio = versionItem.querySelector('input[type="radio"]');
      updateItemStyle(true, radio.checked);
    });

    versionItem.addEventListener('mouseleave', () => {
      const radio = versionItem.querySelector('input[type="radio"]');
      updateItemStyle(false, radio.checked);
    });

    const radio = versionItem.querySelector('input[type="radio"]');
    radio.addEventListener('change', (e) => {
      e.stopPropagation(); // Prevent click from closing overlay
      const allItems = versionsList.querySelectorAll('label');
      allItems.forEach(item => {
        const itemRadio = item.querySelector('input[type="radio"]');
        const isHovered = item.matches(':hover');
        if (itemRadio.checked) {
          item.style.background = isHovered ? 'rgba(76, 175, 80, 0.3)' : 'rgba(76, 175, 80, 0.2)';
          item.style.border = '1px solid rgba(76, 175, 80, 0.5)';
        } else {
          item.style.background = isHovered ? 'rgba(255,255,255,0.15)' : 'rgba(255,255,255,0.05)';
          item.style.border = '1px solid transparent';
        }
      });
    });

    versionsList.appendChild(versionItem);
  }

  const manualDiv = document.createElement('div');
  manualDiv.style.cssText = 'margin: 0.75rem 0; padding: 0.5rem; background: rgba(0,0,0,0.2); border-radius: 0.375rem; position: relative; z-index: 1;';
  manualDiv.innerHTML = `
    <div style="font-size: 0.75rem; margin-bottom: 0.375rem; opacity: 0.8;">Or enter manually (0-65535):</div>
    <input type="number" id="vv-manual-input" min="0" max="65535" placeholder="Version number"
      style="width: 100%; padding: 0.375rem; background: rgba(255,255,255,0.1); border: 0.0625rem solid rgba(255,255,255,0.3); border-radius: 0.25rem; color: white; font-size: 0.8125rem; cursor: text; pointer-events: auto;">
  `;

  manualDiv.addEventListener('click', (e) => {
    e.stopPropagation();
  });

  const footer = document.createElement('div');
  footer.style.cssText = 'display: flex; gap: 0.5rem; margin-top: 0.75rem; position: relative; z-index: 1;';
  footer.innerHTML = `
    <button id="vv-close-history" style="flex: 1; padding: 0.5rem; background: #555; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.8125rem;">Close</button>
    <button id="vv-use-temp" style="flex: 1; padding: 0.5rem; background: #8b5cf6; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.8125rem;">Use (Temporary)</button>
    <button id="vv-set-default" style="flex: 1; padding: 0.5rem; background: #7c3aed; color: white; border: none; border-radius: 0.25rem; cursor: pointer; font-size: 0.8125rem;">Set as Default</button>
  `;

  overlay.appendChild(header);
  overlay.appendChild(versionsTitle);
  overlay.appendChild(versionsList);
  overlay.appendChild(manualDiv);
  overlay.appendChild(footer);
  document.body.appendChild(overlay);

  attachHistoryListeners();

  setTimeout(() => {
    const closeOnClickOutside = (e) => {
      if (!overlay.contains(e.target) && (!currentPasswordField || !currentPasswordField.contains(e.target))) {
        deactivateStarwell();
        document.removeEventListener('click', closeOnClickOutside);
      }
    };
    document.addEventListener('click', closeOnClickOutside);
  }, 100);
}

function attachHistoryListeners() {
  document.getElementById('vv-close-history')?.addEventListener('click', (e) => {
    e.stopPropagation(); // Prevent click-outside handler from firing
    updateOverlay();
  });

  document.getElementById('vv-use-temp')?.addEventListener('click', (e) => {
    e.stopPropagation(); // Prevent click-outside handler from firing
    const selected = getSelectedVersion();
    if (selected !== null && selected !== activeCounter) {
      activeCounter = selected;
      isPreviewMode = (selected !== savedCounter);

      chrome.runtime.sendMessage({
        type: 'STARWELL_RESET'
      });
      characterCount = 0;
    }
    updateOverlay();
    setTimeout(() => {
      if (currentPasswordField) {
        currentPasswordField.focus();
      }
    }, 50);
  });

  document.getElementById('vv-set-default')?.addEventListener('click', async (e) => {
    e.stopPropagation(); // Prevent click-outside handler from firing
    const selected = getSelectedVersion();
    if (selected !== null && selected !== savedCounter) {
      const confirmed = await showConfirmDialog(selected, true);
      if (confirmed) {
        chrome.runtime.sendMessage({
          type: 'SET_COUNTER',
          domain: currentDomain,
          counter: selected
        });
        savedCounter = selected;
        activeCounter = selected;
        isPreviewMode = false;

        chrome.runtime.sendMessage({
          type: 'STARWELL_RESET'
        });
        characterCount = 0;
      }
    }
    updateOverlay();
    setTimeout(() => {
      if (currentPasswordField) {
        currentPasswordField.focus();
      }
    }, 50);
  });
}

function getSelectedVersion() {
  const manual = document.getElementById('vv-manual-input');
  if (manual && manual.value && manual.value.trim() !== '') {
    const value = parseInt(manual.value);
    if (!isNaN(value) && value >= 0 && value <= 65535) {
      return value;
    }
  }

  const radio = document.querySelector('input[name="version"]:checked');
  if (radio) {
    return parseInt(radio.value);
  }

  return activeCounter;
}

async function showConfirmDialog(newCounter, isSetDefault = false) {
  return new Promise((resolve) => {
    const dialog = document.createElement('div');
    dialog.id = 'vv-confirm-dialog';
    dialog.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.75);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 10000000;
      animation: fadeIn 0.2s ease-out;
    `;

    const content = document.createElement('div');
    content.style.cssText = `
      background: linear-gradient(135deg, #1e1b4b, #1a1a1a);
      border: 0.125rem solid #7c3aed;
      border-radius: 0.75rem;
      padding: 1.5rem;
      max-width: 28.125rem;
      color: white;
      font-family: Arial, Helvetica, sans-serif;
      box-shadow: 0 0.75rem 2rem rgba(0,0,0,0.8);
      position: relative;
      overflow: hidden;
    `;

    const starBg = document.createElement('div');
    starBg.style.cssText = `
      position: absolute;
      font-size: 8rem;
      color: rgba(124, 58, 237, 0.08);
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      pointer-events: none;
      z-index: 0;
    `;
    starBg.textContent = '✧';
    content.appendChild(starBg);

    const header = document.createElement('div');
    header.style.cssText = 'font-size: 1.125rem; font-weight: bold; margin-bottom: 1rem; color: #a78bfa; position: relative; z-index: 1; display: flex; align-items: center; gap: 0.375rem;';
    header.innerHTML = '<span style="color: #a78bfa;">✧</span> Confirm Password Change';

    const body = document.createElement('div');
    body.style.cssText = 'margin-bottom: 1.25rem; line-height: 1.5; position: relative; z-index: 1;';

    if (isSetDefault) {
      body.innerHTML = `
        <p style="margin: 0.5rem 0;">Set this as the default password for <strong>${currentDomain}</strong>?</p>
        <p style="margin: 0.5rem 0; color: #a78bfa; font-weight: bold;">Counter will change: v${savedCounter} → v${newCounter}</p>
        <p style="margin: 0.75rem 0; padding: 0.625rem; background: rgba(255, 152, 0, 0.2); border-left: 0.1875rem solid #ff9800; border-radius: 0.25rem; font-size: 0.8125rem;">
          ⚠️ <strong>Important:</strong> Remember to update your password on the website!
        </p>
      `;
    } else {
      body.innerHTML = `
        <p style="margin: 0.5rem 0;">Save this as the new password for <strong>${currentDomain}</strong>?</p>
        <p style="margin: 0.5rem 0; color: #a78bfa; font-weight: bold;">Counter will change: v${savedCounter} → v${newCounter}</p>
        <p style="margin: 0.75rem 0; padding: 0.625rem; background: rgba(255, 152, 0, 0.2); border-left: 0.1875rem solid #ff9800; border-radius: 0.25rem; font-size: 0.8125rem;">
          ⚠️ <strong>Important:</strong> Remember to update your password on the website!
        </p>
      `;
    }

    const footer = document.createElement('div');
    footer.style.cssText = 'display: flex; gap: 0.75rem; justify-content: flex-end; position: relative; z-index: 1;';
    footer.innerHTML = `
      <button id="vv-confirm-cancel" style="padding: 0.625rem 1.25rem; background: #555; color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem; font-weight: 500;">Cancel</button>
      <button id="vv-confirm-yes" style="padding: 0.625rem 1.25rem; background: #7c3aed; color: white; border: none; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem; font-weight: 500;">Confirm</button>
    `;

    content.appendChild(header);
    content.appendChild(body);
    content.appendChild(footer);
    dialog.appendChild(content);
    document.body.appendChild(dialog);

    document.getElementById('vv-confirm-cancel').addEventListener('click', () => {
      dialog.remove();
      resolve(false);
    });

    document.getElementById('vv-confirm-yes').addEventListener('click', () => {
      dialog.remove();
      resolve(true);
    });

    dialog.addEventListener('click', (e) => {
      if (e.target === dialog) {
        dialog.remove();
        resolve(false);
      }
    });

    const escHandler = (e) => {
      if (e.key === 'Escape') {
        dialog.remove();
        document.removeEventListener('keydown', escHandler);
        resolve(false);
      }
    };
    document.addEventListener('keydown', escHandler);
  });
}

function removeOverlay() {
  const overlay = document.getElementById('starwell-counter-overlay');
  if (overlay) overlay.remove();
}

function showNotification(message) {
  const notification = document.createElement('div');
  notification.style.cssText = `
    position: fixed;
    top: 1.25rem;
    right: 1.25rem;
    background: linear-gradient(135deg, #7c3aed, #6d28d9);
    color: white;
    padding: 1rem 1.5rem;
    border-radius: 0.5rem;
    font-family: Arial, Helvetica, sans-serif;
    font-size: 0.875rem;
    z-index: 10000001;
    box-shadow: 0 0 1rem rgba(124, 58, 237, 0.4), 0 0.5rem 1.5rem rgba(0,0,0,0.4);
    animation: slideInRight 0.3s ease-out;
    border: 0.0625rem solid #a78bfa;
  `;
  notification.textContent = message;
  document.body.appendChild(notification);

  setTimeout(() => {
    notification.style.animation = 'fadeOut 0.3s ease-out';
    setTimeout(() => notification.remove(), 300);
  }, 3000);
}

chrome.runtime.onMessage.addListener(async (message, sender, sendResponse) => {

  if (message.type === 'COUNTER_UPDATED') {
    savedCounter = message.savedCounter;
    activeCounter = message.activeCounter;
    isPreviewMode = message.isPreviewMode;

    if (currentPasswordField && !isPreviewMode) {
      const fieldContext = analyzePasswordField(currentPasswordField);
      if (fieldContext.isNewPassword || fieldContext.isPasswordReset) {
        chrome.runtime.sendMessage({
          type: 'ACTIVATE_PREVIEW',
          domain: currentDomain
        });
        characterCount = 0;
        return;
      }
    }

    updateOverlay();
  } else if (message.type === 'COUNTER_COMMITTED') {
    savedCounter = message.counter;
    activeCounter = message.counter;
    isPreviewMode = false;
    updateOverlay();
  } else if (message.type === 'PREVIEW_CANCELLED') {
    activeCounter = message.counter;
    isPreviewMode = false;

    chrome.runtime.sendMessage({
      type: 'STARWELL_RESET'
    });

    updateOverlay();
  } else if (message.type === 'COUNTER_SET_SUCCESS') {
    showNotification(`Counter saved to v${activeCounter} for ${currentDomain}`);
    updateOverlay();
  } else if (message.type === 'UPDATE_PASSWORD') {
    if (currentPasswordField) {  // Removed starwellActive check - allow updates even when deactivated
      let password = message.password;

      // Stupid NFC normalization for compatibility, killing my dreams
      if (message.normalize) {
        password = password.normalize('NFC');
      }

      // Look up rules for THIS domain from storage
      chrome.storage.local.get(['domainRules'], (result) => {
        const allRules = result.domainRules || {};
        const domainRules = allRules[currentDomain] || null;

        if (domainRules && domainRules.enabled) {
          password = normalizePassword(password, domainRules);
        }

        // Simulate paste operation to bypass website JS interference
        // This makes frameworks like React/Angular/Vue recognize the input
        // without triggering individual keypress handlers (avoids loops)

        currentPasswordField.value = password;

        const inputEvent = new InputEvent('input', {
          bubbles: true,
          cancelable: true,
          inputType: 'insertText'
        });
        currentPasswordField.dispatchEvent(inputEvent);

        const changeEvent = new Event('change', { bubbles: true });
        currentPasswordField.dispatchEvent(changeEvent);

        const passwordLength = Array.from(password).length;

        if (domainRules && domainRules.minLength) {
          if (passwordLength < domainRules.minLength) {
            currentPasswordField.style.borderColor = '#f44336';
            currentPasswordField.style.borderWidth = '2px';
          } else {
            currentPasswordField.style.borderColor = '#4CAF50';
            currentPasswordField.style.borderWidth = '2px';
          }
        } else {
          currentPasswordField.style.borderColor = '#4CAF50';
          currentPasswordField.style.borderWidth = '2px';
        }
      });
    }
  } else if (message.type === 'RULES_UPDATED') {
    if (message.domain === currentDomain && starwellActive && currentPasswordField) {
      // Clear the field so user can retype with new rules
      // Send reset to background to clear state
      chrome.runtime.sendMessage({ type: 'STARWELL_RESET' });
    }
  }
});

