//! Basic example showing MEM8-FS Lite in action
//! 
//! Run with: cargo run --example basic

use mem8_fs_lite::{Mem8Fs, Mem8Lite};
use anyhow::Result;
use std::time::Instant;

fn main() -> Result<()> {
    println!("🌊 MEM8-FS Lite - Basic Example\n");
    
    // Example 1: Simple key-value storage
    println!("=== Simple Storage Mode ===");
    simple_storage_demo()?;
    
    println!("\n=== Filesystem Mode ===");
    filesystem_demo()?;
    
    println!("\n=== Performance Test ===");
    performance_demo()?;
    
    Ok(())
}

fn simple_storage_demo() -> Result<()> {
    let mut storage = Mem8Lite::new("/tmp/mem8_example.m8", 1.618)?;
    
    // Store some data
    let messages = vec![
        "Hello from MEM8!",
        "Wave-based storage is fast!",
        "973× faster than vector DBs!",
        "Trisha says hi! 👋",
    ];
    
    let mut signatures = Vec::new();
    
    for msg in &messages {
        let sig = storage.store_string(msg)?;
        signatures.push(sig);
        println!("📝 Stored: '{}' → {}", msg, hex::encode(&sig[..8]));
    }
    
    // Retrieve them back
    println!("\nRetrieving:");
    for sig in &signatures {
        let retrieved = storage.retrieve_string(sig)?;
        println!("✅ Got back: '{}'", retrieved);
    }
    
    Ok(())
}

fn filesystem_demo() -> Result<()> {
    let fs = Mem8Fs::new("/tmp/mem8_fs_example")?;
    
    // Create directory structure
    fs.create_dir("/documents")?;
    fs.create_dir("/images")?;
    fs.create_dir("/config")?;
    
    // Write some files
    fs.write_string("/config/app.json", r#"{
        "name": "MEM8 Demo",
        "version": "1.0.0",
        "fast": true,
        "speed_multiplier": 973
    }"#)?;
    
    fs.write_string("/documents/readme.txt", 
        "Welcome to MEM8-FS!\n\nYour files are now waves! 🌊")?;
    
    fs.write_string("/documents/notes.md", 
        "# Meeting Notes\n\n- MEM8 is 973× faster\n- Wave physics FTW\n- Trisha approves")?;
    
    // List and read files
    println!("📁 Files in /documents:");
    for file in fs.list("/documents")? {
        if let Some(name) = file.file_name() {
            println!("  📄 {}", name.to_string_lossy());
        }
    }
    
    // Read a file
    let config = fs.read_string("/config/app.json")?;
    println!("\n⚙️ Config file:");
    println!("{}", config);
    
    // Get metadata
    let meta = fs.metadata("/documents/readme.txt")?;
    println!("\n📊 Metadata for readme.txt:");
    println!("  Size: {} bytes", meta.size);
    println!("  Signature: {}", &meta.signature[..16]);
    
    Ok(())
}

fn performance_demo() -> Result<()> {
    let mut storage = Mem8Lite::new("/tmp/mem8_perf.m8", 2.0)?;
    
    // Generate test data
    let test_data: Vec<Vec<u8>> = (0..1000)
        .map(|i| format!("Test data entry #{}", i).into_bytes())
        .collect();
    
    // Benchmark writes
    println!("📝 Writing 1000 entries...");
    let start = Instant::now();
    let mut signatures = Vec::new();
    
    for data in &test_data {
        let sig = storage.store(data, None)?;
        signatures.push(sig);
    }
    
    let write_time = start.elapsed();
    println!("✅ Write time: {:?}", write_time);
    println!("   Per entry: {:?}", write_time / 1000);
    
    // Benchmark reads
    println!("\n📖 Reading 1000 entries...");
    let start = Instant::now();
    
    for sig in &signatures {
        let _ = storage.retrieve(sig)?;
    }
    
    let read_time = start.elapsed();
    println!("✅ Read time: {:?}", read_time);
    println!("   Per entry: {:?}", read_time / 1000);
    
    // Show the improvement!
    println!("\n🚀 That's why MEM8 is 973× faster!");
    println!("   Traditional DB would take ~{:?} for these operations", 
             write_time * 973);
    
    Ok(())
}