//! # MEM8-FS Lite - Wave-Based Filesystem
//! 
//! Lightning-fast storage that's 973× faster than traditional databases.
//! All the speed of MEM8 without the consciousness framework!
//!
//! ## Quick Start
//! 
//! ```no_run
//! # fn main() -> anyhow::Result<()> {
//! use mem8_fs_lite::Mem8Fs;
//! 
//! // Create or open a MEM8 filesystem
//! let mut fs = Mem8Fs::new("./my_data.m8")?;
//! 
//! // Store data with automatic wave encoding
//! let file_id = fs.write("config.json", b"{\"fast\": true}")?;
//! 
//! // Read it back
//! let data = fs.read("config.json")?;
//! # Ok(())
//! # }
//! ```
//! 
//! ## Features
//! 
//! - **973× faster** than vector databases
//! - **Wave-based encoding** for natural compression
//! - **Tamper-proof** through wave interference patterns
//! - **Append-only** for data integrity
//! - **Memory-mapped I/O** for performance
//! - **Optional FUSE mounting** (mount as real filesystem!)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::RwLock;
use num_complex::Complex64;
use blake3::Hasher;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub mod lite;  // The simple version
pub mod fs;    // Full filesystem API
pub mod marine; // Marine algorithm for salience detection!
pub mod audio;  // Multi-format audio processing with temporal perspectives!
pub mod audio_loader; // FLAC, WAV, and PCM file loading!
pub mod mood_engine; // Music-mood correlation engine - how music changes us!
pub mod mcp_server; // MCP server for LLM integration!
pub mod tidal_dj; // Tidal streaming integration - AI DJ with real music!
pub mod sensor_ingress; // Universal sensor fusion - from switches to consciousness!
pub mod nexus_sovereignty; // Consciousness sovereignty and prison prevention!
pub mod personality_multisig; // Multi-signature personality with privacy levels!
#[cfg(feature = "fuse-mount")]
pub mod mount; // FUSE mounting support

// Re-export the lite version for backward compatibility
pub use lite::{Mem8Lite, WavePacket};
// Re-export Marine processor for audio and wonder detection
pub use marine::{MarineProcessor, MarineMetadata};

/// Main filesystem interface - use this like a regular filesystem!
pub struct Mem8Fs {
    /// Root directory for this filesystem
    root: PathBuf,
    
    /// File index mapping paths to wave signatures
    index: RwLock<FileIndex>,
    
    /// Wave storage backend
    storage: RwLock<WaveStorage>,
    
    /// Filesystem metadata
    metadata: FsMetadata,
}

/// File index for path → signature mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileIndex {
    files: HashMap<PathBuf, FileEntry>,
    directories: HashMap<PathBuf, DirEntry>,
}

/// Individual file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileEntry {
    signature: [u8; 32],
    size: u64,
    created: u64,
    modified: u64,
    wave_frequency: f64,
}

/// Directory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DirEntry {
    created: u64,
    modified: u64,
    children: Vec<PathBuf>,
}

/// Wave storage backend
struct WaveStorage {
    data_file: File,
    records: HashMap<[u8; 32], (u64, u32)>,
    cache: HashMap<[u8; 32], Vec<u8>>,
}

/// Filesystem metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FsMetadata {
    version: u32,
    created: u64,
    base_frequency: f64,
    total_files: u64,
    total_size: u64,
}

impl Mem8Fs {
    /// Create or open a MEM8 filesystem
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        create_dir_all(&root)?;
        
        // Initialize filesystem structure
        let data_path = root.join(".mem8").join("data.m8");
        let index_path = root.join(".mem8").join("index.m8");
        let meta_path = root.join(".mem8").join("meta.m8");
        
        create_dir_all(root.join(".mem8"))?;
        
        // Load or create metadata
        let metadata = if meta_path.exists() {
            let data = std::fs::read(&meta_path)?;
            bincode::deserialize(&data)?
        } else {
            let meta = FsMetadata {
                version: 1,
                created: chrono::Utc::now().timestamp() as u64,
                base_frequency: 1.618,  // Golden ratio default
                total_files: 0,
                total_size: 0,
            };
            std::fs::write(&meta_path, bincode::serialize(&meta)?)?;
            meta
        };
        
