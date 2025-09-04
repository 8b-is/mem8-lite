//! Audio processing module for MEM8-FS with multi-format support
//! 
//! Handles various sample rates and bit depths for maximum flexibility.
//! From lo-fi 16kHz mono to pristine 192kHz audiophile quality!
//!
//! Hue, this is where we make audio dance at any frequency!
//! Whether it's a phone recording or studio master, we'll find the wonder! 🎵

use crate::marine::{MarineProcessor, MarineMetadata};
use crate::lite::Mem8Lite;
use num_complex::Complex64;
use anyhow::{Result, anyhow};
use std::f64::consts::PI;

/// Supported audio sample rates
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleRate {
    /// Phone quality (16 kHz)
    Phone16k,
    /// Broadcast quality (22.05 kHz)  
    Broadcast22k,
    /// CD quality (44.1 kHz) - Our sweet spot!
    CD44k,
    /// DVD quality (48 kHz)
    DVD48k,
    /// Studio quality (96 kHz)
    Studio96k,
    /// Audiophile quality (192 kHz) - For the golden ears!
    Audiophile192k,
    /// Custom rate
    Custom(f64),
}

impl SampleRate {
    /// Get the actual sample rate in Hz
    pub fn as_f64(&self) -> f64 {
        match self {
            SampleRate::Phone16k => 16_000.0,
            SampleRate::Broadcast22k => 22_050.0,
            SampleRate::CD44k => 44_100.0,
            SampleRate::DVD48k => 48_000.0,
            SampleRate::Studio96k => 96_000.0,
            SampleRate::Audiophile192k => 192_000.0,
            SampleRate::Custom(rate) => *rate,
        }
    }
    
    /// Get optimal Marine processor settings for this sample rate
    pub fn optimal_marine_settings(&self) -> MarineProcessor {
        let mut processor = MarineProcessor::new();
        let rate = self.as_f64();
        
        // Adjust grid tick rate based on sample rate
        // Higher rates need higher tick rates to catch detail
        processor.grid_tick_rate = rate / 441.0;  // ~100Hz for 44.1kHz
        
        // Adjust sensitivity based on quality
        processor.clip_threshold = match self {
            SampleRate::Phone16k => 0.1,        // Less sensitive for noisy phone
            SampleRate::Broadcast22k => 0.08,
            SampleRate::CD44k => 0.05,          // Sweet spot
            SampleRate::DVD48k => 0.04,
            SampleRate::Studio96k => 0.02,      // Very sensitive for studio
            SampleRate::Audiophile192k => 0.01, // Ultra sensitive!
            SampleRate::Custom(_) => 0.05,
        };
        
        // Wonder threshold varies with quality
        // Higher quality audio reveals more wonder!
        processor.wonder_threshold = match self {
            SampleRate::Phone16k => 0.8,        // Hard to find wonder in phone audio
            SampleRate::Broadcast22k => 0.75,
            SampleRate::CD44k => 0.7,           // Standard wonder
            SampleRate::DVD48k => 0.65,
            SampleRate::Studio96k => 0.6,       // Studio reveals hidden wonder
            SampleRate::Audiophile192k => 0.5,  // Everything is wonderful at 192k!
            SampleRate::Custom(_) => 0.7,
        };
        
        processor
    }
    
    /// Get recommended frequency for wave encoding based on sample rate
    pub fn wave_frequency(&self) -> f64 {
        // Higher sample rates can handle higher wave frequencies
        match self {
            SampleRate::Phone16k => 0.5,        // Low frequency for compression
            SampleRate::Broadcast22k => 0.8,
            SampleRate::CD44k => 1.618,         // Golden ratio!
            SampleRate::DVD48k => 2.0,
            SampleRate::Studio96k => 3.14159,   // Pi for studio quality
            SampleRate::Audiophile192k => 4.669, // Feigenbaum constant for chaos!
            SampleRate::Custom(rate) => {
                // Scale frequency with sample rate
                1.618 * (rate / 44100.0).sqrt()
            }
        }
    }
}

/// Audio format configuration
#[derive(Debug, Clone)]
pub struct AudioFormat {
    /// Sample rate
    pub sample_rate: SampleRate,
    
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: usize,
    
    /// Bit depth (8, 16, 24, 32)
    pub bit_depth: usize,
    
    /// Is it floating point? (for 32-bit)
    pub is_float: bool,
}

impl AudioFormat {
    /// Create a standard CD quality format
    pub fn cd_quality() -> Self {
        Self {
            sample_rate: SampleRate::CD44k,
            channels: 2,
            bit_depth: 16,
            is_float: false,
        }
    }
    
    /// Create a phone quality format
    pub fn phone_quality() -> Self {
        Self {
            sample_rate: SampleRate::Phone16k,
            channels: 1,  // Mono
            bit_depth: 16,
            is_float: false,
        }
    }
    
