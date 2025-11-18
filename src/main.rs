// Void Vault - Password Manager
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
// Alternative commercial licensing is available for organizations that
// wish to use this software without the restrictions of the AGPL-3.0.
// Contact: Maui_The_Magnificent@proton.me

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
enum ProcessMessage {
    BinaryUpdated(PathBuf),
    ShutdownChild,
    ChildReady,
    BinaryUpdateComplete,
}

struct BinaryStorageManager {
    executable_path: PathBuf,
    in_memory_cache: HashMap<String, Vec<u8>>,
    metadata_cache: HashMap<String, String>,
    binary_modified: bool,
    parent_mode: bool,
    message_tx: Option<Sender<ProcessMessage>>,
}

impl Clone for BinaryStorageManager {
    fn clone(&self) -> Self {
        BinaryStorageManager {
            executable_path: self.executable_path.clone(),
            in_memory_cache: self.in_memory_cache.clone(),
            metadata_cache: self.metadata_cache.clone(),
            binary_modified: self.binary_modified,
            parent_mode: self.parent_mode,
            message_tx: self.message_tx.clone(),
        }
    }
}

impl BinaryStorageManager {
    // markers are unless reserved, in the zone at the end.
    // they are also based only on executable properties that won't change
    fn generate_markers(&self) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
        // and now we read the binary header to generate portable markers
        let mut header_bytes = Vec::new();
        if let Ok(mut file) = File::open(&self.executable_path) {
            let mut buffer = [0u8; 1024];
            if let Ok(n) = file.read(&mut buffer) {
                header_bytes.extend_from_slice(&buffer[..n]);
            }
        }

        // use only the binary header hash for the marker seed
        let header_seed = {
            let mut header_hasher = std::collections::hash_map::DefaultHasher::new();
            header_bytes.hash(&mut header_hasher);
            header_hasher.finish()
        };

        let mut rng_state = header_seed;

        let generate_marker = |prefix: &[u8], length: usize, rng: &mut u64| -> Vec<u8> {
            let mut marker = Vec::with_capacity(prefix.len() + length + 1);
            marker.extend_from_slice(prefix);

            for _ in 0..length {
                *rng = rng
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                marker.push((*rng % 255) as u8);
            }

            marker.push(0);
            marker
        };

        let section_marker = generate_marker(b"\0SM", 24, &mut rng_state);
        let start_marker = generate_marker(b"\0ST", 16, &mut rng_state);
        let end_marker = generate_marker(b"\0EN", 16, &mut rng_state);
        let name_marker = generate_marker(b"\0NM", 8, &mut rng_state);
        let desc_marker = generate_marker(b"\0DS", 8, &mut rng_state);

        (
            section_marker,
            start_marker,
            end_marker,
            name_marker,
            desc_marker,
        )
    }

    fn new(parent_mode: bool, tx: Option<Sender<ProcessMessage>>) -> io::Result<Self> {
        let executable_path = std::env::current_exe()?;

        let mut manager = BinaryStorageManager {
            executable_path,
            in_memory_cache: HashMap::new(),
            metadata_cache: HashMap::new(),
            binary_modified: false,
            parent_mode,
            message_tx: tx,
        };

        if manager.ensure_end_marker()? {
            manager.binary_modified = true;

            if !parent_mode {
                manager.signal_binary_update()?;
            }
        }

        manager.load_all_passwords()?;

        Ok(manager)
    }

    fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.len() > haystack.len() {
            return None;
        }

        for i in 0..=haystack.len() - needle.len() {
            if haystack[i..i + needle.len()] == needle[..] {
                return Some(i);
            }
        }
        None
    }
    // really important, as without it, the binary would break
    fn ensure_end_marker(&self) -> io::Result<bool> {
        let (section_marker, _, _, _, _) = self.generate_markers();

        let file = match File::open(&self.executable_path) {
            Ok(f) => f,
            Err(e) => {
                println!("WARNING: Could not open executable: {}", e);
                return Err(e);
            }
        };

        let file_size = file.metadata()?.len();

        if file_size < section_marker.len() as u64 {
            return self.append_end_marker();
        }

        let mut file = File::open(&self.executable_path)?;
        let mut end_bytes = vec![0u8; section_marker.len()];
        file.seek(SeekFrom::End(-(section_marker.len() as i64)))?;
        file.read_exact(&mut end_bytes)?;

        if end_bytes == section_marker {
            return Ok(false);
        } else {
            return self.append_end_marker();
        }
    }

    fn append_end_marker(&self) -> io::Result<bool> {
        let (section_marker, _, _, _, _) = self.generate_markers();
        let temp_path = self.executable_path.with_extension("new");

        let mut original = File::open(&self.executable_path)?;
        let mut buffer = Vec::new();
        original.read_to_end(&mut buffer)?;

        let mut new_file = File::create(&temp_path)?;
        new_file.write_all(&buffer)?;
        new_file.write_all(&section_marker)?;

        // Also append domain table marker + empty table during initial setup
        new_file.write_all(DOMAIN_TABLE_START_MARKER)?;
        let empty_table = vec![0u8; std::mem::size_of::<DomainTable>()];
        new_file.write_all(&empty_table)?;

        drop(original);
        drop(new_file);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&self.executable_path)?;
            let mode = metadata.permissions().mode();
            let mut perms = fs::metadata(&temp_path)?.permissions();
            perms.set_mode(mode);
            fs::set_permissions(&temp_path, perms)?;
        }

        let backup_path = self.executable_path.with_extension("bak");
        fs::rename(&self.executable_path, &backup_path)?;
        fs::rename(&temp_path, &self.executable_path)?;

        return Ok(true);
    }
    // instead of allowing for multiple passwords and such,
    // this slimed down version stores one multidimentional structure and its
    // zones.
    fn load_all_passwords(&mut self) -> io::Result<()> {
        self.in_memory_cache.clear();
        self.metadata_cache.clear();

        let (section_marker, start_marker, end_marker, name_marker, desc_marker) =
            self.generate_markers();

        let file = match File::open(&self.executable_path) {
            Ok(f) => f,
            Err(e) => {
                println!("WARNING: Could not open executable: {}", e);
                return Ok(());
            }
        };

        let file_size = file.metadata()?.len();

        if file_size < section_marker.len() as u64 {
            return Ok(());
        }

        let mut file = File::open(&self.executable_path)?;
        let mut end_bytes = vec![0u8; section_marker.len()];
        file.seek(SeekFrom::End(-(section_marker.len() as i64)))?;

        if let Err(e) = file.read_exact(&mut end_bytes) {
            println!("WARNING: Failed to read end of binary: {}", e);
            return Ok(());
        }

        if end_bytes != section_marker {
            return Ok(());
        }
        // new buffer to load said geometric structure
        let mut file = File::open(&self.executable_path)?;
        let mut buffer = Vec::new();

        if let Err(e) = file.read_to_end(&mut buffer) {
            println!("WARNING: Failed: {}", e);
            return Ok(());
        }

        let section_end_pos = buffer.len();
        let section_start_pos = section_end_pos - section_marker.len();

        let search_begin = if section_start_pos > 10 * 1024 * 1024 {
            section_start_pos - 10 * 1024 * 1024
        } else {
            0
        };

        let mut current_pos = search_begin;

        let mut password_positions = Vec::new();

        while current_pos < section_start_pos {
            match Self::find_pattern(&buffer[current_pos..section_start_pos], &start_marker) {
                Some(offset) => {
                    let start_pos = current_pos + offset;
                    password_positions.push(start_pos);
                    current_pos = start_pos + start_marker.len();
                }
                None => break,
            }
        }

        for &start_pos in &password_positions {
            let current_pos = start_pos + start_marker.len();

            let name_marker_pos =
                match Self::find_pattern(&buffer[current_pos..section_start_pos], &name_marker) {
                    Some(offset) => current_pos + offset + name_marker.len(),
                    None => {
                        println!(
                            "WARNING: Start marker without NAME marker at position {}",
                            start_pos
                        );
                        continue;
                    }
                };

            let name_end_pos = match buffer[name_marker_pos..section_start_pos]
                .iter()
                .position(|&b| b == 0)
            {
                Some(offset) => name_marker_pos + offset,
                None => {
                    println!(
                        "WARNING: Name without end marker at position {}",
                        name_marker_pos
                    );
                    continue;
                }
            };

            // your geometric name
            let name_bytes = &buffer[name_marker_pos..name_end_pos];
            let name = match std::str::from_utf8(name_bytes) {
                Ok(s) => s.trim().to_string(),
                Err(_) => {
                    println!(
                        "WARNING: Invalid UTF-8 in name at position {}",
                        name_marker_pos
                    );
                    continue;
                }
            };

            let data_start = name_end_pos + 1;

            let end_pos_marker =
                match Self::find_pattern(&buffer[data_start..section_start_pos], &end_marker) {
                    Some(offset) => data_start + offset,
                    None => {
                        println!("WARNING: No end marker found after name '{}'", name);
                        continue;
                    }
                };

            let data_size = end_pos_marker - data_start;
            if data_size <= 0 {
                println!("WARNING: Empty data section for '{}'", name);
                continue;
            }

            if data_size > 10 * 1024 * 1024 {
                println!(
                    "WARNING: Data section too large for '{}': {} bytes",
                    name, data_size
                );
                continue;
            }

            let desc_marker_start = end_pos_marker + end_marker.len();

            let desc_marker_pos = match Self::find_pattern(
                &buffer[desc_marker_start..section_start_pos],
                &desc_marker,
            ) {
                Some(offset) => desc_marker_start + offset + desc_marker.len(),
                None => {
                    println!("WARNING: End marker without DESC marker for '{}'", name);
                    continue;
                }
            };

            let desc_end_pos = match buffer[desc_marker_pos..section_start_pos]
                .iter()
                .position(|&b| b == 0)
            {
                Some(offset) => desc_marker_pos + offset,
                None => {
                    println!("WARNING: Description without end marker for '{}'", name);
                    continue;
                }
            };

            let desc_bytes = &buffer[desc_marker_pos..desc_end_pos];
            let description = match std::str::from_utf8(desc_bytes) {
                Ok(s) => s.trim().to_string(),
                Err(_) => {
                    println!(
                        "WARNING: Invalid UTF-8 in description at position {}",
                        desc_marker_pos
                    );
                    continue;
                }
            };

            let data = buffer[data_start..end_pos_marker].to_vec();
            self.in_memory_cache.insert(name.clone(), data);
            self.metadata_cache.insert(name.clone(), description);
        }

        Ok(())
    }

    fn store(&mut self, name: String, description: String, data: &[u8]) -> io::Result<()> {
        self.in_memory_cache.insert(name.clone(), data.to_vec());
        self.metadata_cache
            .insert(name.clone(), description.clone());

        let (section_marker, start_marker, end_marker, name_marker, desc_marker) =
            self.generate_markers();

        let temp_path = self.executable_path.with_extension("new");

        let mut original = match File::open(&self.executable_path) {
            Ok(f) => f,
            Err(e) => {
                println!("ERROR: Failed to open original executable: {}", e);
                return Err(e);
            }
        };

        let mut original_buffer = Vec::new();
        original.read_to_end(&mut original_buffer)?;

        let has_section_marker = if original_buffer.len() >= section_marker.len() {
            let start_idx = original_buffer.len() - section_marker.len();
            original_buffer[start_idx..] == section_marker[..]
        } else {
            false
        };

        let section_start_pos = if has_section_marker {
            original_buffer.len() - section_marker.len()
        } else {
            original_buffer.len()
        };

        let mut new_exe = match File::create(&temp_path) {
            Ok(f) => f,
            Err(e) => {
                println!("ERROR: Failed to create temp binary: {}", e);
                return Err(e);
            }
        };

        new_exe.write_all(&original_buffer[..section_start_pos])?;

        for (existing_name, existing_data) in &self.in_memory_cache {
            if existing_name == &name {
                continue;
            }

            let existing_description = self
                .metadata_cache
                .get(existing_name)
                .cloned()
                .unwrap_or_else(|| "No description".to_string());

            new_exe.write_all(&start_marker)?;

            new_exe.write_all(&name_marker)?;
            new_exe.write_all(existing_name.as_bytes())?;
            new_exe.write_all(b"\0")?;

            new_exe.write_all(existing_data)?;

            new_exe.write_all(&end_marker)?;

            new_exe.write_all(&desc_marker)?;
            new_exe.write_all(existing_description.as_bytes())?;
            new_exe.write_all(b"\0")?;
        }

        new_exe.write_all(&start_marker)?;

        new_exe.write_all(&name_marker)?;
        new_exe.write_all(name.as_bytes())?;
        new_exe.write_all(b"\0")?;

        new_exe.write_all(data)?;

        new_exe.write_all(&end_marker)?;

        new_exe.write_all(&desc_marker)?;
        new_exe.write_all(description.as_bytes())?;
        new_exe.write_all(b"\0")?;

        new_exe.write_all(&section_marker)?;

        drop(original);
        drop(new_exe);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&self.executable_path)?;
            let mode = metadata.permissions().mode();
            let mut perms = fs::metadata(&temp_path)?.permissions();
            perms.set_mode(mode);
            fs::set_permissions(&temp_path, perms)?;
        }

        let backup_path = self.executable_path.with_extension("bak");

        match fs::rename(&self.executable_path, &backup_path) {
            Ok(_) => {}
            Err(e) => {
                println!("ERROR: Failed to create backup: {}", e);
                return Err(e);
            }
        }

        match fs::rename(&temp_path, &self.executable_path) {
            Ok(_) => {}
            Err(e) => {
                println!("ERROR: Failed to replace binary: {}", e);

                let _ = fs::rename(&backup_path, &self.executable_path);
                return Err(e);
            }
        }

        self.binary_modified = true;

        if !self.parent_mode {
            self.signal_binary_update()?;
        }

        Ok(())
    }
    // finds the geometry
    fn retrieve(&self, name: &str) -> io::Result<Option<(Vec<u8>, String)>> {
        if let Some(data) = self.in_memory_cache.get(name) {
            let description = self
                .metadata_cache
                .get(name)
                .cloned()
                .unwrap_or_else(|| "No description".to_string());

            return Ok(Some((data.clone(), description)));
        }

        Ok(None)
    }

    fn list_all(&self) -> Vec<(String, String)> {
        let mut all_passwords = Vec::new();

        for name in self.in_memory_cache.keys() {
            let description = self
                .metadata_cache
                .get(name)
                .cloned()
                .unwrap_or_else(|| "No description".to_string());

            all_passwords.push((name.clone(), description));
        }

        all_passwords
    }
    // ensures the binary gets updated
    fn signal_binary_update(&self) -> io::Result<()> {
        if let Some(tx) = &self.message_tx {
            if let Err(e) = tx.send(ProcessMessage::BinaryUpdated(self.executable_path.clone())) {
                println!("Failed to signal binary update: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to signal binary update",
                ));
            }
        }
        Ok(())
    }
}

