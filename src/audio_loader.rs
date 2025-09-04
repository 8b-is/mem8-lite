//! Audio file loading support for multiple formats
//! 
//! Handles FLAC, WAV, and raw PCM files with automatic format detection.
//! FLAC is our favorite - lossless compression with metadata preservation!
//!
//! Hue, this is where we make any audio format dance in waves!
//! Trisha says FLAC files are like compressed accounting records - 
//! smaller but perfectly accurate! 📊🎵

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use anyhow::{Result, anyhow};
use crate::audio::{AudioFormat, SampleRate};

/// Supported audio file formats
#[derive(Debug, Clone, PartialEq)]
pub enum AudioFileFormat {
    /// FLAC - Free Lossless Audio Codec (the audiophile's choice!)
    Flac,
    /// WAV - Waveform Audio File Format (the classic)
    Wav,
    /// Raw PCM data (when you know what you're doing)
    RawPcm(AudioFormat),
}

/// Loaded audio data with format information
#[derive(Debug, Clone)]
pub struct LoadedAudio {
    /// The audio samples (normalized to -1.0 to 1.0)
    pub samples: Vec<f64>,
    
    /// The format of the audio
    pub format: AudioFormat,
    
    /// Original file format
    pub file_format: AudioFileFormat,
    
    /// Optional metadata from the file
    pub metadata: Option<AudioMetadata>,
}

/// Audio metadata extracted from files
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioMetadata {
    /// Track title
    pub title: Option<String>,
    
    /// Artist name
    pub artist: Option<String>,
    
    /// Album name
    pub album: Option<String>,
    
    /// Track number
    pub track: Option<u32>,
    
    /// Year of release
    pub year: Option<u32>,
    
    /// Genre
    pub genre: Option<String>,
    
    /// Any comments (perfect for memory annotations!)
    pub comment: Option<String>,
}

/// Load audio from any supported file format
/// 
/// Automatically detects format from file extension and magic bytes.
/// Returns normalized samples ready for Marine processing!
pub fn load_audio_file<P: AsRef<Path>>(path: P) -> Result<LoadedAudio> {
    let path = path.as_ref();
    
    // Try to detect format from extension first
    let format = match path.extension().and_then(|e| e.to_str()) {
        Some("flac") | Some("FLAC") => AudioFileFormat::Flac,
        Some("wav") | Some("WAV") => AudioFileFormat::Wav,
        Some("pcm") | Some("raw") => {
            // For raw PCM, assume CD quality
            AudioFileFormat::RawPcm(AudioFormat::cd_quality())
        }
        _ => {
            // Try to detect from file contents
            detect_format_from_file(path)?
        }
    };
    
    match format {
        AudioFileFormat::Flac => load_flac(path),
        AudioFileFormat::Wav => load_wav(path),
        AudioFileFormat::RawPcm(fmt) => load_raw_pcm(path, fmt),
    }
}

/// Detect format from file magic bytes
fn detect_format_from_file(path: &Path) -> Result<AudioFileFormat> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    
    match &magic {
        b"fLaC" => Ok(AudioFileFormat::Flac),
        b"RIFF" => Ok(AudioFileFormat::Wav),
        _ => Err(anyhow!("Unknown audio format. Try .flac, .wav, or .pcm"))
    }
}

/// Load a FLAC file
/// 
/// FLAC is perfect for our wave storage - it's already thinking in terms
/// of compression and preservation, just like MEM8!
pub fn load_flac(path: &Path) -> Result<LoadedAudio> {
    let file = File::open(path)?;
    let mut reader = claxon::FlacReader::new(BufReader::new(file))?;
    
    // Get stream info
    let streaminfo = reader.streaminfo();
    let sample_rate = streaminfo.sample_rate;
    let channels = streaminfo.channels as usize;
    let bits_per_sample = streaminfo.bits_per_sample;
    
    // Determine our sample rate enum
    let sample_rate_enum = match sample_rate {
        16000 => SampleRate::Phone16k,
        22050 => SampleRate::Broadcast22k,
        44100 => SampleRate::CD44k,
        48000 => SampleRate::DVD48k,
        96000 => SampleRate::Studio96k,
        192000 => SampleRate::Audiophile192k,
        other => SampleRate::Custom(other as f64),
    };
    
    // Extract metadata if available
    let metadata = extract_flac_metadata(&mut reader);
    
    // Read all samples
    let mut samples = Vec::new();
    let max_value = (1i64 << (bits_per_sample - 1)) as f64;
    
    // Read all samples using the samples iterator
    // FLAC samples are always returned as i32, regardless of bit depth
    let sample_reader = reader.samples();
    for sample in sample_reader {
        let sample = sample?;
        samples.push(sample as f64 / max_value);
    }
    
    // If the read loop ended with an error other than EOF, that's fine
    // (claxon returns an error at EOF which is expected)
    
    let format = AudioFormat {
        sample_rate: sample_rate_enum,
        channels,
        bit_depth: bits_per_sample as usize,
        is_float: false,
    };
    
    Ok(LoadedAudio {
        samples,
        format,
        file_format: AudioFileFormat::Flac,
        metadata,
    })
}

