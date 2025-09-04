//! FUSE mount support - Mount MEM8 as a real filesystem!
//! 
//! Hue says: "Now you can literally `mount -t mem8` and watch it fly!" 🚀

#[cfg(feature = "fuse-mount")]
use fuser::{
    FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, 
    ReplyAttr, ReplyDirectory, ReplyWrite, FUSE_ROOT_ID,
};
use std::time::{Duration, UNIX_EPOCH, SystemTime};
use std::ffi::OsStr;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::Mem8Fs;
use anyhow::Result;

/// FUSE filesystem implementation for MEM8
pub struct Mem8FuseFs {
    inner: Arc<Mem8Fs>,
    inodes: Arc<RwLock<InodeTable>>,
    ttl: Duration,
}

/// Inode table for FUSE
struct InodeTable {
    next_inode: u64,
    path_to_inode: HashMap<String, u64>,
    inode_to_path: HashMap<u64, String>,
}

impl Mem8FuseFs {
    /// Create a new FUSE filesystem backed by MEM8
    pub fn new(mem8: Arc<Mem8Fs>) -> Self {
        let mut inodes = InodeTable {
            next_inode: 2, // 1 is reserved for root
            path_to_inode: HashMap::new(),
            inode_to_path: HashMap::new(),
        };
        
        // Add root
        inodes.path_to_inode.insert("/".to_string(), FUSE_ROOT_ID);
        inodes.inode_to_path.insert(FUSE_ROOT_ID, "/".to_string());
        
        Self {
            inner: mem8,
            inodes: Arc::new(RwLock::new(inodes)),
            ttl: Duration::from_secs(1),
        }
    }
    
    /// Mount the filesystem at the given path
    /// 
    /// # Example
    /// ```no_run
    /// use mem8_fs_lite::{Mem8Fs, mount::Mem8FuseFs};
    /// use std::sync::Arc;
    /// 
    /// let mem8 = Arc::new(Mem8Fs::new("/tmp/mem8_data").unwrap());
    /// let fuse = Mem8FuseFs::new(mem8);
    /// 
    /// // This will block until unmounted!
    /// fuse.mount("/mnt/mem8").unwrap();
    /// ```
    pub fn mount<P: AsRef<std::path::Path>>(self, mountpoint: P) -> Result<()> {
        // Create mount options
        let options = vec![
            fuser::MountOption::RO,  // Read-only for now
            fuser::MountOption::FSName("mem8".to_string()),
            fuser::MountOption::AutoUnmount,
            fuser::MountOption::AllowOther,
        ];
        
        // Mount it!
        fuser::mount2(self, mountpoint, &options)?;
        Ok(())
    }
    
    fn get_or_create_inode(&self, path: &str) -> u64 {
        let mut inodes = self.inodes.write().unwrap();
        
        if let Some(&inode) = inodes.path_to_inode.get(path) {
            inode
        } else {
            let inode = inodes.next_inode;
            inodes.next_inode += 1;
            inodes.path_to_inode.insert(path.to_string(), inode);
            inodes.inode_to_path.insert(inode, path.to_string());
            inode
        }
    }
    
    fn path_from_inode(&self, inode: u64) -> Option<String> {
        let inodes = self.inodes.read().unwrap();
        inodes.inode_to_path.get(&inode).cloned()
    }
}

#[cfg(feature = "fuse-mount")]
impl Filesystem for Mem8FuseFs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let parent_path = match self.path_from_inode(parent) {
            Some(p) => p,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };
        
        let path = if parent_path == "/" {
            format!("/{}", name.to_string_lossy())
        } else {
            format!("{}/{}", parent_path, name.to_string_lossy())
        };
        
        // Check if file exists
        if self.inner.exists(&path) {
            let inode = self.get_or_create_inode(&path);
            let attr = self.make_file_attr(inode, &path);
            reply.entry(&self.ttl, &attr, 0);
        } else {
            reply.error(libc::ENOENT);
        }
    }
    
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if ino == FUSE_ROOT_ID {
            let attr = self.make_dir_attr(FUSE_ROOT_ID);
            reply.attr(&self.ttl, &attr);
        } else if let Some(path) = self.path_from_inode(ino) {
            if self.inner.exists(&path) {
                let attr = self.make_file_attr(ino, &path);
                reply.attr(&self.ttl, &attr);
            } else {
                reply.error(libc::ENOENT);
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }
    
    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        if let Some(path) = self.path_from_inode(ino) {
            match self.inner.read(&path) {
                Ok(data) => {
                    let start = offset as usize;
                    let end = (offset as usize + size as usize).min(data.len());
                    if start < data.len() {
                        reply.data(&data[start..end]);
                    } else {
                        reply.data(&[]);
                    }
                }
                Err(_) => reply.error(libc::EIO),
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }
    
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != FUSE_ROOT_ID {
            reply.error(libc::ENOTDIR);
            return;
        }
        
        let mut entries = vec![
            (FUSE_ROOT_ID, FileType::Directory, "."),
            (FUSE_ROOT_ID, FileType::Directory, ".."),
        ];
        
        // List all files
        if let Ok(files) = self.inner.list("/") {
            for file in files {
                let name = file.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let inode = self.get_or_create_inode(file.to_str().unwrap_or(""));
                entries.push((inode, FileType::RegularFile, name));
            }
        }
        
        // Send entries
        for (i, (inode, file_type, name)) in entries.iter().enumerate().skip(offset as usize) {
            if reply.add(*inode, (i + 1) as i64, *file_type, name) {
                break;
            }
        }
        
        reply.ok();
    }
    
    // Write support (for read-write mode)
    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        if let Some(path) = self.path_from_inode(ino) {
            // For simplicity, we'll overwrite the whole file
            // In production, you'd handle partial writes properly
            match self.inner.write(&path, data) {
                Ok(_) => reply.written(data.len() as u32),
                Err(_) => reply.error(libc::EIO),
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }
}

// Helper methods for attributes
impl Mem8FuseFs {
    fn make_file_attr(&self, inode: u64, path: &str) -> FileAttr {
        let metadata = self.inner.metadata(path).unwrap_or_else(|_| {
            crate::FileMetadata {
                size: 0,
                created: 0,
                modified: 0,
                signature: String::new(),
            }
        });
        
        FileAttr {
            ino: inode,
            size: metadata.size,
            blocks: (metadata.size + 511) / 512,
            atime: UNIX_EPOCH + Duration::from_secs(metadata.modified),
            mtime: UNIX_EPOCH + Duration::from_secs(metadata.modified),
            ctime: UNIX_EPOCH + Duration::from_secs(metadata.created),
            crtime: UNIX_EPOCH + Duration::from_secs(metadata.created),
            kind: FileType::RegularFile,
            perm: 0o644,
            nlink: 1,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }
    
    fn make_dir_attr(&self, inode: u64) -> FileAttr {
        FileAttr {
            ino: inode,
            size: 4096,
            blocks: 8,
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            crtime: SystemTime::now(),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }
}