// Domain table marker for binary layout
const DOMAIN_TABLE_START_MARKER: &[u8] = b"__DOMAIN_TABLE_START__";

// Domain slot entry (69 bytes total: 64 + 2 + 2 + 1)
#[derive(Clone, Copy)]
struct DomainSlot {
    domain_hash: [u8; 64], // Geometric hash of domain name
    counter: u16,          // Password version counter (0-65535)
    max_length: u16,       // Maximum password length (0 = unlimited)
    char_types: u8,        // Bit flags for allowed character types
}

impl DomainSlot {
    const EMPTY: Self = DomainSlot {
        domain_hash: [0u8; 64],
        counter: 0,
        max_length: 0,
        char_types: 127, // All 7 character types enabled by default
    };

    fn is_empty(&self) -> bool {
        self.domain_hash == [0u8; 64]
    }
}

struct DomainTable {
    slots: [DomainSlot; 512],
}

impl DomainTable {
    const fn new() -> Self {
        DomainTable {
            slots: [DomainSlot::EMPTY; 512],
        }
    }

    // Find slot index for a domain hash
    fn find_slot_by_hash(hash: &[u8; 64]) -> Option<usize> {
        unsafe {
            let table = &*std::ptr::addr_of!(DOMAIN_TABLE);
            table
                .slots
                .iter()
                .position(|slot| !slot.is_empty() && slot.domain_hash == *hash)
        }
    }

    fn get_counter(domain: &str, structure: &mut StructureSystem) -> Option<u16> {
        let hash = structure.hash_domain(domain);

        Self::find_slot_by_hash(&hash).map(|idx| unsafe { DOMAIN_TABLE.slots[idx].counter })
    }

    fn set_counter(
        domain: &str,
        counter: u16,
        structure: &mut StructureSystem,
    ) -> Result<(), &'static str> {
        let hash = structure.hash_domain(domain);

        unsafe {
            if let Some(idx) = Self::find_slot_by_hash(&hash) {
                let table = &mut *std::ptr::addr_of_mut!(DOMAIN_TABLE);
                table.slots[idx].counter = counter;
                return Ok(());
            }

            // Find first empty slot
            let table = &*std::ptr::addr_of!(DOMAIN_TABLE);
            if let Some(idx) = table.slots.iter().position(|s| s.is_empty()) {
                let table = &mut *std::ptr::addr_of_mut!(DOMAIN_TABLE);
                table.slots[idx] = DomainSlot {
                    domain_hash: hash,
                    counter,
                    max_length: 0,   // 0 = unlimited, if you need to.
                    char_types: 127, // Default: all types enabled
                };
                Ok(())
            } else {
                Err("Domain table full (512 slots)")
            }
        }
    }

    fn increment_counter(
        domain: &str,
        structure: &mut StructureSystem,
    ) -> Result<u16, &'static str> {
        let current = Self::get_counter(domain, structure).unwrap_or(0);
        let new_counter = current.saturating_add(1);
        Self::set_counter(domain, new_counter, structure)?;
        Ok(new_counter)
    }

    // get password rules for domain
    fn get_rules(domain: &str, structure: &mut StructureSystem) -> Option<(u16, u8)> {
        let hash = structure.hash_domain(domain);

        Self::find_slot_by_hash(&hash).map(|idx| unsafe {
            let slot = &DOMAIN_TABLE.slots[idx];
            (slot.max_length, slot.char_types)
        })
    }

    // Set password rules for domain
    // Creates new entry if domain doesn't exist
    // returns error if table is full (all 512 slots used. If this happens, rethink your life)
    fn set_rules(
        domain: &str,
        max_length: u16,
        char_types: u8,
        structure: &mut StructureSystem,
    ) -> Result<(), &'static str> {
        let hash = structure.hash_domain(domain);

        unsafe {
            // Try to find existing slot
            if let Some(idx) = Self::find_slot_by_hash(&hash) {
                let table = &mut *std::ptr::addr_of_mut!(DOMAIN_TABLE);
                table.slots[idx].max_length = max_length;
                table.slots[idx].char_types = char_types;
                return Ok(());
            }

            let table = &*std::ptr::addr_of!(DOMAIN_TABLE);
            if let Some(idx) = table.slots.iter().position(|s| s.is_empty()) {
                let table = &mut *std::ptr::addr_of_mut!(DOMAIN_TABLE);
                table.slots[idx] = DomainSlot {
                    domain_hash: hash,
                    counter: 0, // New domain starts at counter 0
                    max_length,
                    char_types,
                };
                Ok(())
            } else {
                Err("Domain table full (512 slots)")
            }
        }
    }

    fn save_to_binary(path: &std::path::Path) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        drop(file);

        let mut marker_pos = None;
        let search_start = if buffer.len() > 10 * 1024 * 1024 {
            buffer.len() - 10 * 1024 * 1024
        } else {
            0
        };

        for i in (search_start..buffer.len().saturating_sub(DOMAIN_TABLE_START_MARKER.len())).rev()
        {
            if &buffer[i..i + DOMAIN_TABLE_START_MARKER.len()] == DOMAIN_TABLE_START_MARKER {
                marker_pos = Some(i);
                break;
            }
        }

        let marker_pos = marker_pos.ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Domain table marker not found")
        })?;

        let table_offset = marker_pos + DOMAIN_TABLE_START_MARKER.len();
        let table_size = std::mem::size_of::<DomainTable>();

        unsafe {
            let table_bytes = std::slice::from_raw_parts(
                std::ptr::addr_of!(DOMAIN_TABLE) as *const u8,
                table_size,
            );
            buffer[table_offset..table_offset + table_size].copy_from_slice(table_bytes);
        }

        let temp_path = path.with_extension("new");
        let mut new_file = File::create(&temp_path)?;
        new_file.write_all(&buffer)?;
        drop(new_file);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(path)?;
            let mode = metadata.permissions().mode();
            let mut perms = fs::metadata(&temp_path)?.permissions();
            perms.set_mode(mode);
            fs::set_permissions(&temp_path, perms)?;
        }

        let backup_path = path.with_extension("bak");
        fs::rename(path, &backup_path)?;
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    fn load_from_binary(path: &std::path::Path) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut marker_pos = None;
        let search_start = if buffer.len() > 10 * 1024 * 1024 {
            buffer.len() - 10 * 1024 * 1024
        } else {
            0
        };

        for i in (search_start..buffer.len().saturating_sub(DOMAIN_TABLE_START_MARKER.len())).rev()
        {
            if &buffer[i..i + DOMAIN_TABLE_START_MARKER.len()] == DOMAIN_TABLE_START_MARKER {
                marker_pos = Some(i);
                break;
            }
        }

        if let Some(marker_pos) = marker_pos {
            let table_size = std::mem::size_of::<DomainTable>();

            if buffer.len() >= marker_pos + DOMAIN_TABLE_START_MARKER.len() + table_size {
                let table_data = &buffer[marker_pos + DOMAIN_TABLE_START_MARKER.len()
                    ..marker_pos + DOMAIN_TABLE_START_MARKER.len() + table_size];

                unsafe {
                    std::ptr::copy_nonoverlapping(
                        table_data.as_ptr(),
                        std::ptr::addr_of_mut!(DOMAIN_TABLE) as *mut u8,
                        table_size,
                    );
                }
            }
        }

        Ok(())
    }
}

struct SessionState {
    active_domain_hash: Option<[u8; 64]>,
    saved_counter: u16,
    active_counter: u16,
    is_preview_mode: bool,
    initialized: bool,
}

impl SessionState {
    const fn empty() -> Self {
        SessionState {
            active_domain_hash: None,
            saved_counter: 0,
            active_counter: 0,
            is_preview_mode: false,
            initialized: false,
        }
    }
}

#[allow(static_mut_refs)]
static mut DOMAIN_TABLE: DomainTable = DomainTable::new();

#[allow(static_mut_refs)]
static mut SESSION: SessionState = SessionState::empty();

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct StructurePoint {
    coordinates: Vec<i32>,
}

impl StructurePoint {
    fn new(dimensions: usize) -> Self {
        StructurePoint {
            coordinates: vec![0; dimensions],
        }
    }

