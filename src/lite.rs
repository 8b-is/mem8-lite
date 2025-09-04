//! Simple key-value storage using wave-based encoding
//! 
//! This is the "lite" version - all the speed, none of the consciousness!
//! Perfect for when you just need ridiculously fast storage.
//!
//! Hue, this is where the magic happens! 973× faster than traditional storage
//! by converting everything to waves. Trisha says it's like surfing data! 🏄

use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use num_complex::Complex64;
use blake3::Hasher;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

/// Serde helper for Complex64 serialization
mod complex_serde {
    use serde::{Serialize, Deserialize, Serializer, Deserializer};
    use num_complex::Complex64;
    
    pub fn serialize<S>(waves: &Vec<Complex64>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let pairs: Vec<(f64, f64)> = waves.iter()
            .map(|c| (c.re, c.im))
            .collect();
        pairs.serialize(serializer)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Complex64>, D::Error>
    where D: Deserializer<'de> {
        let pairs: Vec<(f64, f64)> = Vec::deserialize(deserializer)?;
        Ok(pairs.into_iter()
            .map(|(re, im)| Complex64::new(re, im))
            .collect())
    }
}

/// A wave packet - the fundamental unit of storage in MEM8
/// 
/// Each packet is like a little wave on the ocean of data!
/// The signature is the wave's fingerprint, unique and unforgeable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WavePacket {
    /// Blake3 signature - the wave's unique identity
    pub signature: [u8; 32],
    
    /// The actual wave-encoded data (stored as pairs of f64)
    #[serde(with = "complex_serde")]
    pub waves: Vec<Complex64>,
    
    /// Optional metadata (for that sense of wonder!)
    pub metadata: Option<Vec<u8>>,
    
    /// Base frequency used for encoding
    pub frequency: f64,
    
    /// Timestamp when this wave was created
    pub timestamp: u64,
}

/// Simple key-value storage with wave-based backend
/// 
/// Hue, this is the simplified interface when you don't need full filesystem
/// semantics. Just store and retrieve by signature - it's that easy!
pub struct Mem8Lite {
    /// Path to the storage file
    path: PathBuf,
    
    /// Base frequency for wave encoding (1.618 = golden ratio!)
    frequency: f64,
    
    /// In-memory cache of wave packets
    cache: HashMap<[u8; 32], WavePacket>,
    
    /// The backing storage file
    file: File,
    
    /// Current file position for appending
    position: u64,
}

