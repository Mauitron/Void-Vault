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
// Alternative commercial licensing: Maui_The_Magnificent@proton.me


const tabStates = new Map();
let hasCheckedSetup = false;

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {

  switch (message.type) {
    case 'ACTIVATE_STARWELL':
      activateStarwell(sender.tab.id, message.domain);
      sendResponse({status: 'activated'});
      break;

    case 'DEACTIVATE_STARWELL':
      deactivateStarwell(sender.tab.id);
      sendResponse({status: 'deactivated'});
      break;

    case 'STARWELL_INPUT':
      handleInput(message.character, sender.tab.id);
      sendResponse({status: 'processing'});
      break;

    case 'STARWELL_RESET':
      handleReset(sender.tab.id);
      sendResponse({status: 'reset'});
      break;

    case 'STARWELL_FINALIZE':
      deactivateStarwell(sender.tab.id);
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

    case 'ACTIVATE_PREVIEW':
      const tabStatePreview = tabStates.get(sender.tab.id);
      if (tabStatePreview && tabStatePreview.nativePort && tabStatePreview.isActive) {
        tabStatePreview.nativePort.postMessage({
          type: 'ACTIVATE_PREVIEW',
          domain: message.domain
        });
        sendResponse({status: 'preview_activated'});
      } else {
        sendResponse({status: 'error', error: 'Not active'});
      }
      break;

    case 'COMMIT_INCREMENT':
      const tabStateCommit = tabStates.get(sender.tab.id);
      if (tabStateCommit && tabStateCommit.nativePort && tabStateCommit.isActive) {
        tabStateCommit.nativePort.postMessage({
          type: 'COMMIT_INCREMENT',
          domain: message.domain
        });
        sendResponse({status: 'committed'});
      } else {
        sendResponse({status: 'error', error: 'Not active'});
      }
      break;

    case 'CANCEL_PREVIEW':
      const tabStateCancel = tabStates.get(sender.tab.id);
      if (tabStateCancel && tabStateCancel.nativePort && tabStateCancel.isActive) {
        tabStateCancel.nativePort.postMessage({
          type: 'CANCEL_PREVIEW'
        });
        sendResponse({status: 'cancelled'});
      } else {
        sendResponse({status: 'error', error: 'Not active'});
      }
      break;

    case 'SET_COUNTER':
      const tabStateSet = tabStates.get(sender.tab.id);
      if (tabStateSet && tabStateSet.nativePort && tabStateSet.isActive) {
        tabStateSet.nativePort.postMessage({
          type: 'SET_COUNTER',
          domain: message.domain,
          counter: message.counter
        });
        sendResponse({status: 'counter_set'});
      } else {
        sendResponse({status: 'error', error: 'Not active'});
      }
      break;

    case 'GET_COUNTER':
      const tabStateGet = tabStates.get(sender.tab.id);
      if (tabStateGet && tabStateGet.nativePort && tabStateGet.isActive) {
        tabStateGet.nativePort.postMessage({
          type: 'GET_COUNTER',
          domain: message.domain
        });
        sendResponse({status: 'requested'});
      } else {
        sendResponse({status: 'error', error: 'Not active'});
      }
      break;

    case 'GET_RULES':
      // GET_RULES queries the binary for current rules for a domain
      try {
        const tempPort = chrome.runtime.connectNative('com.starwell.void_vault');

        let responded = false;
        const timeoutId = setTimeout(() => {
          if (!responded) {
            responded = true;
            tempPort.disconnect();
            sendResponse({error: 'Timeout'});
          }
        }, 5000);

        tempPort.onMessage.addListener((msg) => {
          if (!responded && msg.status === 'ready') {
            responded = true;
            clearTimeout(timeoutId);
            tempPort.disconnect();

            sendResponse({
              maxLength: msg.max_length || 0,
              charTypes: msg.char_types || 127
            });
          }
        });

        tempPort.onDisconnect.addListener(() => {
          if (!responded) {
            responded = true;
            clearTimeout(timeoutId);
            sendResponse({error: 'Connection lost'});
          }
        });

        tempPort.postMessage({
          type: 'ACTIVATE',
          domain: message.domain
        });
      } catch (e) {
        sendResponse({error: e.message});
      }
      break;

    case 'SET_RULES':
      try {
        const tempPort = chrome.runtime.connectNative('com.starwell.void_vault');

        let responded = false;
        const timeoutId = setTimeout(() => {
          if (!responded) {
            responded = true;
            tempPort.disconnect();
            sendResponse({status: 'error', error: 'Timeout waiting for binary response'});
          }
        }, 5000);

        tempPort.onMessage.addListener((msg) => {
          if (!responded && (msg.status === 'success' || msg.error)) {
            responded = true;
            clearTimeout(timeoutId);
            tempPort.disconnect();

            if (msg.error) {
              sendResponse({status: 'error', error: msg.error});
            } else {
              sendResponse({status: 'success'});
            }
          }
        });

        tempPort.onDisconnect.addListener(() => {
          if (!responded) {
            responded = true;
            clearTimeout(timeoutId);
            const error = chrome.runtime.lastError;
            sendResponse({status: 'error', error: error ? error.message : 'Connection lost'});
          }
        });

        tempPort.postMessage({
          type: 'SET_RULES',
          domain: message.domain,
          max_length: message.maxLength || 0,
          char_types: message.charTypes || 127
        });
      } catch (e) {
        sendResponse({status: 'error', error: 'Failed to connect to binary: ' + e.message});
      }
      break;

    case 'OPEN_SETTINGS':
      chrome.action.openPopup();
      sendResponse({status: 'opened'});
      break;
  }
  return true;
});