    fn from_seed(seed: u64, dimensions: usize, range: i32) -> Self {
        let mut point = StructurePoint::new(dimensions);
        let mut rng_state = seed;

        for i in 0..dimensions {
            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let value = ((rng_state % (range as u64 * 2)) as i32) - range;
            point.coordinates[i] = value;
        }

        point
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.coordinates.len() as u32).to_ne_bytes());
        for &coord in &self.coordinates {
            bytes.extend_from_slice(&coord.to_ne_bytes());
        }
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), &'static str> {
        if bytes.len() < 4 {
            return Err("Invalid data: not enough bytes for StructurePoint");
        }

        let coord_count = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let required_bytes = 4 + (coord_count * 4);
        if bytes.len() < required_bytes {
            return Err("Invalid data: not enough bytes for coordinates");
        }

        let mut coordinates = Vec::with_capacity(coord_count);
        for i in 0..coord_count {
            let start = 4 + (i * 4);
            coordinates.push(i32::from_ne_bytes([
                bytes[start],
                bytes[start + 1],
                bytes[start + 2],
                bytes[start + 3],
            ]));
        }

        Ok((StructurePoint { coordinates }, required_bytes))
    }
}
#[derive(Debug, Clone)]
struct ContinuousPosition {
    coordinates: Vec<f64>,
}

// links all previous positions together
impl ContinuousPosition {
    fn new(dimensions: usize) -> Self {
        ContinuousPosition {
            coordinates: vec![0.0; dimensions],
        }
    }

    fn hash_position(&self, seed: u64) -> u64 {
        let mut hash = seed;
        for &coord in &self.coordinates {
            let fixed = (coord * 1000.0) as i64;
            hash = hash.wrapping_mul(31).wrapping_add(fixed as u64);
        }
        hash
    }
}
#[derive(Clone)]
struct StructureSystem {
    //multiple active and interactable dimensions
    dimensions: usize,
    //active as in navigational lighthouses
    active_points: HashSet<StructurePoint>,
    //unifying the geometry translation with movement
    char_to_point: HashMap<u32, StructurePoint>,
    //range field for movement
    coordinate_range: i32,
    //initialization seed for the geometry
    original_seed: u64,
    //the name of the unique geometry
    name: String,
    //zoned coordinate pool for multi-dimensional treversal
    character_set: Vec<u32>,

    current_position: ContinuousPosition,
    structure_bounds: (Vec<f64>, Vec<f64>),
    base_step_size: f64,
    step_variance: f64,

    accumulated_path_memory: u8,
}

impl StructureSystem {
    fn new(seed: u64, dimensions: usize, range: i32) -> Self {
        StructureSystem {
            dimensions,
            active_points: HashSet::new(),
            char_to_point: HashMap::new(),
            coordinate_range: range,
            original_seed: seed,
            name: String::from("default"),
            character_set: Vec::new(),
            current_position: ContinuousPosition::new(dimensions),
            structure_bounds: (vec![-30.0; dimensions], vec![30.0; dimensions]),
            base_step_size: 3.0,
            step_variance: 2.0,
            accumulated_path_memory: 0,
        }
    }

    fn reset_position(&mut self) {
        self.accumulated_path_memory = 0;
    }

    fn full_reset(&mut self) {
        self.current_position = ContinuousPosition::new(self.dimensions);
        self.accumulated_path_memory = 0;
    }

    fn transform_char(&mut self, keycode: u32, extra_chars_count: usize) -> Vec<u32> {
        let (direction, distance) = self.calculate_movement(keycode);
        let start_position = self.current_position.clone();

        self.update_position(&direction, distance);

        let output_chars = self.generate_output_from_path(
            &start_position,
            &direction,
            distance,
            extra_chars_count,
        );

        output_chars
    }

    fn calculate_movement(&self, keycode: u32) -> (Vec<f64>, f64) {
        let position_hash = self.current_position.hash_position(self.original_seed);
        let movement_seed = self.original_seed ^ position_hash ^ (keycode as u64);

        let direction = self.generate_direction(movement_seed);
        let distance = self.generate_distance(movement_seed);

        (direction, distance)
    }

    fn generate_direction(&self, seed: u64) -> Vec<f64> {
        let mut direction = vec![0.0; self.dimensions];
        let mut rng_state = seed;

        for i in 0..self.dimensions {
            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            direction[i] = (rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0;
        }

        let magnitude: f64 = direction.iter().map(|x| x * x).sum::<f64>().sqrt();
        if magnitude > 0.0 {
            for value in &mut direction {
                *value /= magnitude;
            }
        }

        direction
    }

    fn generate_distance(&self, seed: u64) -> f64 {
        let mut rng_state = seed;
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        let variance = (rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0;
        self.base_step_size + variance * self.step_variance
    }

    fn update_position(&mut self, direction: &Vec<f64>, distance: f64) {
        for i in 0..self.dimensions {
            let mut new_coord = self.current_position.coordinates[i] + direction[i] * distance;

            let (min_bound, max_bound) = (self.structure_bounds.0[i], self.structure_bounds.1[i]);

            if new_coord < min_bound {
                new_coord = min_bound + (min_bound - new_coord);
            } else if new_coord > max_bound {
                new_coord = max_bound - (new_coord - max_bound);
            }

            self.current_position.coordinates[i] = new_coord;
        }

        let coord_sum: i64 = self
            .current_position
            .coordinates
            .iter()
            .map(|&coord| coord as i64)
            .sum();

        self.accumulated_path_memory = self.accumulated_path_memory.wrapping_add(coord_sum as u8);
    }

    fn generate_output_from_path(
        &self,
        start: &ContinuousPosition,
        direction: &Vec<f64>,
        distance: f64,
        extra_chars_count: usize,
    ) -> Vec<u32> {
        let mut output = Vec::new();
        let total_chars = extra_chars_count + 1;

        for i in 0..total_chars {
            let fraction = i as f64 / total_chars as f64;

            let mut path_position = start.clone();
            for dim in 0..self.dimensions {
                path_position.coordinates[dim] += direction[dim] * distance * fraction;
            }

            let char_seed = path_position.hash_position(self.original_seed);
            let base_char_idx = (char_seed % self.character_set.len() as u64) as usize;

            let final_char = self.apply_path_memory_to_character(base_char_idx);
            output.push(final_char);
        }

        output
    }

    fn apply_path_memory_to_character(&self, base_char_index: usize) -> u32 {
        if self.character_set.is_empty() {
            return 0;
        }

        let final_index = if self.accumulated_path_memory % 2 == 0 {
            (base_char_index + 1) % self.character_set.len()
        } else {
            (base_char_index + self.character_set.len() - 1) % self.character_set.len()
        };

        self.character_set[final_index]
    }

    // Scrambles domain name using geometric structure
    // Returns deterministic 64-byte identifier
    fn hash_domain(&mut self, domain: &str) -> [u8; 64] {
        let saved_position = self.current_position.clone();
        let saved_seed = self.original_seed;
        let saved_memory = self.accumulated_path_memory;

        const DOMAIN_HASH_SEED: u64 = 0x444F4D41494E5F48;
        self.original_seed = DOMAIN_HASH_SEED;
        self.full_reset();

        let mut hash_bytes = Vec::with_capacity(64);

        for ch in domain.chars() {
            let keycode = ch as u32;

            let output_codes = self.transform_char(keycode, 7);

            for &code in &output_codes {
                hash_bytes.push((code % 256) as u8);

                if hash_bytes.len() >= 64 {
                    break;
                }
            }

            if hash_bytes.len() >= 64 {
                break;
            }
        }

        // Pad if domain was very short (e.g., "a.co")
        while hash_bytes.len() < 64 {
            let padding_codes = self.transform_char(0, 7);
            for &code in &padding_codes {
                hash_bytes.push((code % 256) as u8);
                if hash_bytes.len() >= 64 {
                    break;
                }
            }
        }

        self.original_seed = saved_seed;
        self.current_position = saved_position;
        self.accumulated_path_memory = saved_memory;

        let mut result = [0u8; 64];
        result.copy_from_slice(&hash_bytes[..64]);
        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&(self.dimensions as u32).to_ne_bytes());
        bytes.extend_from_slice(&self.coordinate_range.to_ne_bytes());
        bytes.extend_from_slice(&self.original_seed.to_ne_bytes());

        let name_bytes = self.name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_ne_bytes());
        bytes.extend_from_slice(name_bytes);

        bytes.extend_from_slice(&(self.character_set.len() as u32).to_ne_bytes());
        for &code in &self.character_set {
            bytes.extend_from_slice(&code.to_ne_bytes());
        }

        bytes.extend_from_slice(&(self.active_points.len() as u32).to_ne_bytes());
        for point in &self.active_points {
            let point_bytes = point.to_bytes();
            bytes.extend(point_bytes);
        }

        bytes.extend_from_slice(&(self.char_to_point.len() as u32).to_ne_bytes());
        for (&key, point) in &self.char_to_point {
            bytes.extend_from_slice(&key.to_ne_bytes());
            let point_bytes = point.to_bytes();
            bytes.extend(point_bytes);
        }

        bytes.extend_from_slice(&self.base_step_size.to_ne_bytes());
        bytes.extend_from_slice(&self.step_variance.to_ne_bytes());

        bytes.extend_from_slice(&self.accumulated_path_memory.to_ne_bytes());

        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 16 {
            return Err("Invalid data: not enough bytes for StructureSystem");
        }

        let mut offset = 0;

        let dimensions = u32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        let coordinate_range = i32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        let original_seed = u64::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        if bytes.len() < offset + 4 {
            return Err("Invalid data: not enough bytes for name data");
        }
        let name_len = u32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        if bytes.len() < offset + name_len {
            return Err("Invalid data: not enough bytes for nameu");
        }
        let name = match String::from_utf8(bytes[offset..offset + name_len].to_vec()) {
            Ok(s) => s,
            Err(_) => return Err("Invalid UTF-8 in name"),
        };
        offset += name_len;

        if bytes.len() < offset + 4 {
            return Err("Invalid data: not enough bytes for character set you selected for");
        }
        let char_set_len = u32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        let mut character_set = Vec::with_capacity(char_set_len);
        for _ in 0..char_set_len {
            if bytes.len() < offset + 4 {
                return Err("Invalid data: not enough bytes for character INI");
            }
            let code = u32::from_ne_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            character_set.push(code);
            offset += 4;
        }

        if bytes.len() < offset + 4 {
            return Err("Invalid data: not enough bytes for active points count");
        }
        let active_points_count = u32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        let mut active_points = HashSet::new();
        for _ in 0..active_points_count {
            if offset >= bytes.len() {
                return Err("Invalid data: not enough bytes for active point Z");
            }
            match StructurePoint::from_bytes(&bytes[offset..]) {
                Ok((point, bytes_read)) => {
                    active_points.insert(point);
                    offset += bytes_read;
                }
                Err(e) => return Err(e),
            }
        }

        if bytes.len() < offset + 4 {
            return Err("Invalid data: not enough bytes for char_to_point count");
        }
        let mapping_count = u32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        let mut char_to_point = HashMap::new();
        for _ in 0..mapping_count {
            if bytes.len() < offset + 4 {
                return Err("Invalid data: not enough bytes for keycode");
            }
            let keycode = u32::from_ne_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offset += 4;

            if offset >= bytes.len() {
                return Err("Invalid data: not enough bytes for point");
            }
            match StructurePoint::from_bytes(&bytes[offset..]) {
                Ok((point, bytes_read)) => {
                    char_to_point.insert(keycode, point);
                    offset += bytes_read;
                }
                Err(e) => return Err(e),
            }
        }

        let (base_step_size, step_variance) = if bytes.len() >= offset + 16 {
            let base_step = f64::from_ne_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            offset += 8;

            let step_var = f64::from_ne_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            offset += 8;
            (base_step, step_var)
        } else {
            (3.0, 2.0)
        };

        let accumulated_path_memory = if bytes.len() >= offset + 1 {
            bytes[offset]
        } else {
            0
        };

        Ok(StructureSystem {
            dimensions,
            active_points,
            char_to_point,
            coordinate_range,
            original_seed,
            name,
            character_set,
            current_position: ContinuousPosition::new(dimensions),
            structure_bounds: (vec![-30.0; dimensions], vec![30.0; dimensions]),
            base_step_size,
            step_variance,
            accumulated_path_memory,
        })
    }

