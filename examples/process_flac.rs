//! Process FLAC files through MEM8's Marine algorithm
//! 
//! Usage: cargo run --example process_flac <path_to_flac_file>
//! 
//! Perfect for Brian Eno's "An Ending (Ascent)" or any FLAC file!
//! The Marine algorithm will find the moments of wonder in the waves.

use mem8_fs_lite::{Mem8Lite, MarineProcessor};
use mem8_fs_lite::audio_loader::{load_audio_file, format_fun_fact};
use anyhow::Result;
use std::env;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("🎵 MEM8-FS FLAC Processor with Marine Algorithm\n");
        println!("Usage: {} <path_to_flac_file>", args[0]);
        println!("\nExample: {} ~/Music/Brian_Eno_An_Ending_Ascent.flac", args[0]);
        println!("\nThis will:");
        println!("  1. Load the FLAC file (or WAV)");
        println!("  2. Process it through the Marine algorithm");
        println!("  3. Detect moments of 'wonder'");
        println!("  4. Store it with emotional signatures");
        println!("  5. Show you what the waves remember!");
        return Ok(());
    }
    
    let audio_path = &args[1];
    
    if !Path::new(audio_path).exists() {
        println!("❌ File not found: {}", audio_path);
        return Ok(());
    }
    
    println!("🌊 Loading audio file: {}", audio_path);
    
    // Load the audio file (FLAC, WAV, or raw PCM)
    let loaded = load_audio_file(audio_path)?;
    
    println!("✅ Loaded successfully!");
    println!("  Format: {} Hz, {} channels, {} bit", 
        loaded.format.sample_rate.as_f64() as u32,
        loaded.format.channels,
        loaded.format.bit_depth);
    println!("  Samples: {} ({:.2} seconds)", 
        loaded.samples.len(),
        loaded.samples.len() as f64 / loaded.format.sample_rate.as_f64() / loaded.format.channels as f64);
    
    // Show metadata if available
    if let Some(ref metadata) = loaded.metadata {
        println!("\n{}", metadata);
    }
    
    println!("\n{}", format_fun_fact(&loaded.file_format));
    
    // Convert to mono if stereo (Marine works best with mono)
    let mono_samples = if loaded.format.channels == 2 {
        println!("\n📻 Converting stereo to mono for Marine processing...");
        loaded.samples.chunks(2)
            .map(|ch| (ch[0] + ch.get(1).unwrap_or(&0.0)) / 2.0)
            .collect::<Vec<_>>()
    } else {
        loaded.samples.clone()
    };
    
    // Process through Marine algorithm
    println!("\n🌊 Running Marine algorithm for salience detection...");
    
    let mut processor = MarineProcessor::for_audio(loaded.format.sample_rate.as_f64());
    
    // Tune for ambient music if it's Brian Eno
    if audio_path.to_lowercase().contains("eno") || 
       audio_path.to_lowercase().contains("ambient") ||
       (loaded.metadata.as_ref().and_then(|m| m.artist.as_ref())
        .map(|a| a.to_lowercase().contains("eno")).unwrap_or(false)) {
        println!("🎹 Detected ambient music - tuning for subtle wonder...");
        processor.wonder_threshold = 0.4;  // Lower threshold for ambient
        processor.clip_threshold = 0.01;   // More sensitive
        processor.weights.harmonic = 0.4;  // Ambient loves harmonics
        processor.weights.wonder = 0.3;    // Extra wonder weight
    }
    
    // Process the audio
    let peaks = processor.process_samples(&mono_samples);
    let metadata = processor.extract_metadata(&peaks);
    
    println!("\n{}", metadata);
    
    // Analyze specific sections for "An Ending (Ascent)"
    if audio_path.to_lowercase().contains("ending") || 
       audio_path.to_lowercase().contains("ascent") {
        analyze_ascent_sections(&mono_samples, &loaded.format.sample_rate.as_f64())?;
    }
    
    // Store in MEM8 format with Marine metadata
    println!("\n💾 Storing in MEM8 wave format...");
    
    let storage_path = "/tmp/mem8_flac_storage.m8";
    let mut storage = Mem8Lite::new(storage_path, loaded.format.sample_rate.wave_frequency())?;
    
    // Create rich metadata
    let meta_json = serde_json::json!({
        "file": audio_path,
        "format": {
            "type": match loaded.file_format {
                mem8_fs_lite::audio_loader::AudioFileFormat::Flac => "FLAC",
                mem8_fs_lite::audio_loader::AudioFileFormat::Wav => "WAV",
                mem8_fs_lite::audio_loader::AudioFileFormat::RawPcm(_) => "RAW_PCM",
            },
            "sample_rate": loaded.format.sample_rate.as_f64(),
            "channels": loaded.format.channels,
            "bit_depth": loaded.format.bit_depth,
        },
        "original_metadata": loaded.metadata,
        "marine_analysis": {
            "total_peaks": metadata.total_peaks,
            "wonder_count": metadata.wonder_count,
            "average_salience": metadata.average_salience,
            "max_salience": metadata.max_salience,
            "has_rhythm": metadata.has_rhythm,
            "emotional_signature": metadata.emotional_signature,
        },
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    });
    
    let signature = storage.store(
        &mono_samples.iter()
            .flat_map(|&s| {
                let pcm = (s * 32767.0).max(-32768.0).min(32767.0) as i16;
                pcm.to_le_bytes()
            })
            .collect::<Vec<_>>(),
        Some(serde_json::to_vec(&meta_json)?),
    )?;
    
    println!("✅ Stored with wave signature: {}", hex::encode(&signature[..16]));
    println!("\n🎵 The waves will remember this music forever!");
    
    // Final thought
    if metadata.wonder_count > metadata.total_peaks / 2 {
        println!("\n✨ This audio is full of wonder - over half the peaks inspired awe!");
        println!("   Trisha says: 'Even the numbers are dancing!'");
    }
    
    Ok(())
}

