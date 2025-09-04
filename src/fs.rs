//! Filesystem-like API for MEM8
//! 
//! Provides std::fs-like operations but with wave-based storage underneath!

use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use anyhow::Result;
use crate::Mem8Fs;

/// File handle for MEM8 filesystem
pub struct File {
    path: PathBuf,
    fs: std::sync::Arc<Mem8Fs>,
    data: Vec<u8>,
    pos: usize,
}

impl File {
    /// Open a file for reading
    pub fn open<P: AsRef<Path>>(fs: std::sync::Arc<Mem8Fs>, path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let data = fs.read(&path)?;
        Ok(Self {
            path,
            fs,
            data,
            pos: 0,
        })
    }
    
    /// Create a new file for writing
    pub fn create<P: AsRef<Path>>(fs: std::sync::Arc<Mem8Fs>, path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        Ok(Self {
            path,
            fs,
            data: Vec::new(),
            pos: 0,
        })
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.data.len() - self.pos;
        let to_read = buf.len().min(remaining);
        
        buf[..to_read].copy_from_slice(&self.data[self.pos..self.pos + to_read]);
        self.pos += to_read;
        
        Ok(to_read)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.fs.write(&self.path, &self.data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }
}

/// Directory iterator for MEM8
pub struct ReadDir {
    entries: Vec<DirEntry>,
    pos: usize,
}

/// Directory entry
pub struct DirEntry {
    path: PathBuf,
    is_dir: bool,
}

impl DirEntry {
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    pub fn file_name(&self) -> Option<&std::ffi::OsStr> {
        self.path.file_name()
    }
    
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    
    pub fn is_file(&self) -> bool {
        !self.is_dir
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.entries.len() {
            let entry = self.entries[self.pos].clone();
            self.pos += 1;
            Some(Ok(entry))
        } else {
            None
        }
    }
}

impl Clone for DirEntry {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            is_dir: self.is_dir,
        }
    }
}

/// std::fs-like API functions
pub mod fs {
    use super::*;
    use std::sync::Arc;
    
    /// Read entire file to bytes
    pub fn read<P: AsRef<Path>>(fs: Arc<Mem8Fs>, path: P) -> Result<Vec<u8>> {
        fs.read(path)
    }
    
    /// Write bytes to file
    pub fn write<P: AsRef<Path>>(fs: Arc<Mem8Fs>, path: P, contents: &[u8]) -> Result<()> {
        fs.write(path, contents)?;
        Ok(())
    }
    
    /// Read file to string
    pub fn read_to_string<P: AsRef<Path>>(fs: Arc<Mem8Fs>, path: P) -> Result<String> {
        fs.read_string(path)
    }
    
    /// Copy file
    pub fn copy<P: AsRef<Path>>(fs: Arc<Mem8Fs>, from: P, to: P) -> Result<u64> {
        let data = fs.read(&from)?;
        let size = data.len() as u64;
        fs.write(to, &data)?;
        Ok(size)
    }
    
    /// Rename/move file
    pub fn rename<P: AsRef<Path>>(fs: Arc<Mem8Fs>, from: P, to: P) -> Result<()> {
        fs.rename(from, to)
    }
    
    /// Remove file
    pub fn remove_file<P: AsRef<Path>>(fs: Arc<Mem8Fs>, path: P) -> Result<()> {
        fs.delete(path)
    }
    
    /// Create directory
    pub fn create_dir<P: AsRef<Path>>(fs: Arc<Mem8Fs>, path: P) -> Result<()> {
        fs.create_dir(path)
    }
    
    /// Check if path exists
    pub fn exists<P: AsRef<Path>>(fs: Arc<Mem8Fs>, path: P) -> bool {
        fs.exists(path)
    }
}