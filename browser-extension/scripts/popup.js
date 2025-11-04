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