/// Special analysis for "An Ending (Ascent)"
fn analyze_ascent_sections(samples: &[f64], sample_rate: &f64) -> Result<()> {
    println!("\n🎹 Special Analysis for 'An Ending (Ascent)':");
    
    let samples_per_second = *sample_rate as usize;
    
    // Analyze key sections
    let sections = [
        (0, 30, "Opening drift"),
        (30, 90, "First theme emergence"),
        (90, 150, "The ascending melody"),
        (150, 210, "Climax and suspension"),
        (210, 260, "Gentle descent"),
    ];
    
    for (start_sec, end_sec, description) in &sections {
        let start_idx = start_sec * samples_per_second;
        let end_idx = (end_sec * samples_per_second).min(samples.len());
        
        if start_idx >= samples.len() {
            break;
        }
        
        let section = &samples[start_idx..end_idx];
        let rms = (section.iter().map(|s| s * s).sum::<f64>() / section.len() as f64).sqrt();
        let peak = section.iter().fold(0.0f64, |a, &b| a.max(b.abs()));
        
        println!("  {:3}-{:3}s {}: RMS={:.3}, Peak={:.3}", 
                 start_sec, end_sec, description, rms, peak);
    }
    
    Ok(())
}

/// Get the wave frequency for this sample rate
trait WaveFrequency {
    fn wave_frequency(&self) -> f64;
}

impl WaveFrequency for mem8_fs_lite::audio::SampleRate {
    fn wave_frequency(&self) -> f64 {
        use mem8_fs_lite::audio::SampleRate;
        match self {
            SampleRate::Phone16k => 0.5,
            SampleRate::Broadcast22k => 0.8,
            SampleRate::CD44k => 1.618,  // Golden ratio!
            SampleRate::DVD48k => 2.0,
            SampleRate::Studio96k => 3.14159,
            SampleRate::Audiophile192k => 4.669,
            SampleRate::Custom(rate) => 1.618 * (rate / 44100.0).sqrt(),
        }
    }
}