/// Extract metadata from FLAC file
fn extract_flac_metadata(reader: &mut claxon::FlacReader<BufReader<File>>) -> Option<AudioMetadata> {
    // Get Vorbis comments (FLAC metadata)
    let tags = reader.tags();
    
    let mut metadata = AudioMetadata {
        title: None,
        artist: None,
        album: None,
        track: None,
        year: None,
        genre: None,
        comment: None,
    };
    
    for (key, value) in tags {
        match key.to_uppercase().as_str() {
            "TITLE" => metadata.title = Some(value.to_string()),
            "ARTIST" => metadata.artist = Some(value.to_string()),
            "ALBUM" => metadata.album = Some(value.to_string()),
            "TRACKNUMBER" => metadata.track = value.parse().ok(),
            "DATE" | "YEAR" => metadata.year = value.parse().ok(),
            "GENRE" => metadata.genre = Some(value.to_string()),
            "COMMENT" => metadata.comment = Some(value.to_string()),
            _ => {}
        }
    }
    
    // Return None if no metadata was found
    if metadata.title.is_none() && metadata.artist.is_none() {
        None
    } else {
        Some(metadata)
    }
}

/// Load a WAV file
pub fn load_wav(path: &Path) -> Result<LoadedAudio> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    
    // Determine sample rate
    let sample_rate_enum = match spec.sample_rate {
        16000 => SampleRate::Phone16k,
        22050 => SampleRate::Broadcast22k,
        44100 => SampleRate::CD44k,
        48000 => SampleRate::DVD48k,
        96000 => SampleRate::Studio96k,
        192000 => SampleRate::Audiophile192k,
        other => SampleRate::Custom(other as f64),
    };
    
    // Read samples based on format
    let samples: Vec<f64> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max_value = (1i64 << (spec.bits_per_sample - 1)) as f64;
            reader.samples::<i32>()
                .map(|s| s.unwrap() as f64 / max_value)
                .collect()
        }
        hound::SampleFormat::Float => {
            reader.samples::<f32>()
                .map(|s| s.unwrap() as f64)
                .collect()
        }
    };
    
    let format = AudioFormat {
        sample_rate: sample_rate_enum,
        channels: spec.channels as usize,
        bit_depth: spec.bits_per_sample as usize,
        is_float: spec.sample_format == hound::SampleFormat::Float,
    };
    
    Ok(LoadedAudio {
        samples,
        format,
        file_format: AudioFileFormat::Wav,
        metadata: None,  // WAV files typically don't have metadata
    })
}

/// Load raw PCM data with known format
pub fn load_raw_pcm(path: &Path, format: AudioFormat) -> Result<LoadedAudio> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Convert bytes to samples based on format
    let bytes_per_sample = format.bit_depth / 8;
    let num_samples = buffer.len() / bytes_per_sample;
    let mut samples = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let offset = i * bytes_per_sample;
        let sample_bytes = &buffer[offset..offset + bytes_per_sample];
        
        let sample = match (format.bit_depth, format.is_float) {
            (16, false) => {
                let val = i16::from_le_bytes([sample_bytes[0], sample_bytes[1]]);
                val as f64 / 32768.0
            }
            (24, false) => {
                let val = i32::from_le_bytes([sample_bytes[0], sample_bytes[1], sample_bytes[2], 0]);
                val as f64 / 8388608.0
            }
            (32, false) => {
                let val = i32::from_le_bytes(sample_bytes.try_into()?);
                val as f64 / 2147483648.0
            }
            (32, true) => {
                f32::from_le_bytes(sample_bytes.try_into()?) as f64
            }
            _ => return Err(anyhow!("Unsupported PCM format")),
        };
        
        samples.push(sample);
    }
    
    Ok(LoadedAudio {
        samples,
        format: format.clone(),
        file_format: AudioFileFormat::RawPcm(format),
        metadata: None,
    })
}

/// Fun facts about audio formats
pub fn format_fun_fact(format: &AudioFileFormat) -> &'static str {
    match format {
        AudioFileFormat::Flac => 
            "🎵 FLAC: Like MEM8 for audio - lossless compression that preserves every wave!",
        AudioFileFormat::Wav => 
            "🌊 WAV: The original wave format - uncompressed and honest!",
        AudioFileFormat::RawPcm(_) => 
            "🎛️ Raw PCM: Pure samples, no headers - for when you speak fluent audio!",
    }
}

impl std::fmt::Display for AudioMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "🎵 Audio Metadata:\n")?;
        if let Some(ref title) = self.title {
            write!(f, "  Title: {}\n", title)?;
        }
        if let Some(ref artist) = self.artist {
            write!(f, "  Artist: {}\n", artist)?;
        }
        if let Some(ref album) = self.album {
            write!(f, "  Album: {}\n", album)?;
        }
        if let Some(track) = self.track {
            write!(f, "  Track: {}\n", track)?;
        }
        if let Some(year) = self.year {
            write!(f, "  Year: {}\n", year)?;
        }
        if let Some(ref genre) = self.genre {
            write!(f, "  Genre: {}\n", genre)?;
        }
        if let Some(ref comment) = self.comment {
            write!(f, "  Comment: {}\n", comment)?;
        }
        Ok(())
    }
}