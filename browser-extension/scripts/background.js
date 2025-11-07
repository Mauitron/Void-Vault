// Void Vault - Browser Extension (Background Service Worker)
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

let nativePort = null;
let currentTabId = null;
let isStarwellActive = false;
let hasCheckedSetup = false;

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log('[Starwell Background] Message received:', message.type); 

  switch (message.type) {
    case 'ACTIVATE_STARWELL':
      console.log('[Starwell Background] Received ACTIVATE_STARWELL');
      activateStarwell(sender.tab.id, message.domain);
      sendResponse({status: 'activated'});
      break;

    case 'DEACTIVATE_STARWELL':
      deactivateStarwell();
      sendResponse({status: 'deactivated'});
      break;

    case 'STARWELL_INPUT':
      console.log('[Starwell Background] Received STARWELL_INPUT');
      handleInput(message.character, sender.tab.id);
      sendResponse({status: 'processing'});
      break;

    case 'STARWELL_RESET':
      console.log('[Starwell Background] Received STARWELL_RESET');
      handleReset(sender.tab.id);
      sendResponse({status: 'reset'});
      break;

    case 'STARWELL_FINALIZE':
      deactivateStarwell();
      sendResponse({status: 'finalized'});
      break;

    case 'CHECK_SHAPE':
      checkForPasswordShape(sendResponse);
      return true; 

    case 'USE_BINARY':
      // TODO: Implement binary path update
      sendResponse({success: false, error: 'Not implemented yet'});
      break;

    case 'CREATE_BACKUP':
      // TODO: Implement backup creation
      sendResponse({success: false, error: 'Manual backup required'});
      break;
  }
  return true; 
});

function activateStarwell(tabId, domain) {
  currentTabId = tabId;
  isStarwellActive = true;

  console.log('[Starwell Background] Activating for domain:', domain);

  try {
    console.log('[Starwell Background] Attempting to connect to native host...');
    nativePort = chrome.runtime.connectNative('com.starwell.void_vault');
    console.log('[Starwell Background] Native port connected:', nativePort);

    nativePort.onMessage.addListener((message) => {
      console.log('[Starwell Background] Received output from binary');

      if (message.output) {
        chrome.tabs.sendMessage(currentTabId, {
          type: 'APPEND_PASSWORD_CHARS',
          characters: message.output,
          normalize: true // Apply (The peak of stupidity) NFC normalization
        });
      }
    });

    nativePort.onDisconnect.addListener(() => {
      console.log('[Starwell Background] Native port disconnected');
      nativePort = null;
      isStarwellActive = false;
    });

    nativePort.postMessage({
      type: 'INIT'
    });

  } catch (error) {
    console.error('[Starwell Background] Failed to connect to native binary:', error);
    isStarwellActive = false;
  }
}

function deactivateStarwell() {
  if (nativePort) {
    nativePort.postMessage({ type: 'FINALIZE' });
    nativePort.disconnect();
    nativePort = null;
  }

  isStarwellActive = false;
  currentTabId = null;

  console.log('[Starwell Background] Deactivated');
}

function handleInput(character, tabId) {
  if (!nativePort || !isStarwellActive) {
    console.error('[Starwell Background] Not connected to binary - port:', nativePort, 'active:', isStarwellActive);
    return;
  }

  console.log('[Starwell Background] Sending character to binary');
  nativePort.postMessage({
    char: character
  });
}

function handleReset(tabId) {
  if (!nativePort || !isStarwellActive) {
    console.error('[Starwell Background] Cannot handle reset - not connected');
    return;
  }

  console.log('[Starwell Background] Resetting binary state');

  nativePort.postMessage({
    type: 'RESET'
  });

  chrome.tabs.sendMessage(tabId, {
    type: 'UPDATE_PASSWORD',
    password: '',
    normalize: false
  });
}

function checkForPasswordShape(callback) {
  console.log('[Starwell Background] Checking for password shape...');

  let callbackInvoked = false; // Should prevent the double-callback bug [TEMPORARY COMMENT]

  const invokeCallback = (result) => {
    if (!callbackInvoked) {
      callbackInvoked = true;
      callback(result);
    }
  };

  try {
    const testPort = chrome.runtime.connectNative('com.starwell.void_vault');
    let timeoutId = null;

    testPort.onMessage.addListener((message) => {
      console.log('[Starwell Background] Test response status:', message.status);

      if (message.status === 'ready') {
        clearTimeout(timeoutId);
        testPort.disconnect();
        invokeCallback({ hasShape: true });
      }
    });

    testPort.onDisconnect.addListener(() => {
      clearTimeout(timeoutId);
      const error = chrome.runtime.lastError;
      if (error) {
        console.log('[Starwell Background] Binary check failed:', error);
        invokeCallback({ hasShape: false, error: error.message });
      }
    });

    testPort.postMessage({ type: 'INIT' });

    timeoutId = setTimeout(() => {
      testPort.disconnect();
      invokeCallback({ hasShape: false, error: 'Timeout' });
    }, 2000);

  } catch (error) {
    console.error('[Starwell Background] Error checking shape:', error);
    invokeCallback({ hasShape: false, error: error.message });
  }
}

let setupCheckInProgress = false;

function checkAndOpenSetup() {
  if (setupCheckInProgress) {
    console.log('[Starwell Background] Setup check already in progress, skipping...');
    return;
  }

  setupCheckInProgress = true;

  chrome.storage.local.get(['setupComplete'], (result) => {
    if (result.setupComplete) {
      console.log('[Starwell Background] Setup already completed');
      setupCheckInProgress = false;
      return;
    }

    checkForPasswordShape((response) => {
      if (!response.hasShape) {
        console.log('[Starwell Background] No structure found, checking for existing setup tab...');

        chrome.tabs.query({ url: chrome.runtime.getURL('setup.html') }, (tabs) => {
          if (tabs.length > 0) {
            console.log('[Starwell Background] Setup tab already open, focusing...');
            chrome.tabs.update(tabs[0].id, { active: true });
            setupCheckInProgress = false;
          } else {
            console.log('[Starwell Background] Opening new setup tab...');
            chrome.tabs.create({
              url: chrome.runtime.getURL('setup.html')
            }, () => {
              setupCheckInProgress = false;
            });
          }
        });
      } else {
        console.log('[Starwell Background] Structure found, marking setup complete');
        chrome.storage.local.set({ setupComplete: true });
        setupCheckInProgress = false;
      }
    });
  });
}

chrome.runtime.onInstalled.addListener((details) => {
  if (details.reason === 'install') {
    console.log('[Starwell] Extension installed, checking setup...');
    checkAndOpenSetup();
  } else if (details.reason === 'update') {
    console.log('[Starwell] Extension updated');
  }
});

if (!hasCheckedSetup) {
  hasCheckedSetup = true;
  setTimeout(() => {
    checkAndOpenSetup();
  }, 1000); 
}

console.log('[Starwell] Background service worker loaded');