    /// Create studio quality format
    pub fn studio_quality() -> Self {
        Self {
            sample_rate: SampleRate::Studio96k,
            channels: 2,
            bit_depth: 24,
            is_float: false,
        }
    }
    
    /// Create audiophile format (the ultimate!)
    pub fn audiophile() -> Self {
        Self {
            sample_rate: SampleRate::Audiophile192k,
            channels: 2,
            bit_depth: 32,
            is_float: true,  // 32-bit float for maximum dynamic range
        }
    }
}

/// Audio processor that combines Marine algorithm with MEM8 storage
pub struct AudioProcessor {
    format: AudioFormat,
    storage: Mem8Lite,
    processor: MarineProcessor,
}

impl AudioProcessor {
    /// Create a new audio processor
    pub fn new(format: AudioFormat, storage_path: &str) -> Result<Self> {
        let wave_freq = format.sample_rate.wave_frequency();
        let storage = Mem8Lite::new(storage_path, wave_freq)?;
        let processor = format.sample_rate.optimal_marine_settings();
        
        Ok(Self {
            format,
            storage,
            processor,
        })
    }
    
    /// Process raw PCM bytes based on format
    pub fn process_pcm(&mut self, pcm_data: &[u8]) -> Result<AudioAnalysis> {
        // Convert PCM to normalized float samples
        let samples = self.pcm_to_samples(pcm_data)?;
        
        // If stereo, mix to mono for Marine processing
        let mono_samples = if self.format.channels == 2 {
            self.stereo_to_mono(&samples)
        } else {
            samples
        };
        
        // Convert to waves
        let waves = self.samples_to_waves(&mono_samples);
        
        // Run Marine analysis
        let peaks = self.processor.process_waves(&waves);
        let metadata = self.processor.extract_metadata(&peaks);
        
        // Calculate additional audio-specific metrics
        let analysis = AudioAnalysis {
            marine_metadata: metadata,
            format: self.format.clone(),
            duration_seconds: mono_samples.len() as f64 / self.format.sample_rate.as_f64(),
            rms_level: calculate_rms(&mono_samples),
            peak_level: mono_samples.iter().fold(0.0, |a, &b| a.max(b.abs())),
            dynamic_range: calculate_dynamic_range(&mono_samples),
        };
        
        Ok(analysis)
    }
    
    /// Convert PCM bytes to normalized float samples
    fn pcm_to_samples(&self, pcm_data: &[u8]) -> Result<Vec<f64>> {
        let bytes_per_sample = self.format.bit_depth / 8;
        let total_samples = pcm_data.len() / bytes_per_sample;
        let mut samples = Vec::with_capacity(total_samples);
        
        for i in 0..total_samples {
            let offset = i * bytes_per_sample;
            let sample_bytes = &pcm_data[offset..offset + bytes_per_sample];
            
            let sample = match (self.format.bit_depth, self.format.is_float) {
                (8, false) => {
                    // 8-bit unsigned
                    let val = sample_bytes[0] as i8;
                    val as f64 / 128.0
                }
                (16, false) => {
                    // 16-bit signed
                    let val = i16::from_le_bytes([sample_bytes[0], sample_bytes[1]]);
                    val as f64 / 32768.0
                }
                (24, false) => {
                    // 24-bit signed (stored in lower 3 bytes)
                    let val = i32::from_le_bytes([sample_bytes[0], sample_bytes[1], sample_bytes[2], 0]);
                    val as f64 / 8388608.0  // 2^23
                }
                (32, false) => {
                    // 32-bit signed integer
                    let val = i32::from_le_bytes(sample_bytes.try_into()?);
                    val as f64 / 2147483648.0  // 2^31
                }
                (32, true) => {
                    // 32-bit float
                    f32::from_le_bytes(sample_bytes.try_into()?) as f64
                }
                _ => return Err(anyhow!("Unsupported bit depth: {}", self.format.bit_depth)),
            };
            
            samples.push(sample);
        }
        
        Ok(samples)
    }
    
    /// Convert stereo to mono by averaging channels
    fn stereo_to_mono(&self, samples: &[f64]) -> Vec<f64> {
        samples.chunks(2)
            .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
            .collect()
    }
    
    /// Convert samples to complex waves
    fn samples_to_waves(&self, samples: &[f64]) -> Vec<Complex64> {
        // Add frequency-dependent phase encoding
        let base_freq = self.format.sample_rate.wave_frequency();
        
        samples.iter().enumerate().map(|(i, &sample)| {
            // Phase encodes position with sample-rate awareness
            let phase = 2.0 * PI * i as f64 / samples.len() as f64;
            
            // Magnitude encodes amplitude with quality-based scaling
            let quality_factor = (self.format.sample_rate.as_f64() / 44100.0).sqrt();
            let magnitude = sample.abs() * quality_factor;
            
            Complex64::from_polar(magnitude * base_freq, phase)
        }).collect()
    }
    