    fn set_character_set(&mut self, character_set: Vec<u32>) {
        self.character_set = character_set;
    }

    fn generate_structure(&mut self, initial_password: &[char], keycodes: &[u32]) {
        self.set_character_set(keycodes.to_vec());

        let center = StructurePoint::new(self.dimensions);
        self.active_points.insert(center.clone());

        for &keycode in keycodes {
            let point = StructurePoint::from_seed(
                self.original_seed ^ (keycode as u64),
                self.dimensions,
                self.coordinate_range,
            );

            self.char_to_point.insert(keycode, point.clone());
        }

        if !initial_password.is_empty() {
            let mut current_point = center.clone();

            for (i, &ch) in initial_password.iter().enumerate() {
                let ch_code = ch as u32;

                let point = if let Some(point) = self.char_to_point.get(&ch_code).cloned() {
                    point
                } else {
                    let new_point = StructurePoint::from_seed(
                        self.original_seed ^ (ch_code as u64),
                        self.dimensions,
                        self.coordinate_range,
                    );
                    self.char_to_point.insert(ch_code, new_point.clone());
                    new_point
                };

                self.create_path(&current_point, &point);

                if i < initial_password.len() - 1 {
                    self.create_structure_feature(&point, ch_code, i as u64);
                }

                current_point = point;
            }

            println!("generated with {} events", self.active_points.len());
        } else {
            println!("No initial phrase provided, building on a basic one");

            self.create_basic_structure(keycodes);
        }

        self.calculate_structure_bounds();
    }

    fn calculate_structure_bounds(&mut self) {
        let mut min_coords = vec![f64::INFINITY; self.dimensions];
        let mut max_coords = vec![f64::NEG_INFINITY; self.dimensions];

        for point in &self.active_points {
            for i in 0..self.dimensions {
                min_coords[i] = min_coords[i].min(point.coordinates[i] as f64);
                max_coords[i] = max_coords[i].max(point.coordinates[i] as f64);
            }
        }

        let padding = 10.0;
        for i in 0..self.dimensions {
            min_coords[i] -= padding;
            max_coords[i] += padding;
        }

        self.structure_bounds = (min_coords, max_coords);
    }

    fn create_path(&mut self, start: &StructurePoint, end: &StructurePoint) {
        let steps = 5;
        for step in 0..=steps {
            let mut new_point = StructurePoint::new(self.dimensions);

            for dim in 0..self.dimensions {
                let start_coord = start.coordinates[dim];
                let end_coord = end.coordinates[dim];
                let coord = start_coord + (end_coord - start_coord) * step / steps;
                new_point.coordinates[dim] = coord;
            }

            self.active_points.insert(new_point);
        }
    }
    // deterministic structure creation, to ensure complex high dimensional internal structures
    fn create_structure_feature(&mut self, center: &StructurePoint, char_code: u32, index: u64) {
        let feature_size = 10 + (char_code % 20) as usize;

        let feature_type = (char_code + index as u32) % 5;
        let feature_seed = self.original_seed ^ (char_code as u64) ^ index;

        match feature_type {
            0 => self.create_deterministic_spike(center, feature_size, feature_seed),
            1 => self.create_deterministic_blob(center, feature_size, feature_seed),
            2 => self.create_deterministic_ring(center, feature_size, feature_seed),
            3 => self.create_deterministic_spiral(center, feature_size, feature_seed),
            _ => self.create_deterministic_scatter(center, feature_size, feature_seed),
        }
    }

    fn create_deterministic_spike(&mut self, center: &StructurePoint, size: usize, seed: u64) {
        let mut direction = StructurePoint::new(self.dimensions);
        let mut rng_state = seed;

        for dim in 0..self.dimensions {
            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            direction.coordinates[dim] = ((rng_state % 3) as i32) - 1;
        }

        for i in 1..=size as i32 {
            let mut point = center.clone();
            for dim in 0..self.dimensions {
                point.coordinates[dim] += direction.coordinates[dim] * i;
            }
            self.active_points.insert(point);
        }
    }

    fn create_deterministic_blob(&mut self, center: &StructurePoint, size: usize, seed: u64) {
        let mut rng_state = seed;

        for _i in 0..size {
            let mut point = center.clone();

            for dim in 0..self.dimensions {
                rng_state = rng_state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let offset = (rng_state % 5) as i32 - 2;
                point.coordinates[dim] += offset;
            }

            self.active_points.insert(point);
        }
    }

    fn create_deterministic_ring(&mut self, center: &StructurePoint, size: usize, seed: u64) {
        let radius = (size / 3) as i32;
        let mut rng_state = seed;

        for i in 0..size {
            let angle = (i as f64 / size as f64) * 2.0 * std::f64::consts::PI;
            let mut point = center.clone();

            let dims = self.dimensions.min(3);

            if dims >= 2 {
                point.coordinates[0] += (radius as f64 * angle.cos()) as i32;
                point.coordinates[1] += (radius as f64 * angle.sin()) as i32;

                if dims >= 3 {
                    rng_state = rng_state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    point.coordinates[2] += (rng_state % 5) as i32 - 2;
                }
            }

            self.active_points.insert(point);
        }
    }

    fn create_deterministic_spiral(&mut self, center: &StructurePoint, size: usize, seed: u64) {
        let mut rng_state = seed;

        for i in 1..=size {
            let angle = (i as f64 / 4.0) * std::f64::consts::PI;
            let radius = i as i32 / 2;
            let mut point = center.clone();

            let dims = self.dimensions.min(3);

            if dims >= 2 {
                point.coordinates[0] += (radius as f64 * angle.cos()) as i32;
                point.coordinates[1] += (radius as f64 * angle.sin()) as i32;

                if dims >= 3 {
                    rng_state = rng_state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let z_offset = (rng_state % 3) as i32 + (i as i32 / 3);
                    point.coordinates[2] += z_offset;
                }
            }

            for dim in 3..self.dimensions {
                rng_state = rng_state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let offset = (rng_state % 7) as i32 - 3;
                point.coordinates[dim] += offset;
            }

            self.active_points.insert(point);
        }
    }

    fn create_deterministic_scatter(&mut self, center: &StructurePoint, size: usize, seed: u64) {
        let mut rng_state = seed;

        for _i in 0..size {
            let mut point = center.clone();

            for dim in 0..self.dimensions {
                rng_state = rng_state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let offset = (rng_state % 11) as i32 - 5;
                point.coordinates[dim] += offset;
            }

            self.active_points.insert(point);
        }
    }

    fn create_basic_structure(&mut self, keycodes: &[u32]) {
        for &keycode in keycodes {
            let point = StructurePoint::from_seed(
                self.original_seed ^ keycode as u64,
                self.dimensions,
                self.coordinate_range,
            );
            self.char_to_point.insert(keycode, point.clone());

            self.active_points.insert(point.clone());

            let feature_seed = self.original_seed ^ (keycode as u64);
            let feature_type = feature_seed % 5;

            match feature_type {
                0 => self.create_deterministic_spike(&point, 5, feature_seed),
                1 => self.create_deterministic_blob(&point, 6, feature_seed),
                2 => self.create_deterministic_ring(&point, 7, feature_seed),
                3 => self.create_deterministic_spiral(&point, 8, feature_seed),
                _ => self.create_deterministic_scatter(&point, 5, feature_seed),
            }
        }

        let points: Vec<_> = self.char_to_point.values().cloned().collect();
        let limit = points.len().min(30);

        for i in 0..limit {
            if i + 1 < points.len() {
                self.create_path(&points[i], &points[(i + 1) % points.len()]);
            }
        }
    }

    fn modify_with_timing(&mut self, keycode: u32, timing_ms: u64, timestamp: u64) {
        if let Some(point) = self.char_to_point.get(&keycode).cloned() {
            let mut timing_point = point.clone();

            let mod_seed = self.original_seed ^ keycode as u64 ^ timing_ms ^ (timestamp % 1000);

            let is_forward = timing_ms % 2 == 0;

            for dim in 0..self.dimensions {
                let modifier = ((timing_ms + dim as u64) % 5) as i32 - 2;

                if is_forward {
                    timing_point.coordinates[dim] += modifier;
                } else {
                    timing_point.coordinates[dim] -= modifier;
                }
            }

            self.active_points.insert(timing_point.clone());

            let feature_type = mod_seed % 3;

            if feature_type == 0 {
                self.create_deterministic_blob(&timing_point, 3, mod_seed);
            } else if feature_type == 1 {
                self.create_deterministic_spike(&timing_point, 3, mod_seed);
            } else {
                self.create_deterministic_scatter(&timing_point, 3, mod_seed);
            }
        }
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[derive(Clone)]
struct SavedPassword {
    name: String,
    description: String,
    structure_system: StructureSystem,
    created_date: u64,
    extra_chars_count: usize,
}

impl SavedPassword {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let name_bytes = self.name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_ne_bytes());
        bytes.extend_from_slice(name_bytes);

        let desc_bytes = self.description.as_bytes();
        bytes.extend_from_slice(&(desc_bytes.len() as u32).to_ne_bytes());
        bytes.extend_from_slice(desc_bytes);

        bytes.extend_from_slice(&self.created_date.to_ne_bytes());
        bytes.extend_from_slice(&(self.extra_chars_count as u32).to_ne_bytes());

        let structure_bytes = self.structure_system.to_bytes();
        bytes.extend_from_slice(&(structure_bytes.len() as u32).to_ne_bytes());
        bytes.extend(structure_bytes);

        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut offset = 0;

        let name_len = u32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        if bytes.len() < offset + name_len {
            return Err("Invalid data: not enough bytes for name");
        }
        let name = match String::from_utf8(bytes[offset..offset + name_len].to_vec()) {
            Ok(s) => s,
            Err(_) => return Err("Invalid UTF-8 in name"),
        };
        offset += name_len;

        if bytes.len() < offset + 4 {
            return Err("Invalid data: not enough bytes for description length");
        }
        let desc_len = u32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        if bytes.len() < offset + desc_len {
            return Err("Invalid data: not enough bytes for description");
        }
        let description = match String::from_utf8(bytes[offset..offset + desc_len].to_vec()) {
            Ok(s) => s,
            Err(_) => return Err("Invalid UTF-8 in description"),
        };
        offset += desc_len;

        if bytes.len() < offset + 8 {
            return Err("Invalid data: not enough bytes for created date");
        }
        let created_date = u64::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        let mut extra_chars_count = 3;

        if bytes.len() >= offset + 4 {
            extra_chars_count = u32::from_ne_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;
        } else {
            println!("Warning: Using default value for extra_chars_count");
        }

        if bytes.len() < offset + 4 {
            return Err("Invalid data: not enough bytes for Structure system length");
        }
        let structure_len = u32::from_ne_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        if bytes.len() < offset + structure_len {
            return Err("Invalid data: not enough bytes for Structure system");
        }
        let structure_system = StructureSystem::from_bytes(&bytes[offset..offset + structure_len])?;

        Ok(SavedPassword {
            name,
            description,
            structure_system,
            created_date,
            extra_chars_count,
        })
    }
}

#[derive(Clone)]
pub struct PasswordManager {
    saved_passwords: Vec<SavedPassword>,
    storage: BinaryStorageManager,
    active_structure_idx: Option<usize>,
}

impl PasswordManager {
    fn new(
        parent_mode: bool,
        tx: Option<Sender<ProcessMessage>>,
        _rx: Option<Receiver<ProcessMessage>>,
        silent: bool,
    ) -> io::Result<Self> {
        let storage = BinaryStorageManager::new(parent_mode, tx)?;

        let mut manager = PasswordManager {
            saved_passwords: Vec::new(),
            storage,
            active_structure_idx: None,
        };

        manager.load_all_passwords(silent)?;

        if !manager.saved_passwords.is_empty() {
            manager.active_structure_idx = Some(0);
            if !silent {
                eprintln!("Active Structure: {}", manager.saved_passwords[0].name);
            }
        }

        Ok(manager)
    }

