// Void Vault - Browser Extension (Setup Page Script)
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

let uploadedBinaryPath = null;

function goToStep(stepNumber) {
  document.querySelectorAll('.step').forEach(step => {
    step.classList.remove('active');
  });

  document.getElementById('step' + stepNumber).classList.add('active');

  document.querySelectorAll('.status-message').forEach(msg => {
    msg.style.display = 'none';
  });
}

function showStatus(stepId, message, type) {
  const statusEl = document.getElementById(stepId + '-status');
  statusEl.textContent = message;
  statusEl.className = 'status-message ' + type;
  statusEl.style.display = 'block';
}

async function checkBinaryForShape() {
  const statusStep = document.getElementById('step2').classList.contains('active') ? 'step2' : 'step3';
  showStatus(statusStep, 'Checking for configuration...', 'info');

  try {
    chrome.runtime.sendMessage({ type: 'CHECK_SHAPE' }, (response) => {
      if (chrome.runtime.lastError) {
        showStatus(statusStep, 'Error: ' + chrome.runtime.lastError.message, 'error');
        return;
      }

      if (response && response.hasShape) {
        showStatus(statusStep, 'Yay! Configuration found! Setup complete...', 'success');
        setTimeout(() => goToStep(3), 1500);
      } else {
        showStatus(statusStep, 'No configuration found yet. Please endure the setup in your terminal first.', 'error');
      }
    });
  } catch (error) {
    showStatus(statusStep, 'Error checking: ' + error.message, 'error');
  }
}

function handleBinaryUpload(event) {
  const file = event.target.files[0];
  if (!file) return;

  const fileNameEl = document.getElementById('file-name');
  fileNameEl.textContent = 'Selected: ' + file.name;

  uploadedBinaryPath = file.path || file.name;

  document.getElementById('use-binary-btn').classList.remove('hidden');

  showStatus('step4', 'Binary file selected. Click "Use This Binary" to proceed.', 'info');
}

async function useBinary() {
  if (!uploadedBinaryPath) {
    showStatus('step4', 'No binary file selected', 'error');
    return;
  }

  showStatus('step4', 'Setting up Void Vault...', 'info');

  try {
    // Send message to background to use this binary
    chrome.runtime.sendMessage({
      type: 'USE_BINARY',
      binaryPath: uploadedBinaryPath
    }, (response) => {
      if (chrome.runtime.lastError) {
        showStatus('step4', 'Error: ' + chrome.runtime.lastError.message, 'error');
        return;
      }

      if (response && response.success) {
        showStatus('step4', 'Binary loaded successfully!', 'success');
        setTimeout(() => goToStep(5), 1500);
      } else {
        showStatus('step4', 'Failed to load binary: ' + (response?.error || 'Unknown error'), 'error');
      }
    });
  } catch (error) {
    showStatus('step4', 'Error: ' + error.message, 'error');
  }
}

// Create backup of current binary
function createBackup() {
  showStatus('step5', 'Creating backup...', 'info');

  try {
    chrome.runtime.sendMessage({ type: 'CREATE_BACKUP' }, (response) => {
      if (chrome.runtime.lastError) {
        showStatus('step5', 'Error: ' + chrome.runtime.lastError.message, 'error');
        return;
      }

      if (response && response.success) {
        showStatus('step5', 'Backup created! Thank you, now save it to one or more secure locations.', 'success');
      } else {
        showStatus('step5', 'Backup creation not available. Please manually copy your binary file somewhere safe, you will thank me later.', 'info');
      }
    });
  } catch (error) {
    showStatus('step5', 'Manual backup required. Annoying but important! Copy the binary file to a secure location.', 'info');
  }
}

function finishSetup() {
  chrome.storage.local.set({ setupComplete: true }, () => {
    showStatus('step5', 'Setup complete! You did well, you can now close this tab and relax.', 'success');

    setTimeout(() => {
      window.close();
    }, 2000);
  });
}

window.addEventListener('DOMContentLoaded', () => {
  chrome.storage.local.get(['setupComplete'], (result) => {
    if (result.setupComplete) {
      goToStep(5);
    }
  });

  document.getElementById('continue-setup-btn').addEventListener('click', () => goToStep(2));
  document.getElementById('check-setup-btn').addEventListener('click', checkBinaryForShape);
  document.getElementById('copy-command-btn').addEventListener('click', copyCommand);

  const checkShapeBtn = document.getElementById('check-shape-btn');
  if (checkShapeBtn) checkShapeBtn.addEventListener('click', checkBinaryForShape);

  const backBtn3 = document.getElementById('back-from-step3-btn');
  if (backBtn3) backBtn3.addEventListener('click', () => goToStep(2));

  const binaryFile = document.getElementById('binary-file');
  if (binaryFile) binaryFile.addEventListener('change', handleBinaryUpload);

  const useBinaryBtn = document.getElementById('use-binary-btn');
  if (useBinaryBtn) useBinaryBtn.addEventListener('click', useBinary);

  const backBtn4 = document.getElementById('back-from-step4-btn');
  if (backBtn4) backBtn4.addEventListener('click', () => goToStep(2));

  const backupBtn = document.getElementById('backup-btn');
  if (backupBtn) backupBtn.addEventListener('click', createBackup);

  const finishBtn = document.getElementById('finish-setup-btn');
  if (finishBtn) finishBtn.addEventListener('click', finishSetup);
});

function copyCommand() {
  const command = document.getElementById('terminal-command').textContent;
  navigator.clipboard.writeText(command).then(() => {
    const btn = document.getElementById('copy-command-btn');
    const originalText = btn.textContent;
    btn.textContent = 'Copied!';
    setTimeout(() => {
      btn.textContent = originalText;
    }, 2000);
  }).catch(err => {
    showStatus('step2', 'Failed to copy: ' + err.message, 'error');
  });
}