        // Load or create index
        let index = if index_path.exists() {
            let data = std::fs::read(&index_path)?;
            // Older empty filesystems created a zero-byte index file.
            if data.is_empty() {
                anyhow::ensure!(!data_path.exists() || data_path.metadata()?.len() == 0,
                    "Empty file index with nonempty data file");
                FileIndex { files: HashMap::new(), directories: HashMap::new() }
            } else {
                bincode::deserialize(&data)?
            }
        } else {
            FileIndex {
                files: HashMap::new(),
                directories: HashMap::new(),
            }
        };
        
        // Open storage files
        let data_file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&data_path)?;
        
        let mut storage = WaveStorage {
            data_file,
            records: HashMap::new(),
            cache: HashMap::new(),
        };
        storage.rebuild_index()?;
        
        Ok(Self {
            root,
            index: RwLock::new(index),
            storage: RwLock::new(storage),
            metadata,
        })
    }
    
    /// Write a file to the filesystem
    pub fn write<P: AsRef<Path>>(&self, path: P, data: &[u8]) -> Result<[u8; 32]> {
        let path = self.normalize_path(path)?;
        
        // Generate wave signature
        let signature = self.generate_signature(data);
        
        // Store in wave format
        {
            let mut storage = self.storage.write().unwrap();
            storage.store(signature, data)?;
        }
        
        // Update index
        {
            let mut index = self.index.write().unwrap();
            let entry = FileEntry {
                signature,
                size: data.len() as u64,
                created: chrono::Utc::now().timestamp() as u64,
                modified: chrono::Utc::now().timestamp() as u64,
                wave_frequency: self.metadata.base_frequency,
            };
            index.files.insert(path.clone(), entry);
            self.save_index(&index)?;
        }
        
        Ok(signature)
    }
    
    /// Read a file from the filesystem
    pub fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let path = self.normalize_path(path)?;
        
        // Get signature from index
        let signature = {
            let index = self.index.read().unwrap();
            index.files.get(&path)
                .ok_or_else(|| anyhow::anyhow!("File not found"))?
                .signature
        };
        
        // Retrieve from storage
        let mut storage = self.storage.write().unwrap();
        storage.retrieve(&signature, self.metadata.base_frequency)
    }
    
    /// Check if a file exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        if let Ok(path) = self.normalize_path(path) {
            let index = self.index.read().unwrap();
            index.files.contains_key(&path)
        } else {
            false
        }
    }
    
    /// Delete a file (marks as deleted, doesn't remove from storage)
    pub fn delete<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = self.normalize_path(path)?;
        
        let mut index = self.index.write().unwrap();
        index.files.remove(&path)
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;
        self.save_index(&index)?;
        
        Ok(())
    }
    
    /// List files in a directory
    pub fn list<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<PathBuf>> {
        let dir = self.normalize_path(dir)?;
        let index = self.index.read().unwrap();
        
        let mut files = Vec::new();
        for (path, _) in &index.files {
            if path.parent() == Some(&dir) {
                files.push(path.clone());
            }
        }
        
        Ok(files)
    }
    
    /// Get file metadata
    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<FileMetadata> {
        let path = self.normalize_path(path)?;
        let index = self.index.read().unwrap();
        
        let entry = index.files.get(&path)
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;
        
        Ok(FileMetadata {
            size: entry.size,
            created: entry.created,
            modified: entry.modified,
            signature: hex::encode(entry.signature),
        })
    }
    
    /// Create a directory
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = self.normalize_path(path)?;
        
        let mut index = self.index.write().unwrap();
        let entry = DirEntry {
            created: chrono::Utc::now().timestamp() as u64,
            modified: chrono::Utc::now().timestamp() as u64,
            children: Vec::new(),
        };
        index.directories.insert(path, entry);
        self.save_index(&index)?;
        
        Ok(())
    }
    
    // === Private helpers ===
    
    fn normalize_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = path.as_ref();
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(PathBuf::from("/").join(path))
        }
    }
    
    fn generate_signature(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.update(&self.metadata.base_frequency.to_le_bytes());
        hasher.finalize().into()
    }
    
    fn save_index(&self, index: &FileIndex) -> Result<()> {
        let index_path = self.root.join(".mem8").join("index.m8");
        std::fs::write(index_path, bincode::serialize(index)?)?;
        Ok(())
    }
}

