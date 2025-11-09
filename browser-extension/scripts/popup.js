// Void Vault - Browser Extension (Popup Script)
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

let currentDomain = '';

function updateStatus() {
  const statusDiv = document.getElementById('status');

  chrome.runtime.sendMessage({ type: 'GET_STATUS' }, (response) => {
    if (response && response.active) {
      statusDiv.className = 'status active';
      statusDiv.innerHTML = '<strong>Status:</strong> Active';
    } else {
      statusDiv.className = 'status inactive';
      statusDiv.innerHTML = '<strong>Status:</strong> Inactive';
    }
  });
}

updateStatus();
setInterval(updateStatus, 1000);

document.getElementById('settingsButton').addEventListener('click', () => {
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs[0] && tabs[0].url) {
      try {
        const url = new URL(tabs[0].url);
        currentDomain = url.hostname;
        document.getElementById('domainName').textContent = currentDomain;

        loadRules();

        document.getElementById('mainView').style.display = 'none';
        document.getElementById('settingsView').style.display = 'block';
      } catch (e) {
        document.getElementById('domainName').textContent = 'Invalid URL';
        document.getElementById('mainView').style.display = 'none';
        document.getElementById('settingsView').style.display = 'block';
      }
    }
  });
});

document.getElementById('backButton').addEventListener('click', () => {
  document.getElementById('settingsView').style.display = 'none';
  document.getElementById('mainView').style.display = 'block';
});

document.getElementById('enableRules').addEventListener('change', (e) => {
  document.getElementById('rulesForm').style.display = e.target.checked ? 'block' : 'none';
});

document.getElementById('presetAll').addEventListener('click', () => {
  document.getElementById('allowLowercase').checked = true;
  document.getElementById('allowUppercase').checked = true;
  document.getElementById('allowDigits').checked = true;
  document.getElementById('allowBasicSymbols').checked = true;
  document.getElementById('allowExtendedSymbols').checked = true;
  document.getElementById('allowEmojis').checked = true;
  document.getElementById('allowExtendedUnicode').checked = true;
});

document.getElementById('presetBasic').addEventListener('click', () => {
  document.getElementById('allowLowercase').checked = true;
  document.getElementById('allowUppercase').checked = true;
  document.getElementById('allowDigits').checked = true;
  document.getElementById('allowBasicSymbols').checked = true;
  document.getElementById('allowExtendedSymbols').checked = false;
  document.getElementById('allowEmojis').checked = false;
  document.getElementById('allowExtendedUnicode').checked = false;
});

document.getElementById('presetNoSymbols').addEventListener('click', () => {
  document.getElementById('allowLowercase').checked = true;
  document.getElementById('allowUppercase').checked = true;
  document.getElementById('allowDigits').checked = true;
  document.getElementById('allowBasicSymbols').checked = false;
  document.getElementById('allowExtendedSymbols').checked = false;
  document.getElementById('allowEmojis').checked = false;
  document.getElementById('allowExtendedUnicode').checked = false;
});

function loadRules() {
  chrome.storage.local.get(['domainRules'], (result) => {
    const allRules = result.domainRules || {};
    const rules = allRules[currentDomain];

    if (rules && rules.enabled) {
      document.getElementById('enableRules').checked = true;
      document.getElementById('rulesForm').style.display = 'block';

      if (rules.minLength) {
        document.getElementById('minLength').value = rules.minLength;
      }
      if (rules.maxLength) {
        document.getElementById('maxLength').value = rules.maxLength;
      }

      const allowed = rules.allowedChars || ['lowercase', 'uppercase', 'digits', 'basicSymbols', 'extendedSymbols', 'emojis', 'extendedUnicode'];
      document.getElementById('allowLowercase').checked = allowed.includes('lowercase');
      document.getElementById('allowUppercase').checked = allowed.includes('uppercase');
      document.getElementById('allowDigits').checked = allowed.includes('digits');
      document.getElementById('allowBasicSymbols').checked = allowed.includes('basicSymbols');
      document.getElementById('allowExtendedSymbols').checked = allowed.includes('extendedSymbols');
      document.getElementById('allowEmojis').checked = allowed.includes('emojis');
      document.getElementById('allowExtendedUnicode').checked = allowed.includes('extendedUnicode');
    } else {
      // Reset form
      document.getElementById('enableRules').checked = false;
      document.getElementById('rulesForm').style.display = 'none';
      document.getElementById('minLength').value = '';
      document.getElementById('maxLength').value = '';
      document.getElementById('allowLowercase').checked = true;
      document.getElementById('allowUppercase').checked = true;
      document.getElementById('allowDigits').checked = true;
      document.getElementById('allowBasicSymbols').checked = true;
      document.getElementById('allowExtendedSymbols').checked = true;
      document.getElementById('allowEmojis').checked = true;
      document.getElementById('allowExtendedUnicode').checked = true;
    }
  });
}