function activateStarwell(tabId, domain) {
  const existingState = tabStates.get(tabId);
  if (existingState && existingState.nativePort) {
    existingState.nativePort.disconnect();
  }

  try {
    console.log('[Starwell Background] Tab', tabId, 'activating for domain:', domain);
    const nativePort = chrome.runtime.connectNative('com.starwell.void_vault');
    console.log('[Starwell Background] Native port connected for tab:', tabId);

    tabStates.set(tabId, {
      nativePort: nativePort,
      isActive: true,
      domain: domain
    });

    const thisTabId = tabId;

    nativePort.onMessage.addListener((message) => {
      console.log('[Starwell Background] Tab', thisTabId, 'received message from binary:', message);

      if (message.status === 'ready' && message.saved_counter !== undefined) {
        chrome.tabs.sendMessage(thisTabId, {
          type: 'COUNTER_UPDATED',
          savedCounter: message.saved_counter,
          activeCounter: message.active_counter,
          maxLength: message.max_length || 0,
          charTypes: message.char_types || 127,
          isPreviewMode: false
        });
      }

      if (message.status === 'preview' && message.saved_counter !== undefined) {
        chrome.tabs.sendMessage(thisTabId, {
          type: 'COUNTER_UPDATED',
          savedCounter: message.saved_counter,
          activeCounter: message.active_counter,
          maxLength: message.max_length || 0,
          charTypes: message.char_types || 127,
          isPreviewMode: true
        });
      }

      if (message.status === 'committed') {
        chrome.tabs.sendMessage(thisTabId, {
          type: 'COUNTER_COMMITTED',
          counter: message.counter
        });
      }

      if (message.status === 'cancelled') {
        chrome.tabs.sendMessage(thisTabId, {
          type: 'PREVIEW_CANCELLED',
          counter: message.counter
        });
      }

      if (message.status === 'success') {
        chrome.tabs.sendMessage(thisTabId, {
          type: 'COUNTER_SET_SUCCESS'
        });
      }

      if (message.counter !== undefined && message.status !== 'ready' && message.status !== 'preview' && message.status !== 'cancelled' && message.status !== 'committed') {
        chrome.tabs.sendMessage(thisTabId, {
          type: 'COUNTER_RETRIEVED',
          counter: message.counter
        });
      }

      if (message.output) {
        chrome.tabs.sendMessage(thisTabId, {
          type: 'UPDATE_PASSWORD',
          password: message.output,
          normalize: true
        });
      }
    });

    nativePort.onDisconnect.addListener(() => {
      console.log('[Starwell Background] Tab', thisTabId, 'native port disconnected');
      tabStates.delete(thisTabId);
    });

    nativePort.postMessage({
      type: 'ACTIVATE',
      domain: domain
    });

  } catch (error) {
    console.error('[Starwell Background] Tab', tabId, 'failed to connect to native binary:', error);
    tabStates.delete(tabId);
  }
}

function deactivateStarwell(tabId) {
  const tabState = tabStates.get(tabId);
  if (tabState && tabState.nativePort) {
    tabState.nativePort.postMessage({ type: 'FINALIZE' });
    tabState.nativePort.disconnect();
    tabStates.delete(tabId);
    console.log('[Starwell Background] Tab', tabId, 'deactivated');
  }
}

function handleInput(character, tabId) {
  const tabState = tabStates.get(tabId);
  if (!tabState || !tabState.nativePort || !tabState.isActive) {
    console.error('[Starwell Background] Tab', tabId, 'not connected to binary');
    return;
  }

  const charCode = character.charCodeAt(0);
  tabState.nativePort.postMessage({
    charCode: charCode
  });
}

function handleReset(tabId) {
  const tabState = tabStates.get(tabId);
  if (!tabState || !tabState.nativePort || !tabState.isActive) {
    return;
  }

  tabState.nativePort.postMessage({
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
    console.error('[Starwell Background] Error checking geometry:', error);
    invokeCallback({ hasShape: false, error: error.message });
  }
}

let setupCheckInProgress = false;

function checkAndOpenSetup() {
  if (setupCheckInProgress) {
    return;
  }

  setupCheckInProgress = true;

  chrome.storage.local.get(['setupComplete'], (result) => {
    if (result.setupComplete) {
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
