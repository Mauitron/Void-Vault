// Void Vault - Browser Extension (Domain Shift Module)
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

/**
 * Apply domain-specific shifting to an input character
 * @param {string} inputChar - Single character to shift
 * @param {string} domain - Domain name (e.g., "gmail.com")
 * @param {number} inputPosition - Position of this character in the input sequence
 * @returns {string} - Shifted character
 */
function applyDomainShift(inputChar, domain, inputPosition) {
  // Extract domain letters (skip dots and other punctuation)
  const domainLetters = domain.split('').filter(ch => ch !== '.');

  if (domainLetters.length === 0) {
    return inputChar;
  }

  // Cycle through domain letters based on input position
  const domainChar = domainLetters[inputPosition % domainLetters.length];
  const domainByte = domainChar.charCodeAt(0);

  const shiftAmount = domainByte % 26;

  const isEven = (domainByte % 2) === 0;

  let charCode = inputChar.charCodeAt(0);

  if (isEven) {
    charCode += shiftAmount;
  } else {
    charCode -= shiftAmount;
  }

  return String.fromCharCode(charCode);
}

/**
 * Apply domain shifting to an entire input sequence
 * @param {string} input - Input sequence (e.g., "grandchildren")
 * @param {string} domain - Domain name (e.g., "gmail.com")
 * @returns {string} - Shifted input sequence
 */
function shiftInputForDomain(input, domain) {
  return input.split('').map((char, index) => {
    return applyDomainShift(char, domain, index);
  }).join('');
}

if (typeof module !== 'undefined' && module.exports) {
  module.exports = { applyDomainShift, shiftInputForDomain };
}