    /// Store audio with Marine metadata
    pub fn store_audio(&mut self, pcm_data: &[u8], name: &str) -> Result<[u8; 32]> {
        let analysis = self.process_pcm(pcm_data)?;
        
        // Create rich metadata
        let metadata = serde_json::json!({
            "name": name,
            "format": {
                "sample_rate": self.format.sample_rate.as_f64(),
                "channels": self.format.channels,
                "bit_depth": self.format.bit_depth,
                "is_float": self.format.is_float,
            },
            "analysis": {
                "duration": analysis.duration_seconds,
                "rms_level": analysis.rms_level,
                "peak_level": analysis.peak_level,
                "dynamic_range": analysis.dynamic_range,
            },
            "marine": {
                "peaks": analysis.marine_metadata.total_peaks,
                "wonder": analysis.marine_metadata.wonder_count,
                "salience": analysis.marine_metadata.average_salience,
                "rhythm": analysis.marine_metadata.has_rhythm,
                "emotion": analysis.marine_metadata.emotional_signature,
            },
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        });
        
        let meta_bytes = serde_json::to_vec(&metadata)?;
        self.storage.store(pcm_data, Some(meta_bytes))
    }
}

/// Complete audio analysis results
#[derive(Debug, Clone)]
pub struct AudioAnalysis {
    pub marine_metadata: MarineMetadata,
    pub format: AudioFormat,
    pub duration_seconds: f64,
    pub rms_level: f64,
    pub peak_level: f64,
    pub dynamic_range: f64,
}

impl std::fmt::Display for AudioAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "🎵 Audio Analysis\n")?;
        write!(f, "  Format: {} Hz, {} ch, {} bit\n", 
            self.format.sample_rate.as_f64() as u32,
            self.format.channels,
            self.format.bit_depth)?;
        write!(f, "  Duration: {:.2}s\n", self.duration_seconds)?;
        write!(f, "  Levels: RMS={:.3}, Peak={:.3}\n", self.rms_level, self.peak_level)?;
        write!(f, "  Dynamic Range: {:.1} dB\n", self.dynamic_range)?;
        write!(f, "\n{}", self.marine_metadata)?;
        Ok(())
    }
}

/// Calculate RMS (Root Mean Square) level
fn calculate_rms(samples: &[f64]) -> f64 {
    let sum: f64 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f64).sqrt()
}

/// Calculate dynamic range in dB
fn calculate_dynamic_range(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    
    // Sort samples by absolute value
    let mut sorted: Vec<f64> = samples.iter().map(|s| s.abs()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    // Get 95th percentile (loud) and 5th percentile (quiet)
    let loud_idx = (sorted.len() as f64 * 0.95) as usize;
    let quiet_idx = (sorted.len() as f64 * 0.05) as usize;
    
    let loud = sorted[loud_idx.min(sorted.len() - 1)];
    let quiet = sorted[quiet_idx].max(0.000001);  // Avoid log(0)
    
    // Dynamic range in dB
    20.0 * (loud / quiet).log10()
}

/// Fun fact generator based on sample rate
pub fn sample_rate_fun_fact(rate: &SampleRate) -> &'static str {
    match rate {
        SampleRate::Phone16k => 
            "📞 16kHz: The classic phone quality! Trisha's accounting calls sound just fine at this rate.",
        SampleRate::Broadcast22k => 
            "📻 22.05kHz: Half of CD quality - perfect for AM radio memories!",
        SampleRate::CD44k => 
            "💿 44.1kHz: The golden standard! Why 44.1? It divides nicely for both PAL and NTSC video.",
        SampleRate::DVD48k => 
            "📀 48kHz: Movie magic frequency! Every film soundtrack you love was probably mastered at this.",
        SampleRate::Studio96k => 
            "🎙️ 96kHz: Studio grade! You can hear the engineer's coffee breath at this quality.",
        SampleRate::Audiophile192k => 
            "🎧 192kHz: Audiophile nirvana! You can hear colors and taste the music at this rate!",
        SampleRate::Custom(_) => 
            "🎵 Custom rate: Breaking the rules! Sometimes the best frequency is the one you choose.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sample_rates() {
        assert_eq!(SampleRate::CD44k.as_f64(), 44100.0);
        assert_eq!(SampleRate::Phone16k.as_f64(), 16000.0);
        assert_eq!(SampleRate::Custom(12345.0).as_f64(), 12345.0);
    }
    
    #[test]
    fn test_wave_frequencies() {
        // CD quality should use golden ratio
        assert_eq!(SampleRate::CD44k.wave_frequency(), 1.618);
        
        // Higher sample rates should have higher wave frequencies
        assert!(SampleRate::Studio96k.wave_frequency() > SampleRate::CD44k.wave_frequency());
    }
    
    #[test]
    fn test_rms_calculation() {
        let samples = vec![0.5, -0.5, 0.5, -0.5];
        let rms = calculate_rms(&samples);
        assert!((rms - 0.5).abs() < 0.001);
    }
}