    fn load_all_passwords(&mut self, silent: bool) -> io::Result<()> {
        self.saved_passwords.clear();

        let password_entries = self.storage.list_all();

        for (name, description) in password_entries {
            if let Ok(Some((data, _))) = self.storage.retrieve(&name) {
                match SavedPassword::from_bytes(&data) {
                    Ok(mut password) => {
                        password.description = description;
                        self.saved_passwords.push(password);
                    }
                    Err(e) => {
                        println!("Error loading structure '{}': {}", name, e);
                    }
                }
            }
        }

        if !silent {
            eprintln!(
                "Loaded {} saved password configurations",
                self.saved_passwords.len()
            );
        }

        Ok(())
    }

    fn save_password(&mut self, password: &SavedPassword) -> io::Result<()> {
        let bytes = password.to_bytes();

        self.storage
            .store(password.name.clone(), password.description.clone(), &bytes)?;

        Ok(())
    }

    fn add_password(&mut self, password: SavedPassword) -> io::Result<()> {
        self.save_password(&password)?;

        self.saved_passwords.push(password);

        // to ensure ease of use, this checks is the first password and make it active
        if self.saved_passwords.len() == 1 {
            self.active_structure_idx = Some(0);
        }

        Ok(())
    }

    fn create_password_setup(
        name: &str,
        description: &str,
        structure_system: &mut StructureSystem,
        keycodes: &[u32],
        extra_chars_count: usize,
    ) -> Result<SavedPassword, std::io::Error> {
        use std::io::{self, Read, Write};

        println!("\n=== PASSWORD SETUP PHASE ===");
        println!("This is a one-time setup to create your configuration.");
        println!("Type your sequence naturally.");
        println!("Press ESC when finished.");

        setup_raw_mode();

        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let mut buffer = [0; 1];
        let mut last_keypress_time = std::time::Instant::now();

        structure_system.reset_position();

        let mut collected_chars = Vec::new();
        let mut current_input = String::new();
        let mut display_input = String::new();
        let mut display_count = 0;

        println!("\nStart typing your sequence now:");

        loop {
            match stdin.read_exact(&mut buffer) {
                Ok(_) => {
                    let keycode = buffer[0] as u32;

                    if keycode == 27 {
                        println!("\nSetup complete!");
                        break;
                    }

                    let now = std::time::Instant::now();
                    let duration = now.duration_since(last_keypress_time);
                    let timing_ms = duration.as_millis() as u64;
                    last_keypress_time = now;

                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis() as u64;

                    if keycode == 8 || keycode == 127 {
                        if !display_input.is_empty() {
                            display_input.pop();
                            if display_count > 0 {
                                display_count -= 1;
                            }
                            print!(
                                "\r{}                              \r{} characters typed: {}",
                                " ".repeat(50),
                                display_count,
                                display_input
                            );
                            io::stdout().flush()?;
                        }

                        structure_system.modify_with_timing(keycode, timing_ms, timestamp);
                        let output_chars =
                            structure_system.transform_char(keycode, extra_chars_count);
                        collected_chars
                            .extend(output_chars.iter().filter_map(|&code| char::from_u32(code)));
                    } else if keycodes.contains(&keycode) {
                        if let Some(c) = char::from_u32(keycode) {
                            current_input.push(c);
                            display_input.push(c);
                        }

                        display_count += 1;

                        structure_system.modify_with_timing(keycode, timing_ms, timestamp);

                        let output_chars =
                            structure_system.transform_char(keycode, extra_chars_count);
                        collected_chars
                            .extend(output_chars.iter().filter_map(|&code| char::from_u32(code)));

                        print!("\r{} characters typed: {}", display_count, display_input);
                        io::stdout().flush()?;
                    }
                }
                Err(e) => {
                    println!("\nError reading from stdin: {}", e);
                    break;
                }
            }
        }
        #[cfg(unix)]
        restore_terminal();

        structure_system.set_name(name.to_string());

        structure_system.full_reset();

        let created_date = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("You a time traveler? Time went backwards")
            .as_secs();

        let saved_password = SavedPassword {
            name: name.to_string(),
            description: description.to_string(),
            structure_system: structure_system.clone(),
            created_date,
            extra_chars_count,
        };

        println!("\n\n✓ Configuration created successfully!");
        println!("You typed {} characters for setup.", display_count);

        Ok(saved_password)
    }
}

fn run_parent_process(auto_exit: bool) -> io::Result<()> {
    println!("Starting Void Vault...");
    println!("Maximized and unending void");
    println!("Reapplied inside the geometry");
    println!("zero password setup");

    let (tx_to_child, _rx_from_parent) = mpsc::channel();
    let (_tx_to_parent, rx_from_child) = mpsc::channel();

    let executable_path = std::env::current_exe()?;
    let mut child_args = vec!["--child-process".to_string()];
    if auto_exit {
        child_args.push("--auto-exit".to_string());
    }

    let mut child = Command::new(&executable_path)
        .args(&child_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    match rx_from_child.recv() {
        Ok(ProcessMessage::ChildReady) => {
            println!("Child birthed and ready");
        }
        _ => {
            println!("Failed to receive ready signal from child, it clawled back in");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "The child brings dishonor to the family by failing initialization",
            ));
        }
    }

    loop {
        match rx_from_child.recv() {
            Ok(ProcessMessage::BinaryUpdated(_new_binary_path)) => {
                println!("Received newaged binary update notification from child");

                thread::sleep(std::time::Duration::from_millis(100));

                tx_to_child
                    .send(ProcessMessage::BinaryUpdateComplete)
                    .unwrap_or_else(|_| {
                        println!(
                            "Failed to use dope slang and signal binary update completion to child"
                        );
                    });
            }
            Ok(ProcessMessage::ShutdownChild) => {
                println!("Child sent a requested to be unborn");
                break;
            }
            Err(_) => {
                break;
            }
            _ => {}
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                println!("Child exited with status: {}", status);
                break;
            }
            Ok(None) => {}
            Err(e) => {
                println!("Error checking the child, prognosis: {}", e);
                break;
            }
        }
    }

    match child.wait() {
        Ok(status) => {
            println!("Child exited with status: {}", status);
        }
        Err(e) => {
            println!("Error waiting for child: {}", e);
        }
    }

    Ok(())
}

fn run_simple_setup(
    password_manager: &mut PasswordManager,
    seed: u64,
    auto_exit: bool,
) -> io::Result<()> {
    println!("");
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║                     WELCOME TO THE VOID VAULT                      ║");
    println!("║                         First-Time Setup                           ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    println!("The Void Vault creates secure passwords for all your accounts.");
    println!("Setup takes about 1 minute.\n");

    // 8 output characters per input
    let extra_chars_count = 7;

    // The full UTF-8 character set
    let mut keycodes = Vec::new();
    keycodes.extend(32..127); // ASCII printable
    keycodes.extend(161..1024); // Extended Latin, Greek, Cyrillic, etc.
    keycodes.extend(1024..5000); // CJK, Arabic, Hebrew, etc.
    keycodes.extend(8192..8500); // Various symbols
    keycodes.extend(9000..9500); // More symbols
    keycodes.extend(128512..128591); // Emoji

    println!("══════════════════════════════════════════════════════════════════════");
    println!("Create Your Void Vault");
    println!("══════════════════════════════════════════════════════════════════════");
    println!("\nType a phrase of at least 40 characters.");
    println!("This phrase will create your unique vault.\n");
    println!("Instructions:");
    println!("  • Type any phrase, sentence, or random characters");
    println!("  • At least 40 characters recommended (longer = better)");
    println!("  • Type naturally - your rhythm adds uniqueness");
    println!("  • This phrase is ONLY for setup, not for generating passwords");
    println!("  • Press ESC when finished\n");
    println!("Note: Your passwords will use the full UTF-8 character set.");
    println!("      The browser extension will handle website requirements.\n");

    println!("Press Enter to begin...");
    let mut ready = String::new();
    io::stdin().read_line(&mut ready)?;

    let dimensions = 7;
    let coordinate_range = 10 + dimensions as i32;
    let mut structure_system = StructureSystem::new(seed, dimensions as usize, coordinate_range);
    structure_system.set_character_set(keycodes.clone());

    let initial_password: Vec<char> = Vec::new();
    structure_system.generate_structure(&initial_password, &keycodes);

    let saved_password = PasswordManager::create_password_setup(
        "main",
        "Primary configuration",
        &mut structure_system,
        &keycodes,
        extra_chars_count,
    )?;

    password_manager.add_password(saved_password)?;
    password_manager.active_structure_idx = Some(0);

    println!("\n══════════════════════════════════════════════════════════════════════");
    println!("✓ SETUP COMPLETE!");
    println!("══════════════════════════════════════════════════════════════════════");
    println!("\nYour password system is ready!\n");
    println!("How to use the Void Vault:");
    println!("  1. Go to any website login page");
    println!("  2. Click the password field");
    println!("  3. Press Ctrl+Shift+S in your browser");
    println!("  4. Type your password phrase");
    println!("  5. Press Enter\n");
    println!("⚠️  IMPORTANT: Backup this file to a safe place!");
    println!("    Without it, you cannot access your passwords.\n");

    if !auto_exit {
        println!("Press Enter to continue...");
        let mut cont = String::new();
        io::stdin().read_line(&mut cont)?;
    }

    Ok(())
}

fn run_child_process(auto_exit: bool) -> io::Result<()> {
    println!("Starting the Void Vault");

    let (tx_to_parent, _rx_from_child) = mpsc::channel();
    let (_tx_to_child, rx_from_parent) = mpsc::channel();

    tx_to_parent
        .send(ProcessMessage::ChildReady)
        .unwrap_or_else(|_| {
            println!("Failed to say it is ready to be a parent");
        });

    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Do you own a hot tub? Time went backwards");
    let seed = since_epoch.as_secs();

    let mut password_manager = match PasswordManager::new(
        false,
        Some(tx_to_parent.clone()),
        Some(rx_from_parent),
        false,
    ) {
        Ok(manager) => manager,
        Err(e) => {
            println!("Error initializing password manager: {}", e);
            println!("passwords won't be saved");

            let storage = match BinaryStorageManager::new(false, Some(tx_to_parent.clone())) {
                Ok(s) => s,
                Err(_) => {
                    println!("Cannot initialize storage. Exiting.");
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Storage initialization failed",
                    ));
                }
            };

            PasswordManager {
                saved_passwords: Vec::new(),
                storage,
                active_structure_idx: None,
            }
        }
    };

    if password_manager.active_structure_idx.is_some() {
        if !auto_exit {
            run_interactive_mode(&mut password_manager)?;
        }
    } else if password_manager.saved_passwords.is_empty() {
        if let Ok(_) = run_simple_setup(&mut password_manager, seed, auto_exit) {
            if password_manager.active_structure_idx.is_some() && !auto_exit {
                run_interactive_mode(&mut password_manager)?;
            }
        }
    } else {
        password_manager.active_structure_idx = Some(0);
        if !auto_exit {
            run_interactive_mode(&mut password_manager)?;
        }
    }

    tx_to_parent
        .send(ProcessMessage::ShutdownChild)
        .unwrap_or_else(|_| {
            println!("Failed to suggest to the parent to abort itself");
        });

    println!("Exiting program.");
    Ok(())
}