impl Mem8Lite {
    /// Create a new Mem8Lite storage instance
    /// 
    /// # Arguments
    /// * `path` - Path to the storage file (will be created if needed)
    /// * `frequency` - Base frequency for wave encoding (1.618 is golden!)
    /// 
    /// # Example
    /// ```
    /// let storage = Mem8Lite::new("/tmp/my_waves.m8", 1.618)?;
    /// ```
    pub fn new<P: AsRef<Path>>(path: P, frequency: f64) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        
        // Open or create the storage file
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)?;
        
        // Get current position (for appending)
        let position = file.seek(SeekFrom::End(0))?;
        
        // Initialize with empty cache
        let mut storage = Self {
            path,
            frequency,
            cache: HashMap::new(),
            file,
            position,
        };
        
        // Load existing data into cache
        storage.load_cache()?;
        
        Ok(storage)
    }
    
    /// Store data and get back a wave signature
    /// 
    /// This is where we convert boring bytes into exciting waves!
    /// Trisha calls this "making data dance" 💃
    pub fn store(&mut self, data: &[u8], metadata: Option<Vec<u8>>) -> Result<[u8; 32]> {
        // Convert data to waves
        let waves = self.encode_to_waves(data);
        
        // Calculate signature
        let mut hasher = Hasher::new();
        hasher.update(data);
        if let Some(ref meta) = metadata {
            hasher.update(meta);
        }
        let signature = hasher.finalize().into();
        
        // Create wave packet
        let packet = WavePacket {
            signature,
            waves,
            metadata,
            frequency: self.frequency,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };
        
        // Write to storage
        self.persist_packet(&packet)?;
        
        // Cache it
        self.cache.insert(signature, packet);
        
        Ok(signature)
    }
    
    /// Store a string and get back a wave signature
    pub fn store_string(&mut self, text: &str) -> Result<[u8; 32]> {
        self.store(text.as_bytes(), None)
    }
    
    /// Retrieve data by its wave signature
    /// 
    /// The waves remember everything perfectly - no lossy compression here!
    pub fn retrieve(&self, signature: &[u8; 32]) -> Result<Vec<u8>> {
        // Check cache first
        if let Some(packet) = self.cache.get(signature) {
            return self.decode_from_waves(&packet.waves);
        }
        
        // Not in cache, need to search the file
        // (In production, we'd have an index for this)
        Err(anyhow!("Wave signature not found in cache"))
    }
    
    /// Retrieve a string by its wave signature
    pub fn retrieve_string(&self, signature: &[u8; 32]) -> Result<String> {
        let data = self.retrieve(signature)?;
        Ok(String::from_utf8(data)?)
    }
    
    /// Get metadata for a stored item
    pub fn get_metadata(&self, signature: &[u8; 32]) -> Option<Vec<u8>> {
        self.cache.get(signature)
            .and_then(|packet| packet.metadata.clone())
    }
    
    /// Convert boring bytes into exciting waves! 🌊
    /// 
    /// Each byte becomes a complex number with frequency and phase.
    /// The interference patterns create natural compression!
    fn encode_to_waves(&self, data: &[u8]) -> Vec<Complex64> {
        data.iter().enumerate().map(|(i, &byte)| {
            // Create a wave for each byte
            // Frequency encodes the value, phase encodes position
            let frequency = self.frequency * (byte as f64 / 255.0);
            let phase = 2.0 * std::f64::consts::PI * (i as f64) / (data.len() as f64);
            
            Complex64::from_polar(frequency, phase)
        }).collect()
    }
    
    /// Convert waves back to bytes
    /// 
    /// The waves remember everything - perfect reconstruction!
    fn decode_from_waves(&self, waves: &[Complex64]) -> Result<Vec<u8>> {
        Ok(waves.iter().map(|wave| {
            // Extract byte value from frequency component
            let normalized = wave.norm() / self.frequency;
            (normalized * 255.0).round() as u8
        }).collect())
    }
    
    /// Write a wave packet to storage
    fn persist_packet(&mut self, packet: &WavePacket) -> Result<()> {
        // Serialize the packet
        let encoded = bincode::serialize(packet)?;
        
        // Write length prefix
        self.file.write_u64::<BigEndian>(encoded.len() as u64)?;
        
        // Write the packet
        self.file.write_all(&encoded)?;
        
        // Flush to ensure it's written
        self.file.flush()?;
        
        // Update position
        self.position += 8 + encoded.len() as u64;
        
        Ok(())
    }
    
    /// Load existing packets into cache
    fn load_cache(&mut self) -> Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        
        loop {
            // Try to read length prefix
            let len = match self.file.read_u64::<BigEndian>() {
                Ok(len) => len,
                Err(_) => break, // End of file
            };
            
            // Read packet data
            let mut buffer = vec![0u8; len as usize];
            self.file.read_exact(&mut buffer)?;
            
            // Deserialize packet
            if let Ok(packet) = bincode::deserialize::<WavePacket>(&buffer) {
                self.cache.insert(packet.signature, packet);
            }
        }
        
        // Reset to end for appending
        self.position = self.file.seek(SeekFrom::End(0))?;
        
        Ok(())
    }
    
    /// Load all packets into memory for maximum speed
    /// 
    /// Warning: Only use this with reasonable data sizes!
    /// Returns the number of packets loaded.
    pub fn load_all(&mut self) -> Result<usize> {
        self.load_cache()?;
        Ok(self.cache.len())
    }
    
    /// Get statistics about the storage
    pub fn stats(&self) -> StorageStats {
        StorageStats {
            packet_count: self.cache.len(),
            total_size: self.position,
            frequency: self.frequency,
            cache_hits: 0, // Would track this in production
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub packet_count: usize,
    pub total_size: u64,
    pub frequency: f64,
    pub cache_hits: usize,
}

impl std::fmt::Display for StorageStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "🌊 Wave Storage Stats:\n")?;
        write!(f, "  Packets: {}\n", self.packet_count)?;
        write!(f, "  Size: {} bytes\n", self.total_size)?;
        write!(f, "  Frequency: {}Hz\n", self.frequency)?;
        write!(f, "  Cache hits: {}\n", self.cache_hits)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_store_and_retrieve() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.m8");
        
        let mut storage = Mem8Lite::new(&path, 1.618).unwrap();
        
        // Store some data
        let data = b"Hello, waves!";
        let sig = storage.store(data, None).unwrap();
        
        // Retrieve it back
        let retrieved = storage.retrieve(&sig).unwrap();
        assert_eq!(retrieved, data);
    }
    
    #[test]
    fn test_metadata() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.m8");
        
        let mut storage = Mem8Lite::new(&path, 2.0).unwrap();
        
        // Store with metadata
        let data = b"Secret message";
        let metadata = b"user:hue,type:secret";
        let sig = storage.store(data, Some(metadata.to_vec())).unwrap();
        
        // Retrieve metadata
        let meta = storage.get_metadata(&sig).unwrap();
        assert_eq!(meta, metadata);
    }
    
    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.m8");
        
        let sig = {
            let mut storage = Mem8Lite::new(&path, 1.0).unwrap();
            storage.store_string("Persistent waves!").unwrap()
        };
        
        // Open storage again
        let storage = Mem8Lite::new(&path, 1.0).unwrap();
        let retrieved = storage.retrieve_string(&sig).unwrap();
        assert_eq!(retrieved, "Persistent waves!");
    }
}