document.getElementById('saveButton').addEventListener('click', () => {
  const enabled = document.getElementById('enableRules').checked;

  if (!enabled) {
    deleteRules();
    return;
  }

  const minLength = document.getElementById('minLength').value;
  const maxLength = document.getElementById('maxLength').value;

  if (minLength && maxLength && parseInt(minLength) > parseInt(maxLength)) {
    showStatus('Min length cannot be greater than max length', 'error');
    return;
  }

  const allowedChars = [];
  if (document.getElementById('allowLowercase').checked) allowedChars.push('lowercase');
  if (document.getElementById('allowUppercase').checked) allowedChars.push('uppercase');
  if (document.getElementById('allowDigits').checked) allowedChars.push('digits');
  if (document.getElementById('allowBasicSymbols').checked) allowedChars.push('basicSymbols');
  if (document.getElementById('allowExtendedSymbols').checked) allowedChars.push('extendedSymbols');
  if (document.getElementById('allowEmojis').checked) allowedChars.push('emojis');
  if (document.getElementById('allowExtendedUnicode').checked) allowedChars.push('extendedUnicode');

  if (allowedChars.length === 0) {
    showStatus('At least one character type must be allowed', 'error');
    return;
  }

  const rules = {
    enabled: true,
    minLength: minLength ? parseInt(minLength) : null,
    maxLength: maxLength ? parseInt(maxLength) : null,
    allowedChars: allowedChars
  };

  chrome.storage.local.get(['domainRules'], (result) => {
    const allRules = result.domainRules || {};
    allRules[currentDomain] = rules;

    chrome.storage.local.set({ domainRules: allRules }, () => {
      chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
        if (tabs[0]) {
          chrome.tabs.sendMessage(tabs[0].id, {
            type: 'RULES_UPDATED',
            domain: currentDomain,
            rules: rules
          }, () => {
            chrome.runtime.lastError;
          });
        }
      });

      chrome.runtime.sendMessage({
        type: 'RULES_UPDATED',
        domain: currentDomain,
        rules: rules
      }, () => {
        chrome.runtime.lastError;
      });

      showStatus('Rules saved!', 'success');
      setTimeout(() => {
        window.close();
      }, 500);
    });
  });
});

document.getElementById('deleteButton').addEventListener('click', () => {
  if (confirm('Delete password rules for ' + currentDomain + '?')) {
    deleteRules();
  }
});

function deleteRules() {
  chrome.storage.local.get(['domainRules'], (result) => {
    const allRules = result.domainRules || {};
    delete allRules[currentDomain];

    chrome.storage.local.set({ domainRules: allRules }, () => {
      chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
        if (tabs[0]) {
          chrome.tabs.sendMessage(tabs[0].id, {
            type: 'RULES_UPDATED',
            domain: currentDomain,
            rules: null
          }, () => {
            chrome.runtime.lastError;
          });
        }
      });

      chrome.runtime.sendMessage({
        type: 'RULES_UPDATED',
        domain: currentDomain,
        rules: null
      }, () => {
        chrome.runtime.lastError;
      });

      showStatus('Rules deleted!', 'success');
      setTimeout(() => {
        window.close();
      }, 500);
    });
  });
}

function showStatus(message, type) {
  const statusEl = document.getElementById('statusMessage');
  statusEl.textContent = message;
  statusEl.className = 'status-message';

  if (type === 'success') {
    statusEl.style.background = '#E8F5E9';
    statusEl.style.color = '#2E7D32';
  } else {
    statusEl.style.background = '#FFEBEE';
    statusEl.style.color = '#C62828';
  }

  statusEl.style.display = 'block';

  setTimeout(() => {
    statusEl.style.display = 'none';
  }, 3000);
}