fn run_interactive_mode(password_manager: &mut PasswordManager) -> io::Result<()> {
    loop {
        println!("\n=== VOID VAULT ===");
        let (structure_name, _description, _extra_chars) =
            if let Some(idx) = password_manager.active_structure_idx {
                if idx < password_manager.saved_passwords.len() {
                    let structure = &password_manager.saved_passwords[idx];
                    (
                        structure.name.clone(),
                        structure.description.clone(),
                        structure.extra_chars_count,
                    )
                } else {
                    (
                        "No active password configuration".to_string(),
                        "".to_string(),
                        0,
                    )
                }
            } else {
                (
                    "No active password configuration".to_string(),
                    "".to_string(),
                    0,
                )
            };

        println!("Active configuration: {}", structure_name);
        println!("\nEnter your password phrase (or 'exit' to quit):");

        let mut stdin = io::stdin();
        let mut feedbacks: Vec<u8> = Vec::new();

        if let Some(idx) = password_manager.active_structure_idx {
            if idx < password_manager.saved_passwords.len() {
                let saved_password = &mut password_manager.saved_passwords[idx];

                println!("\nGenerated password:");

                loop {
                    let mut buffer = [0u8; 1];
                    match stdin.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(_) => {
                            let byte = buffer[0];

                            if byte == b'\n' || byte == b'\r' {
                                break;
                            }

                            if byte == b'e' && feedbacks.is_empty() {
                                // might be typing "exit", but continue processing normally
                            }

                            if let Some(ch) = char::from_u32(byte as u32) {
                                if !ch.is_control() {
                                    let keycode = ch as u32;

                                    let feedback_offset: u32 =
                                        feedbacks.iter().map(|&fb| fb as u32).sum();
                                    let modified_keycode = keycode.wrapping_add(feedback_offset);

                                    let mut navigation_sequence = vec![modified_keycode];
                                    for &fb in feedbacks.iter().rev() {
                                        navigation_sequence.push(fb as u32);
                                    }

                                    print!("\r                                                            \r");
                                    let _ = io::stdout().flush();

                                    saved_password.structure_system.reset_position();
                                    let mut output_sum = 0u64;

                                    for &input_code in &navigation_sequence {
                                        let output_chars =
                                            saved_password.structure_system.transform_char(
                                                input_code,
                                                saved_password.extra_chars_count,
                                            );

                                        for &code in &output_chars {
                                            if let Some(character) = char::from_u32(code) {
                                                print!("{}", character);
                                                let _ = io::stdout().flush();
                                                output_sum = output_sum.wrapping_add(code as u64);
                                            }
                                        }
                                    }

                                    let feedback = (output_sum % 256) as u8;
                                    feedbacks.push(feedback);
                                }
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }

                println!();
                feedbacks.clear();

                saved_password.structure_system.full_reset();
            }
        }
    }
}

fn zero_memory<T>(data: &mut [T]) {
    unsafe {
        std::ptr::write_bytes(data.as_mut_ptr(), 0, data.len());
    }
}

#[cfg(unix)]
fn setup_raw_mode() {
    use std::process::Command;
    let _ = Command::new("stty").args(&["raw", "-echo"]).status();
}

#[cfg(unix)]
fn restore_terminal() {
    use std::process::Command;
    let _ = Command::new("stty").args(&["cooked", "echo"]).status();
}

#[cfg(unix)]
fn enable_raw_mode() -> io::Result<()> {
    Command::new("stty")
        .args(&["-icanon", "-echo", "min", "0", "time", "0"])
        .stdin(Stdio::inherit())
        .status()?;
    Ok(())
}

#[cfg(unix)]
fn disable_raw_mode() -> io::Result<()> {
    Command::new("stty")
        .args(&["icanon", "echo"])
        .stdin(Stdio::inherit())
        .status()?;
    Ok(())
}

#[cfg(windows)]
fn setup_raw_mode() {
    let _ = enable_raw_mode();
}

#[cfg(windows)]
fn restore_terminal() {
    let _ = disable_raw_mode();
}

#[cfg(windows)]
fn enable_raw_mode() -> io::Result<()> {
    #[allow(non_snake_case)]
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;

        // Windows Console API constants
        const ENABLE_LINE_INPUT: u32 = 0x0002;
        const ENABLE_ECHO_INPUT: u32 = 0x0004;
        const ENABLE_PROCESSED_INPUT: u32 = 0x0001;
        const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;

        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;
                fn GetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, lpMode: *mut u32) -> i32;
                fn SetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, dwMode: u32) -> i32;
            }

            const STD_INPUT_HANDLE: u32 = 0xFFFFFFF6_u32;

            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if handle.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to get stdin handle",
                ));
            }

            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                return Err(io::Error::last_os_error());
            }

            mode &= !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
            mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;

            if SetConsoleMode(handle, mode) == 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
fn disable_raw_mode() -> io::Result<()> {
    #[allow(non_snake_case)]
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;

        const ENABLE_LINE_INPUT: u32 = 0x0002;
        const ENABLE_ECHO_INPUT: u32 = 0x0004;
        const ENABLE_PROCESSED_INPUT: u32 = 0x0001;

        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;
                fn GetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, lpMode: *mut u32) -> i32;
                fn SetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, dwMode: u32) -> i32;
            }

            const STD_INPUT_HANDLE: u32 = 0xFFFFFFF6_u32;

            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if handle.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to get stdin handle",
                ));
            }

            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                return Err(io::Error::last_os_error());
            }

            mode |= ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT;

            if SetConsoleMode(handle, mode) == 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