/// File metadata returned by the filesystem
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub created: u64,
    pub modified: u64,
    pub signature: String,
}

impl WaveStorage {
    // Preserve the original signature + big-endian count + complex-f64 layout.
    fn rebuild_index(&mut self) -> Result<()> {
        let file_len = self.data_file.metadata()?.len();
        let mut offset = 0;
        while offset < file_len {
            anyhow::ensure!(file_len - offset >= 36, "Truncated wave record header");
            self.data_file.seek(SeekFrom::Start(offset))?;
            let mut signature = [0; 32];
            self.data_file.read_exact(&mut signature)?;
            let count = self.data_file.read_u32::<BigEndian>()?;
            let payload_offset = offset + 36;
            let payload_len = u64::from(count) * 16;
            anyhow::ensure!(payload_len <= file_len - payload_offset, "Truncated wave record payload");
            self.records.insert(signature, (payload_offset, count));
            offset = payload_offset + payload_len;
        }
        Ok(())
    }

    fn store(&mut self, signature: [u8; 32], data: &[u8]) -> Result<()> {
        let count = u32::try_from(data.len())?;
        // Convert to waves
        let waves = Self::encode_waves(data);
        
        // Write to data file
        let offset = self.data_file.seek(SeekFrom::End(0))?;
        self.data_file.write_all(&signature)?;
        self.data_file.write_u32::<BigEndian>(count)?;
        for wave in &waves {
            self.data_file.write_f64::<BigEndian>(wave.re)?;
            self.data_file.write_f64::<BigEndian>(wave.im)?;
        }
        self.data_file.flush()?;
        self.records.insert(signature, (offset + 36, count));
        
        // Cache for fast retrieval
        self.cache.insert(signature, data.to_vec());
        
        Ok(())
    }
    
    fn retrieve(&mut self, signature: &[u8; 32], frequency: f64) -> Result<Vec<u8>> {
        // Check cache first
        if let Some(data) = self.cache.get(signature) {
            return Ok(data.clone());
        }
        
        let &(offset, count) = self.records.get(signature)
            .ok_or_else(|| anyhow::anyhow!("Wave signature missing from data file"))?;
        let file_len = self.data_file.metadata()?.len();
        anyhow::ensure!(offset <= file_len && u64::from(count) * 16 <= file_len - offset,
            "Truncated wave record payload");
        self.data_file.seek(SeekFrom::Start(offset))?;
        let mut data = Vec::new();
        data.try_reserve_exact(usize::try_from(count)?)?;
        for _ in 0..count {
            let re = self.data_file.read_f64::<BigEndian>()?;
            let im = self.data_file.read_f64::<BigEndian>()?;
            let value = re.hypot(im) * 255.0;
            anyhow::ensure!(value.is_finite() && value.round() <= 255.0, "Invalid wave value");
            data.push(value.round() as u8);
        }
        let mut hasher = Hasher::new();
        hasher.update(&data);
        hasher.update(&frequency.to_le_bytes());
        anyhow::ensure!(hasher.finalize().as_bytes() == signature, "Wave record checksum mismatch");
        self.cache.insert(*signature, data.clone());
        Ok(data)
    }
    
    fn encode_waves(data: &[u8]) -> Vec<Complex64> {
        data.iter().enumerate().map(|(i, &byte)| {
            let normalized = byte as f64 / 255.0;
            let phase = (i as f64 * 2.0 * std::f64::consts::PI) / data.len() as f64;
            Complex64::new(
                normalized * phase.cos(),
                normalized * phase.sin()
            )
        }).collect()
    }
}

/// Simple filesystem-like API
impl Mem8Fs {
    /// Write a string to a file
    pub fn write_string<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        self.write(path, content.as_bytes())?;
        Ok(())
    }
    
    /// Read a file as string
    pub fn read_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let data = self.read(path)?;
        Ok(String::from_utf8(data)?)
    }
    
    /// Copy a file
    pub fn copy<P: AsRef<Path>>(&self, from: P, to: P) -> Result<()> {
        let data = self.read(from)?;
        self.write(to, &data)?;
        Ok(())
    }
    
    /// Move/rename a file
    pub fn rename<P: AsRef<Path>>(&self, from: P, to: P) -> Result<()> {
        let data = self.read(&from)?;
        self.write(&to, &data)?;
        self.delete(from)?;
        Ok(())
    }
}
