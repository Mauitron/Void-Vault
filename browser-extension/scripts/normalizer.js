// Void Vault - Password Normalization
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
 * Normalizes passwords to conform with website-specific requirements
 * All normalization is deterministic - same input always produces same output
 */

function simpleHash(str) {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // convert to i32
  }
  return Math.abs(hash);
}

// Character sets for stupid normalization
const CHAR_SETS = {
  lowercase: 'abcdefghijklmnopqrstuvwxyz',
  uppercase: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
  digits: '0123456789',
  basicSymbols: '!@#$%^&*',
  extendedSymbols: '()_+-=[]{}|;:,.<>?~`\'"\\/',
  alphanumeric: 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
};

function isEmoji(char) {
  const code = char.codePointAt(0);
  if (!code) return false;

  return (
    (code >= 0x1F600 && code <= 0x1F64F) || // Emotjis
    (code >= 0x1F300 && code <= 0x1F5FF) || // Misc Symbols and Pictographs
    (code >= 0x1F680 && code <= 0x1F6FF) || // Transport and Map
    (code >= 0x1F900 && code <= 0x1F9FF) || // Supplemental Symbols
    (code >= 0x2600 && code <= 0x26FF) ||   // Misc symbols
    (code >= 0x2700 && code <= 0x27BF) ||   // Dingbats
    (code >= 0x1F000 && code <= 0x1F02F) || // Tiles
    (code >= 0x1F0A0 && code <= 0x1F0FF) || // Playing Cards
    (code >= 0x1FA70 && code <= 0x1FAFF)    // Extended Pictographs
  );
}

function isExtendedUnicode(char) {
  const code = char.codePointAt(0);
  if (!code) return false;
  return code > 127 && !isEmoji(char);
}

/**
 * Normalizes a password according to whatever domain rules are forced upon it
 * @param {string} rawPassword, The raw password from the binary
 * @param {object} rules, Domain-specific rules
 * @returns {string}, Normalized password
 */
function normalizePassword(rawPassword, rules) {
  if (!rules || !rules.enabled) {
    return rawPassword; 
  }

  let normalized = rawPassword;

  const allTypesEnabled = rules.allowedChars &&
    rules.allowedChars.includes('lowercase') &&
    rules.allowedChars.includes('uppercase') &&
    rules.allowedChars.includes('digits') &&
    rules.allowedChars.includes('basicSymbols') &&
    rules.allowedChars.includes('extendedSymbols') &&
    rules.allowedChars.includes('emojis') &&
    rules.allowedChars.includes('extendedUnicode');

  if (!allTypesEnabled && rules.allowedChars && rules.allowedChars.length > 0) {
    const allowedSet = new Set();
    let allowedString = '';

    if (rules.allowedChars.includes('lowercase')) {
      CHAR_SETS.lowercase.split('').forEach(c => allowedSet.add(c));
      allowedString += CHAR_SETS.lowercase;
    }
    if (rules.allowedChars.includes('uppercase')) {
      CHAR_SETS.uppercase.split('').forEach(c => allowedSet.add(c));
      allowedString += CHAR_SETS.uppercase;
    }
    if (rules.allowedChars.includes('digits')) {
      CHAR_SETS.digits.split('').forEach(c => allowedSet.add(c));
      allowedString += CHAR_SETS.digits;
    }
    if (rules.allowedChars.includes('basicSymbols')) {
      CHAR_SETS.basicSymbols.split('').forEach(c => allowedSet.add(c));
      allowedString += CHAR_SETS.basicSymbols;
    }
    if (rules.allowedChars.includes('extendedSymbols')) {
      CHAR_SETS.extendedSymbols.split('').forEach(c => allowedSet.add(c));
      allowedString += CHAR_SETS.extendedSymbols;
    }

    const allowEmojis = rules.allowedChars.includes('emojis');
    const allowExtendedUnicode = rules.allowedChars.includes('extendedUnicode');

    const chars = Array.from(normalized);

    if (allowedString.length === 0) {
      normalized = chars.filter(char => {
        if (isEmoji(char)) return allowEmojis;
        if (isExtendedUnicode(char)) return allowExtendedUnicode;
        return false; // Remove all ASCII characters
      }).join('');
    } else {
      normalized = chars.map(char => {
        if (isEmoji(char)) {
          if (allowEmojis) {
            return char; 
          } else {
            const charCode = char.codePointAt(0);
            const index = charCode % allowedString.length;
            return allowedString[index];
          }
        }

        if (isExtendedUnicode(char)) {
          if (allowExtendedUnicode) {
            return char; 
          } else {
            const charCode = char.codePointAt(0);
            const index = charCode % allowedString.length;
            return allowedString[index];
          }
        }

        if (allowedSet.has(char)) {
          return char; 
        } else {
          const charCode = char.codePointAt(0);
          const index = charCode % allowedString.length;
          return allowedString[index];
        }
      }).join('');
    }
  }

  if (rules.maxLength) {
    const chars = Array.from(normalized);
    if (chars.length > rules.maxLength) {
      normalized = chars.slice(0, rules.maxLength).join('');
    }
  }

  // minLength is only for visual feedback, creating the red border

  return normalized;
}

/**
 * Gets normalization rules for a domain
 * @param {string} domain
 * @returns {Promise<object>} - Rules object or null
 */
async function getRulesForDomain(domain) {
  return new Promise((resolve) => {
    chrome.storage.local.get(['domainRules'], (result) => {
      const allRules = result.domainRules || {};
      resolve(allRules[domain] || null);
    });
  });
}

/**
 * Saves normalization rules for a domain
 * @param {string} domain
 * @param {object} rules
 */
async function saveRulesForDomain(domain, rules) {
  return new Promise((resolve) => {
    chrome.storage.local.get(['domainRules'], (result) => {
      const allRules = result.domainRules || {};
      allRules[domain] = rules;
      chrome.storage.local.set({ domainRules: allRules }, resolve);
    });
  });
}

/**
 * Deletes normalization rules for a domain
 * @param {string} domain
 */
async function deleteRulesForDomain(domain) {
  return new Promise((resolve) => {
    chrome.storage.local.get(['domainRules'], (result) => {
      const allRules = result.domainRules || {};
      delete allRules[domain];
      chrome.storage.local.set({ domainRules: allRules }, resolve);
    });
  });
}

if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    normalizePassword,
    getRulesForDomain,
    saveRulesForDomain,
    deleteRulesForDomain
  };
}