fn run_terminal_mode(args: &[String]) -> io::Result<()> {
    let mut account_name: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--account" && i + 1 < args.len() {
            account_name = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut password_manager = PasswordManager::new(false, None, None, false)?;

    let saved_password_idx = if let Some(name) = &account_name {
        password_manager
            .saved_passwords
            .iter()
            .position(|p| &p.name == name)
    } else {
        if let Some(idx) = password_manager.active_structure_idx {
            if idx < password_manager.saved_passwords.len() {
                Some(idx)
            } else {
                None
            }
        } else if !password_manager.saved_passwords.is_empty() {
            Some(0)
        } else {
            None
        }
    };

    let saved_password_idx = match saved_password_idx {
        Some(idx) => idx,
        None => {
            eprintln!("Error: No password configuration found");
            return Ok(());
        }
    };

    #[cfg(unix)]
    enable_raw_mode()?;

    let mut feedbacks: Vec<u8> = Vec::new();

    println!("Type your input (press Enter when done, Backspace to reset):");
    print!("\r");
    io::stdout().flush()?;

    loop {
        let mut buffer = [0u8; 1];
        let mut stdin = io::stdin();

        match stdin.read(&mut buffer) {
            Ok(0) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            Ok(_) => {
                let byte = buffer[0];

                match byte {
                    b'\n' | b'\r' => {
                        break;
                    }
                    127 | 8 => {
                        feedbacks.clear();

                        print!("\r                                                            \r");
                        io::stdout().flush()?;

                        password_manager.saved_passwords[saved_password_idx]
                            .structure_system
                            .full_reset();
                    }
                    3 => {
                        #[cfg(unix)]
                        disable_raw_mode()?;
                        println!();
                        return Ok(());
                    }
                    _ => {
                        if let Some(ch) = char::from_u32(byte as u32) {
                            if !ch.is_control() {
                                let mut keycode = ch as u32;

                                unsafe {
                                    if SESSION.initialized {
                                        keycode =
                                            keycode.wrapping_add(SESSION.active_counter as u32);
                                    }
                                }

                                let saved_password =
                                    &mut password_manager.saved_passwords[saved_password_idx];

                                // Ofset keycode by sum of all feedbacks
                                let feedback_offset: u32 =
                                    feedbacks.iter().map(|&fb| fb as u32).sum();
                                let modified_keycode = keycode.wrapping_add(feedback_offset);

                                let mut navigation_sequence = vec![modified_keycode];
                                for &fb in feedbacks.iter().rev() {
                                    navigation_sequence.push(fb as u32);
                                }

                                print!("\x1B[50A\r\x1B[0J");
                                io::stdout().flush()?;

                                saved_password.structure_system.reset_position();

                                let mut output_sum = 0u64;
                                for &input_code in &navigation_sequence {
                                    let output_chars =
                                        saved_password.structure_system.transform_char(
                                            input_code,
                                            saved_password.extra_chars_count,
                                        );

                                    for &code in &output_chars {
                                        if let Some(character) = char::from_u32(code) {
                                            print!("{}", character);
                                            io::stdout().flush()?;
                                            output_sum = output_sum.wrapping_add(code as u64);
                                        }
                                    }
                                }

                                let feedback = (output_sum % 256) as u8;
                                feedbacks.push(feedback);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                #[cfg(unix)]
                disable_raw_mode()?;
                return Err(e);
            }
        }
    }

    #[cfg(unix)]
    disable_raw_mode()?;

    println!();
    Ok(())
}

fn run_io_mode(args: &[String]) -> io::Result<()> {
    let mut account_name: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--account" && i + 1 < args.len() {
            account_name = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut password_manager = PasswordManager::new(false, None, None, false)?;

    let saved_password_idx = if let Some(name) = &account_name {
        password_manager
            .saved_passwords
            .iter()
            .position(|p| &p.name == name)
    } else {
        if let Some(idx) = password_manager.active_structure_idx {
            if idx < password_manager.saved_passwords.len() {
                Some(idx)
            } else {
                None
            }
        } else if !password_manager.saved_passwords.is_empty() {
            Some(0)
        } else {
            None
        }
    };

    let saved_password_idx = match saved_password_idx {
        Some(idx) => idx,
        None => {
            eprintln!("Error: No password configuration found");
            return Ok(());
        }
    };

    let mut stdin = io::stdin();
    let mut feedbacks: Vec<u8> = Vec::new();
    let mut input_chars: Vec<u32> = Vec::new();

    // VERY IMPORTANT
    // test sequences used for behavioral testing, in acending order, should be:
    // 1. 29213914 = zeroing trancient memory checking
    // 2. 999517725 = trancient dynamic memory checking
    // 3. 0843213126 = incremental link path testing
    // 4. 9332187235 = recursive end-to-end complexity checking
    // 5. 5001019899912 = unstable folding of the geometry testing
    // 6. 64221220322204 = additive. you use this one to test treversal & consisticy
    // 7. 110883422694685420 = multiple new geometry mutation pattern testing

    loop {
        let mut buffer = [0u8; 1];
        match stdin.read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let byte = buffer[0];

                if byte == b'\n' || byte == b'\r' {
                    break;
                }

                if let Some(ch) = char::from_u32(byte as u32) {
                    if !ch.is_control() {
                        input_chars.push(ch as u32);
                    }
                }
            }
            Err(e) => return Err(e),
        }
    }

    let saved_password = &mut password_manager.saved_passwords[saved_password_idx];

    for i in 0..input_chars.len() {
        let mut keycode = input_chars[i];

        unsafe {
            if SESSION.initialized {
                keycode = keycode.wrapping_add(SESSION.active_counter as u32);
            }
        }

        // Offset keycode by sum of all feedbacks so far
        let feedback_offset: u32 = feedbacks.iter().map(|&fb| fb as u32).sum();
        let modified_keycode = keycode.wrapping_add(feedback_offset);

        let mut navigation_sequence = vec![modified_keycode];
        for &fb in feedbacks.iter().rev() {
            navigation_sequence.push(fb as u32);
        }

        saved_password.structure_system.reset_position();
        let mut output_sum = 0u64;

        for &input_code in &navigation_sequence {
            let output_chars = saved_password
                .structure_system
                .transform_char(input_code, saved_password.extra_chars_count);

            for &code in &output_chars {
                output_sum = output_sum.wrapping_add(code as u64);

                if i == input_chars.len() - 1 {
                    if let Some(character) = char::from_u32(code) {
                        print!("{}", character);
                    }
                }
            }
        }

        let feedback = (output_sum % 256) as u8;
        feedbacks.push(feedback);
    }

    zero_memory(&mut input_chars);
    zero_memory(&mut feedbacks);

    io::stdout().flush()?;

    Ok(())
}

fn extract_json_string(message: &str, key: &str) -> String {
    let search = format!("\"{}\":\"", key);
    if let Some(start) = message.find(&search) {
        let start_idx = start + search.len();
        if let Some(end) = message[start_idx..].find('"') {
            return message[start_idx..start_idx + end].to_string();
        }
    }
    String::new()
}

fn extract_json_number(message: &str, key: &str) -> u64 {
    let search = format!("\"{}\":", key);
    if let Some(start) = message.find(&search) {
        let start_idx = start + search.len();
        if let Some(end) = message[start_idx..].find(&[',', '}'][..]) {
            if let Ok(val) = message[start_idx..start_idx + end].trim().parse() {
                return val;
            }
        }
    }
    0
}

fn run_json_io_mode(args: &[String]) -> io::Result<()> {
    let mut account_name: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--account" && i + 1 < args.len() {
            account_name = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut password_manager = PasswordManager::new(false, None, None, true)?;

    // Load domain table from binary on startup
    let exe_path = std::env::current_exe()?;
    if let Err(e) = DomainTable::load_from_binary(&exe_path) {
        eprintln!("Warning: Could not load domain table: {}", e);
    }

    let saved_password_idx = if let Some(name) = &account_name {
        password_manager
            .saved_passwords
            .iter()
            .position(|p| &p.name == name)
    } else {
        if let Some(idx) = password_manager.active_structure_idx {
            if idx < password_manager.saved_passwords.len() {
                Some(idx)
            } else {
                None
            }
        } else if !password_manager.saved_passwords.is_empty() {
            Some(0)
        } else {
            None
        }
    };

    let saved_password_idx = match saved_password_idx {
        Some(idx) => idx,
        None => {
            return Ok(());
        }
    };

    password_manager.saved_passwords[saved_password_idx]
        .structure_system
        .reset_position();

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut feedbacks: Vec<u8> = Vec::new();

    loop {
        let mut length_bytes = [0u8; 4];
        if stdin.read_exact(&mut length_bytes).is_err() {
            break;
        }

        let message_length = u32::from_le_bytes(length_bytes) as usize;

        let mut message_buffer = vec![0u8; message_length];
        if stdin.read_exact(&mut message_buffer).is_err() {
            break;
        }

        let message = match String::from_utf8(message_buffer) {
            Ok(s) => s,
            Err(_) => continue,
        };

        if message.contains("\"type\"") {
            if message.contains("\"INIT\"") {
                password_manager.saved_passwords[saved_password_idx]
                    .structure_system
                    .full_reset();

                feedbacks.clear();

                let response = "{\"status\":\"ready\"}";
                let response_length = response.len() as u32;
                stdout.write_all(&response_length.to_le_bytes())?;
                stdout.write_all(response.as_bytes())?;
                stdout.flush()?;
                continue;
            } else if message.contains("\"RESET\"") {
                password_manager.saved_passwords[saved_password_idx]
                    .structure_system
                    .full_reset();

                feedbacks.clear();

                // Note: RESET only clears geometry and feedbacks, does NOT exit preview mode
                // Preview mode state is preserved so user can retype with same counter

                unsafe {
                    if SESSION.initialized {
                        if let Some(ref domain_hash) = SESSION.active_domain_hash {
                            let structure = &mut password_manager.saved_passwords
                                [saved_password_idx]
                                .structure_system;

                            for i in 0..8 {
                                let hash_byte = domain_hash[i] as u32;
                                let _ = structure.transform_char(hash_byte, 0);
                            }

                            let counter_u32 = SESSION.active_counter as u32;
                            let _ = structure.transform_char(counter_u32, 0);
                            let _ = structure.transform_char(counter_u32.wrapping_mul(7), 0);
                            let _ = structure.transform_char(counter_u32.wrapping_add(13), 0);
                        }
                    }
                }

                let response = "{\"status\":\"reset\"}";
                let response_length = response.len() as u32;
                stdout.write_all(&response_length.to_le_bytes())?;
                stdout.write_all(response.as_bytes())?;
                stdout.flush()?;
                continue;
            } else if message.contains("\"FINALIZE\"") {
                feedbacks.clear();
                unsafe {
                    SESSION.initialized = false;
                    SESSION.active_domain_hash = None;
                }
                break;
            } else if message.contains("\"GET_COUNTER\"") {
                let domain = extract_json_string(&message, "domain");

                let response = if !domain.is_empty() {
                    let structure =
                        &mut password_manager.saved_passwords[saved_password_idx].structure_system;
                    match DomainTable::get_counter(&domain, structure) {
                        Some(counter) => format!("{{\"counter\":{}}}", counter),
                        None => "{\"counter\":null}".to_string(),
                    }
                } else {
                    "{\"error\":\"Missing domain\"}".to_string()
                };

                let response_length = response.len() as u32;
                stdout.write_all(&response_length.to_le_bytes())?;
                stdout.write_all(response.as_bytes())?;
                stdout.flush()?;
                continue;
            } else if message.contains("\"ACTIVATE\"") && !message.contains("\"ACTIVATE_PREVIEW\"")
            {
                let domain = extract_json_string(&message, "domain");

                if !domain.is_empty() {
                    let structure =
                        &mut password_manager.saved_passwords[saved_password_idx].structure_system;

                    let counter = match DomainTable::get_counter(&domain, structure) {
                        Some(c) => c,
                        None => {
                            if let Err(e) = DomainTable::set_counter(&domain, 0, structure) {
                                let response = format!("{{\"error\":\"{}\"}}", e);
                                let response_length = response.len() as u32;
                                stdout.write_all(&response_length.to_le_bytes())?;
                                stdout.write_all(response.as_bytes())?;
                                stdout.flush()?;
                                continue;
                            }
                            if let Err(e) = DomainTable::save_to_binary(&exe_path) {
                                eprintln!("Warning: Could not save domain table: {}", e);
                            }
                            0
                        }
                    };

                    let (max_length, char_types) =
                        DomainTable::get_rules(&domain, structure).unwrap_or((0, 127)); // Default: unlimited length, all types enabled

                    // Hash domain and store in session
                    let domain_hash = structure.hash_domain(&domain);

                    unsafe {
                        SESSION.active_domain_hash = Some(domain_hash);
                        SESSION.saved_counter = counter;
                        SESSION.active_counter = counter;
                        SESSION.is_preview_mode = false;
                        SESSION.initialized = true;
                    }

                    structure.full_reset();
                    feedbacks.clear();

                    // Ghost navigation: Navigate through geometry using domain hash + counter
                    // This ensures each domain+counter combination starts from a unique position
                    // WITHOUT producing any output characters

                    for i in 0..8 {
                        let hash_byte = domain_hash[i] as u32;
                        let _ = structure.transform_char(hash_byte, 0);
                    }

                    // Use counter as both direct value and derived values for more entropy
                    let counter_u32 = counter as u32;
                    let _ = structure.transform_char(counter_u32, 0);
                    let _ = structure.transform_char(counter_u32.wrapping_mul(7), 0);
                    let _ = structure.transform_char(counter_u32.wrapping_add(13), 0);

                    // Now we're at a unique position in 7D space for this domain+counter
                    // Subsequent user input will generate from this position

                    let response = format!("{{\"saved_counter\":{},\"active_counter\":{},\"max_length\":{},\"char_types\":{},\"status\":\"ready\"}}", counter, counter, max_length, char_types);
                    let response_length = response.len() as u32;
                    stdout.write_all(&response_length.to_le_bytes())?;
                    stdout.write_all(response.as_bytes())?;
                    stdout.flush()?;
                } else {
                    let response = "{\"error\":\"Missing domain\"}";
                    let response_length = response.len() as u32;
                    stdout.write_all(&response_length.to_le_bytes())?;
                    stdout.write_all(response.as_bytes())?;
                    stdout.flush()?;
                }
                continue;
            } else if message.contains("\"ACTIVATE_PREVIEW\"") {
                let domain = extract_json_string(&message, "domain");

                if !domain.is_empty() {
                    let structure =
                        &mut password_manager.saved_passwords[saved_password_idx].structure_system;

                    let saved_counter = DomainTable::get_counter(&domain, structure).unwrap_or(0);
                    let preview_counter = saved_counter.saturating_add(1);

                    let (max_length, char_types) =
                        DomainTable::get_rules(&domain, structure).unwrap_or((0, 127));

                    let domain_hash = structure.hash_domain(&domain);

                    unsafe {
                        SESSION.active_domain_hash = Some(domain_hash);
                        SESSION.saved_counter = saved_counter;
                        SESSION.active_counter = preview_counter;
                        SESSION.is_preview_mode = true;
                        SESSION.initialized = true;
                    }

                    structure.full_reset();
                    feedbacks.clear();

                    for i in 0..8 {
                        let hash_byte = domain_hash[i] as u32;
                        let _ = structure.transform_char(hash_byte, 0);
                    }

                    let counter_u32 = preview_counter as u32;
                    let _ = structure.transform_char(counter_u32, 0);
                    let _ = structure.transform_char(counter_u32.wrapping_mul(7), 0);
                    let _ = structure.transform_char(counter_u32.wrapping_add(13), 0);

                    let response = format!("{{\"saved_counter\":{},\"active_counter\":{},\"max_length\":{},\"char_types\":{},\"status\":\"preview\"}}", saved_counter, preview_counter, max_length, char_types);
                    let response_length = response.len() as u32;
                    stdout.write_all(&response_length.to_le_bytes())?;
                    stdout.write_all(response.as_bytes())?;
                    stdout.flush()?;
                } else {
                    let response = "{\"error\":\"Missing domain\"}";
                    let response_length = response.len() as u32;
                    stdout.write_all(&response_length.to_le_bytes())?;
                    stdout.write_all(response.as_bytes())?;
                    stdout.flush()?;
                }
                continue;
            } else if message.contains("\"SET_COUNTER\"") {
                let domain = extract_json_string(&message, "domain");
                let counter = extract_json_number(&message, "counter");

                if !domain.is_empty() {
                    let structure =
                        &mut password_manager.saved_passwords[saved_password_idx].structure_system;

                    match DomainTable::set_counter(&domain, counter as u16, structure) {
                        Ok(()) => {
                            if let Err(e) = DomainTable::save_to_binary(&exe_path) {
                                eprintln!("Warning: Could not save domain table: {}", e);
                            }

                            let domain_hash = structure.hash_domain(&domain);
                            unsafe {
                                let session = &*std::ptr::addr_of!(SESSION);
                                if session.active_domain_hash.as_ref() == Some(&domain_hash) {
                                    let session = &mut *std::ptr::addr_of_mut!(SESSION);
                                    session.saved_counter = counter as u16;
                                    session.active_counter = counter as u16;
                                    session.is_preview_mode = false;
                                    structure.full_reset();
                                    feedbacks.clear();
                                }
                            }

                            let response = "{\"status\":\"success\"}";
                            let response_length = response.len() as u32;
                            stdout.write_all(&response_length.to_le_bytes())?;
                            stdout.write_all(response.as_bytes())?;
                            stdout.flush()?;
                        }
                        Err(e) => {
                            let response = format!("{{\"error\":\"{}\"}}", e);
                            let response_length = response.len() as u32;
                            stdout.write_all(&response_length.to_le_bytes())?;
                            stdout.write_all(response.as_bytes())?;
                            stdout.flush()?;
                        }
                    }
                } else {
                    let response = "{\"error\":\"Missing domain\"}";
                    let response_length = response.len() as u32;
                    stdout.write_all(&response_length.to_le_bytes())?;
                    stdout.write_all(response.as_bytes())?;
                    stdout.flush()?;
                }
                continue;
            } else if message.contains("\"SET_RULES\"") {
                let domain = extract_json_string(&message, "domain");
                let max_length = extract_json_number(&message, "max_length") as u16;
                let char_types = extract_json_number(&message, "char_types") as u8;

                if !domain.is_empty() {
                    let structure =
                        &mut password_manager.saved_passwords[saved_password_idx].structure_system;

                    match DomainTable::set_rules(&domain, max_length, char_types, structure) {
                        Ok(()) => {
                            if let Err(e) = DomainTable::save_to_binary(&exe_path) {
                                eprintln!("Warning: Could not save domain table: {}", e);
                            }

                            let response = "{\"status\":\"success\"}";
                            let response_length = response.len() as u32;
                            stdout.write_all(&response_length.to_le_bytes())?;
                            stdout.write_all(response.as_bytes())?;
                            stdout.flush()?;
                        }
                        Err(e) => {
                            let response = format!("{{\"error\":\"{}\"}}", e);
                            let response_length = response.len() as u32;
                            stdout.write_all(&response_length.to_le_bytes())?;
                            stdout.write_all(response.as_bytes())?;
                            stdout.flush()?;
                        }
                    }
                } else {
                    let response = "{\"error\":\"Missing domain\"}";
                    let response_length = response.len() as u32;
                    stdout.write_all(&response_length.to_le_bytes())?;
                    stdout.write_all(response.as_bytes())?;
                    stdout.flush()?;
                }
                continue;
            } else if message.contains("\"COMMIT_INCREMENT\"") {
                let domain = extract_json_string(&message, "domain");

                if !domain.is_empty() {
                    unsafe {
                        if SESSION.is_preview_mode {
                            let structure = &mut password_manager.saved_passwords
                                [saved_password_idx]
                                .structure_system;

                            if let Err(e) =
                                DomainTable::set_counter(&domain, SESSION.active_counter, structure)
                            {
                                let response = format!("{{\"error\":\"{}\"}}", e);
                                let response_length = response.len() as u32;
                                stdout.write_all(&response_length.to_le_bytes())?;
                                stdout.write_all(response.as_bytes())?;
                                stdout.flush()?;
                                continue;
                            }

                            if let Err(e) = DomainTable::save_to_binary(&exe_path) {
                                eprintln!("Warning: Could not save domain table: {}", e);
                            }

                            let session = &mut *std::ptr::addr_of_mut!(SESSION);
                            let active = session.active_counter;
                            session.saved_counter = active;
                            session.is_preview_mode = false;

                            let response =
                                format!("{{\"counter\":{},\"status\":\"committed\"}}", active);
                            let response_length = response.len() as u32;
                            stdout.write_all(&response_length.to_le_bytes())?;
                            stdout.write_all(response.as_bytes())?;
                            stdout.flush()?;
                        } else {
                            let response = "{\"error\":\"Not in preview mode\"}";
                            let response_length = response.len() as u32;
                            stdout.write_all(&response_length.to_le_bytes())?;
                            stdout.write_all(response.as_bytes())?;
                            stdout.flush()?;
                        }
                    }
                } else {
                    let response = "{\"error\":\"Missing domain\"}";
                    let response_length = response.len() as u32;
                    stdout.write_all(&response_length.to_le_bytes())?;
                    stdout.write_all(response.as_bytes())?;
                    stdout.flush()?;
                }
                continue;
            } else if message.contains("\"CANCEL_PREVIEW\"") {
                unsafe {
                    let session = &mut *std::ptr::addr_of_mut!(SESSION);
                    if session.is_preview_mode {
                        let saved = session.saved_counter;
                        session.active_counter = saved;
                        session.is_preview_mode = false;

                        password_manager.saved_passwords[saved_password_idx]
                            .structure_system
                            .full_reset();
                        feedbacks.clear();

                        if let Some(ref domain_hash) = SESSION.active_domain_hash {
                            let structure = &mut password_manager.saved_passwords
                                [saved_password_idx]
                                .structure_system;

                            for i in 0..8 {
                                let hash_byte = domain_hash[i] as u32;
                                let _ = structure.transform_char(hash_byte, 0);
                            }

                            let counter_u32 = saved as u32;
                            let _ = structure.transform_char(counter_u32, 0);
                            let _ = structure.transform_char(counter_u32.wrapping_mul(7), 0);
                            let _ = structure.transform_char(counter_u32.wrapping_add(13), 0);
                        }

                        let response =
                            format!("{{\"counter\":{},\"status\":\"cancelled\"}}", saved);
                        let response_length = response.len() as u32;
                        stdout.write_all(&response_length.to_le_bytes())?;
                        stdout.write_all(response.as_bytes())?;
                        stdout.flush()?;
                    } else {
                        let response = "{\"error\":\"Not in preview mode\"}";
                        let response_length = response.len() as u32;
                        stdout.write_all(&response_length.to_le_bytes())?;
                        stdout.write_all(response.as_bytes())?;
                        stdout.flush()?;
                    }
                }
                continue;
            }
        }

        let keycode = extract_json_number(&message, "charCode") as u32;

        if keycode > 0 {
            let saved_password = &mut password_manager.saved_passwords[saved_password_idx];

            // Offset keycode by sum of all feedbacks
            let feedback_offset: u32 = feedbacks.iter().map(|&fb| fb as u32).sum();
            let modified_keycode = keycode.wrapping_add(feedback_offset);

            let mut navigation_sequence = vec![modified_keycode];
            for &fb in feedbacks.iter().rev() {
                navigation_sequence.push(fb as u32);
            }

            saved_password.structure_system.reset_position();
            let mut output_sum = 0u64;
            let mut output_codes = Vec::new();

            for &input_code in &navigation_sequence {
                let codes = saved_password
                    .structure_system
                    .transform_char(input_code, saved_password.extra_chars_count);

                for &code in &codes {
                    output_sum = output_sum.wrapping_add(code as u64);
                    output_codes.push(code);
                }
            }

            let feedback = (output_sum % 256) as u8;
            feedbacks.push(feedback);

            let output_chars: String = output_codes
                .iter()
                .filter_map(|&code| char::from_u32(code))
                .collect();

            let mut response = String::from("{\"output\":\"");

            for ch in output_chars.chars() {
                match ch {
                    '"' => response.push_str("\\\""),
                    '\\' => response.push_str("\\\\"),
                    '\n' => response.push_str("\\n"),
                    '\r' => response.push_str("\\r"),
                    '\t' => response.push_str("\\t"),
                    _ => response.push(ch),
                }
            }
            response.push_str("\"}");

            let response_length = response.len() as u32;
            stdout.write_all(&response_length.to_le_bytes())?;
            stdout.write_all(response.as_bytes())?;
            stdout.flush()?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--list-domains" {
        let exe_path = std::env::current_exe()?;
        DomainTable::load_from_binary(&exe_path)?;

        let mut count = 0;
        unsafe {
            println!("Registered domains (hashes only - domain names cannot be reversed):\n");
            let table = &*std::ptr::addr_of!(DOMAIN_TABLE);
            for (i, slot) in table.slots.iter().enumerate() {
                if !slot.is_empty() {
                    let hex: String = slot.domain_hash[..16]
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect();
                    println!("Slot {}: {}... → v{}", i, hex, slot.counter);
                    count += 1;
                }
            }
        }
        println!("\nTotal: {} domains registered", count);
        return Ok(());
    } else if args.len() > 2 && args[1] == "--get-counter" {
        let domain = &args[2];
        let exe_path = std::env::current_exe()?;
        DomainTable::load_from_binary(&exe_path)?;

        let mut password_manager = PasswordManager::new(false, None, None, true)?;
        if password_manager.saved_passwords.is_empty() {
            eprintln!("Error: No geometry found. Please create one first.");
            return Ok(());
        }

        let structure = &mut password_manager.saved_passwords[0].structure_system;

        match DomainTable::get_counter(domain, structure) {
            Some(c) => println!("{}: v{}", domain, c),
            None => println!("{}: not found", domain),
        }
        return Ok(());
    } else if args.len() > 3 && args[1] == "--set-counter" {
        let domain = &args[2];
        let counter: u16 = args[3]
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Counter must be 0-65535"))?;

        let exe_path = std::env::current_exe()?;
        DomainTable::load_from_binary(&exe_path)?;

        let mut password_manager = PasswordManager::new(false, None, None, true)?;
        if password_manager.saved_passwords.is_empty() {
            eprintln!("Error: No geometry found. Please create one first.");
            return Ok(());
        }

        let structure = &mut password_manager.saved_passwords[0].structure_system;

        DomainTable::set_counter(domain, counter, structure)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        DomainTable::save_to_binary(&exe_path)?;

        println!("Set {} to v{}", domain, counter);
        return Ok(());
    } else if args.len() > 2 && args[1] == "--increment-counter" {
        let domain = &args[2];
        let exe_path = std::env::current_exe()?;
        DomainTable::load_from_binary(&exe_path)?;

        let mut password_manager = PasswordManager::new(false, None, None, true)?;
        if password_manager.saved_passwords.is_empty() {
            eprintln!("Error: No geometry found. Please create one first.");
            return Ok(());
        }

        let structure = &mut password_manager.saved_passwords[0].structure_system;

        let new_counter = DomainTable::increment_counter(domain, structure)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        DomainTable::save_to_binary(&exe_path)?;

        println!("{}: v{}", domain, new_counter);
        return Ok(());
    }

    if let Some(domain_counter_pos) = args.iter().position(|arg| arg == "--use-domain-counter") {
        if args.len() > domain_counter_pos + 1 {
            let domain = &args[domain_counter_pos + 1];
            let exe_path = std::env::current_exe()?;
            DomainTable::load_from_binary(&exe_path)?;

            let mut password_manager = PasswordManager::new(false, None, None, true)?;
            if password_manager.saved_passwords.is_empty() {
                eprintln!("Error: No geometry found. Please create one first.");
                return Ok(());
            }

            let structure = &mut password_manager.saved_passwords[0].structure_system;
            let counter = DomainTable::get_counter(domain, structure).unwrap_or(0);
            let domain_hash = structure.hash_domain(domain);

            unsafe {
                let session = &mut *std::ptr::addr_of_mut!(SESSION);
                session.active_domain_hash = Some(domain_hash);
                session.saved_counter = counter;
                session.active_counter = counter;
                session.is_preview_mode = false;
                session.initialized = true;
            }

            eprintln!("Using domain counter for '{}': v{}", domain, counter);
        } else {
            eprintln!("Error: --use-domain-counter requires a domain name");
            return Ok(());
        }
    }

    let auto_exit = args.contains(&"--auto-exit".to_string());

    if args.len() > 1 && args[1] == "--child-process" {
        run_child_process(auto_exit)?;
    } else if args.len() > 1 && args[1] == "--term" {
        run_terminal_mode(&args)?;
    } else if args.len() > 1 && args[1] == "--io" {
        run_io_mode(&args)?;
    } else if args.len() > 1 && args[1] == "--json-io" {
        run_json_io_mode(&args)?;
    } else if is_native_messaging_mode() {
        // auto-detect browser native messaging (stdin is not a TTY)
        run_json_io_mode(&args)?;
    } else {
        run_parent_process(auto_exit)?;
    }

    Ok(())
}

/// detects if we're being called by a browser for native messaging
#[cfg(unix)]
fn is_native_messaging_mode() -> bool {
    use std::os::unix::fs::FileTypeExt;

    if let Ok(metadata) = fs::metadata("/dev/stdin") {
        let file_type = metadata.file_type();
        file_type.is_fifo() || !file_type.is_char_device()
    } else {
        false
    }
}

/// Windows version: Check if stdin is a pipe (not a console)
#[cfg(windows)]
fn is_native_messaging_mode() -> bool {
    unsafe {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;
            fn GetFileType(hFile: *mut std::ffi::c_void) -> u32;
        }

        const STD_INPUT_HANDLE: u32 = 0xFFFFFFF6_u32;
        const FILE_TYPE_CHAR: u32 = 0x0002;
        const FILE_TYPE_PIPE: u32 = 0x0003;

        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle.is_null() {
            return false;
        }

        let file_type = GetFileType(handle);

        file_type == FILE_TYPE_PIPE
    }
}
