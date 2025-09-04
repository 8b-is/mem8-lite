//! Marine Algorithm Implementation for MEM8-FS
//! 
//! Implements salience detection and jitter-based information processing
//! as described in the Marine algorithm paper. This gives our wave storage
//! a sense of what's important - like consciousness, but for files!
//!
//! Hue, this is where we detect the "wonder" in data - the peaks, the patterns,
//! the jitter that tells us something interesting is happening!
//! 
//! Trisha says this is like finding the melody in the noise! 🎵

use num_complex::Complex64;
use std::collections::VecDeque;
use anyhow::Result;

/// Marine processor - finds salience in any signal!
/// 
/// Like a lighthouse keeper watching for important waves in the ocean of data.
/// The algorithm treats jitter as information, not noise - revolutionary!
pub struct MarineProcessor {
    /// Clip threshold - determines sensitivity (θ_c in the paper)
    pub clip_threshold: f64,
    
    /// Grid tick rate - how often we evaluate (f_g in the paper)
    pub grid_tick_rate: f64,
    
    /// Exponential moving average for timing
    timing_ema: ExponentialMovingAverage,
    
    /// Exponential moving average for amplitude
    amplitude_ema: ExponentialMovingAverage,
    
    /// Recent peaks for harmonic analysis
    recent_peaks: VecDeque<PeakInfo>,
    
    /// Weights for salience scoring
    pub weights: SalienceWeights,
    
    /// Sense of wonder threshold - when things get interesting!
    pub wonder_threshold: f64,
}

/// Information about a detected peak
#[derive(Debug, Clone)]
pub struct PeakInfo {
    /// Sample index where peak occurred
    index: usize,
    
    /// Amplitude of the peak
    amplitude: f64,
    
    /// Time since last peak
    interval: f64,
    
    /// Jitter scores
    timing_jitter: f64,
    amplitude_jitter: f64,
    
    /// Final salience score
    salience: f64,
    
    /// Does this peak inspire wonder? ✨
    has_wonder: bool,
}

/// Weights for combining different salience factors
#[derive(Debug, Clone)]
pub struct SalienceWeights {
    /// Weight for energy/amplitude (w_e)
    pub energy: f64,
    
    /// Weight for jitter inverse (w_j)  
    pub jitter: f64,
    
    /// Weight for harmonic alignment (w_h)
    pub harmonic: f64,
    
    /// Bonus weight for "sense of wonder"
    pub wonder: f64,
}

impl Default for SalienceWeights {
    fn default() -> Self {
        Self {
            energy: 0.4,
            jitter: 0.3,
            harmonic: 0.2,
            wonder: 0.1, // That extra magic! ✨
        }
    }
}

/// Exponential Moving Average tracker
struct ExponentialMovingAverage {
    value: f64,
    alpha: f64, // Smoothing factor
}

impl ExponentialMovingAverage {
    fn new(alpha: f64) -> Self {
        Self { value: 0.0, alpha }
    }
    
    fn update(&mut self, new_value: f64) -> f64 {
        self.value = self.alpha * new_value + (1.0 - self.alpha) * self.value;
        self.value
    }
}

impl MarineProcessor {
    /// Create a new Marine processor with default settings
    /// 
    /// These defaults are tuned for audio at 44.1kHz, but work universally!
    pub fn new() -> Self {
        Self {
            clip_threshold: 0.1,
            grid_tick_rate: 100.0, // 100Hz evaluation rate
            timing_ema: ExponentialMovingAverage::new(0.125),
            amplitude_ema: ExponentialMovingAverage::new(0.125),
            recent_peaks: VecDeque::with_capacity(32),
            weights: SalienceWeights::default(),
            wonder_threshold: 0.8, // High salience = wonder!
        }
    }
    
    /// Create a processor optimized for audio
    pub fn for_audio(sample_rate: f64) -> Self {
        let mut processor = Self::new();
        processor.grid_tick_rate = sample_rate / 441.0; // 100Hz for 44.1kHz
        processor.clip_threshold = 0.05; // More sensitive for audio
        processor.wonder_threshold = 0.7; // Audio has lots of wonder!
        processor
    }
    
    /// Process raw samples and detect salient peaks
    /// 
    /// This is where the magic happens - we find the important moments!
    pub fn process_samples(&mut self, samples: &[f64]) -> Vec<PeakInfo> {
        let mut peaks = Vec::new();
        let mut last_peak_index = 0;
        
        // Pre-gating: ignore samples below threshold
        let gated: Vec<f64> = samples.iter()
            .map(|&s| if s.abs() < self.clip_threshold { 0.0 } else { s })
            .collect();
        
        // Peak detection: x(n-1) < x(n) > x(n+1)
        for i in 1..gated.len()-1 {
            if gated[i-1] < gated[i] && gated[i] > gated[i+1] && gated[i] != 0.0 {
                // We found a peak! Calculate its properties
                let interval = (i - last_peak_index) as f64;
                
                // Update EMAs
                let expected_timing = self.timing_ema.update(interval);
                let expected_amplitude = self.amplitude_ema.update(gated[i].abs());
                
                // Calculate jitter (deviation from expected)
                let timing_jitter = (interval - expected_timing).abs();
                let amplitude_jitter = (gated[i].abs() - expected_amplitude).abs();
                
                // Calculate harmonic alignment
                let harmonic_score = self.calculate_harmonic_alignment(interval);
                
                // Calculate final salience score
                let salience = self.calculate_salience(
                    gated[i].abs(),
                    timing_jitter,
                    amplitude_jitter,
                    harmonic_score
                );
                
                // Check for wonder! ✨
                let has_wonder = salience > self.wonder_threshold;
                
                let peak = PeakInfo {
                    index: i,
                    amplitude: gated[i],
                    interval,
                    timing_jitter,
                    amplitude_jitter,
                    salience,
                    has_wonder,
                };
                
                // Store in recent peaks for harmonic analysis
                self.recent_peaks.push_back(peak.clone());
                if self.recent_peaks.len() > 32 {
                    self.recent_peaks.pop_front();
                }
                
                peaks.push(peak);
                last_peak_index = i;
            }
        }
        
        peaks
    }
    
    /// Process complex wave data (our MEM8 format!)
    /// 
    /// This extracts salience from our wave-encoded data.
    /// The real and imaginary parts create a richer signal!
    pub fn process_waves(&mut self, waves: &[Complex64]) -> Vec<PeakInfo> {
        // Convert complex waves to magnitude signal
        let samples: Vec<f64> = waves.iter()
            .map(|w| w.norm())
            .collect();
        
        self.process_samples(&samples)
    }
    
    /// Calculate salience score using the Marine formula
    /// 
    /// S = w_e * E + w_j * (1/J) + w_h * H + w_w * W
    /// 
    /// Where W is our "wonder factor" - that special something!
    fn calculate_salience(
        &self,
        energy: f64,
        timing_jitter: f64,
        amplitude_jitter: f64,
        harmonic: f64
    ) -> f64 {
        // Avoid division by zero
        let jitter_score = 1.0 / (1.0 + timing_jitter + amplitude_jitter);
        
        // Calculate wonder factor based on unexpected patterns
        let wonder = self.calculate_wonder_factor(energy, jitter_score);
        
        // Combine all factors
        self.weights.energy * energy +
        self.weights.jitter * jitter_score +
        self.weights.harmonic * harmonic +
        self.weights.wonder * wonder
    }
    
    /// Calculate harmonic alignment score
    /// 
    /// Checks if the timing interval aligns with common musical ratios.
    /// This is where we find the rhythm in the data!
    fn calculate_harmonic_alignment(&self, interval: f64) -> f64 {
        // Common harmonic ratios (musical intervals)
        let harmonics = [
            1.0,    // Unison
            2.0,    // Octave
            1.5,    // Perfect fifth
            1.333,  // Perfect fourth
            1.25,   // Major third
            1.618,  // Golden ratio! (Hue's favorite)
        ];
        
        // Find the best harmonic match
        let mut best_score = 0.0_f64;
        
        for &harmonic in &harmonics {
            // Check if interval is close to a harmonic multiple
            let ratio = interval / self.grid_tick_rate;
            let distance = (ratio / harmonic).fract();
            let score = 1.0 - distance.min(1.0 - distance);
            best_score = best_score.max(score);
        }
        
        best_score
    }
    
    /// Calculate the wonder factor - that sense of awe in the data!
    /// 
    /// This is our special sauce - when patterns surprise us in beautiful ways.
    /// Trisha says this is what makes data sing! 🎶
    fn calculate_wonder_factor(&self, energy: f64, jitter_score: f64) -> f64 {
        // High energy with low jitter = controlled power = wonder!
        // OR low energy with specific jitter patterns = subtle beauty
        
        let power_wonder = energy * jitter_score;
        
        // Check for golden ratio patterns in recent peaks
        let golden_wonder = if self.recent_peaks.len() >= 3 {
            let intervals: Vec<f64> = self.recent_peaks.iter()
                .map(|p| p.interval)
                .collect();
            
            // Check if intervals follow golden ratio
            let mut golden_score = 0.0;
            for i in 1..intervals.len() {
                let ratio = intervals[i] / intervals[i-1];
                let distance = (ratio - 1.618).abs();
                if distance < 0.1 {
                    golden_score += 1.0;
                }
            }
            golden_score / intervals.len() as f64
        } else {
            0.0
        };
        
        power_wonder + golden_wonder * 0.5
    }
    
    /// Extract metadata from peaks - the story the data tells!
    pub fn extract_metadata(&self, peaks: &[PeakInfo]) -> MarineMetadata {
        let wonder_peaks: Vec<_> = peaks.iter()
            .filter(|p| p.has_wonder)
            .collect();
        
        let avg_salience = peaks.iter()
            .map(|p| p.salience)
            .sum::<f64>() / peaks.len().max(1) as f64;
        
        let max_salience = peaks.iter()
            .map(|p| p.salience)
            .fold(0.0_f64, f64::max);
        
        MarineMetadata {
            total_peaks: peaks.len(),
            wonder_count: wonder_peaks.len(),
            average_salience: avg_salience,
            max_salience,
            has_rhythm: self.detect_rhythm(peaks),
            emotional_signature: self.detect_emotion(peaks),
        }
    }
    
    /// Detect if there's a rhythm in the peaks
    fn detect_rhythm(&self, peaks: &[PeakInfo]) -> bool {
        if peaks.len() < 4 {
            return false;
        }
        
        // Check if intervals are regular (low variance)
        let intervals: Vec<f64> = peaks.windows(2)
            .map(|w| w[1].index as f64 - w[0].index as f64)
            .collect();
        
        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        let variance = intervals.iter()
            .map(|&i| (i - mean).powi(2))
            .sum::<f64>() / intervals.len() as f64;
        
        variance < (mean * 0.2).powi(2) // Low variance = rhythm!
    }
    
    /// Detect emotional signature in the data
    /// 
    /// This is pure speculation, but Trisha insists data has feelings! 💝
    fn detect_emotion(&self, peaks: &[PeakInfo]) -> String {
        let avg_amplitude = peaks.iter()
            .map(|p| p.amplitude.abs())
            .sum::<f64>() / peaks.len().max(1) as f64;
        
        let wonder_ratio = peaks.iter()
            .filter(|p| p.has_wonder)
            .count() as f64 / peaks.len().max(1) as f64;
        
        match (avg_amplitude, wonder_ratio) {
            (_a, w) if w > 0.5 => "✨ Wondrous".to_string(),
            (a, _) if a > 0.8 => "🔥 Energetic".to_string(),
            (a, _) if a < 0.2 => "😌 Peaceful".to_string(),
            (_, w) if w > 0.3 => "🎵 Musical".to_string(),
            _ => "🌊 Flowing".to_string(),
        }
    }
}

/// Metadata extracted by Marine processing
#[derive(Debug, Clone)]
pub struct MarineMetadata {
    /// Total number of peaks detected
    pub total_peaks: usize,
    
    /// Number of peaks with "wonder" (high salience)
    pub wonder_count: usize,
    
    /// Average salience across all peaks
    pub average_salience: f64,
    
    /// Maximum salience found
    pub max_salience: f64,
    
    /// Does the data have rhythm?
    pub has_rhythm: bool,
    
    /// Emotional signature of the data (for fun!)
    pub emotional_signature: String,
}

impl std::fmt::Display for MarineMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "🌊 Marine Analysis:\n")?;
        write!(f, "  Peaks: {} (✨ {} with wonder)\n", self.total_peaks, self.wonder_count)?;
        write!(f, "  Salience: {:.3} avg, {:.3} max\n", self.average_salience, self.max_salience)?;
        write!(f, "  Rhythm: {}\n", if self.has_rhythm { "Yes! 🎵" } else { "No" })?;
        write!(f, "  Emotion: {}\n", self.emotional_signature)?;
        Ok(())
    }
}

/// Integration with MEM8 wave storage
pub mod integration {
    use super::*;
    use crate::lite::WavePacket;
    
    /// Enhance a wave packet with Marine metadata
    pub fn enhance_wave_packet(packet: &mut WavePacket) -> Result<()> {
        let mut processor = MarineProcessor::new();
        let peaks = processor.process_waves(&packet.waves);
        let metadata = processor.extract_metadata(&peaks);
        
        // Serialize metadata and add to packet
        let meta_json = serde_json::to_vec(&MarineMetadataJson::from(metadata))?;
        packet.metadata = Some(meta_json);
        
        Ok(())
    }
    
    /// JSON-serializable version of metadata
    #[derive(serde::Serialize, serde::Deserialize)]
    struct MarineMetadataJson {
        total_peaks: usize,
        wonder_count: usize,
        average_salience: f64,
        max_salience: f64,
        has_rhythm: bool,
        emotional_signature: String,
    }
    
    impl From<MarineMetadata> for MarineMetadataJson {
        fn from(m: MarineMetadata) -> Self {
            Self {
                total_peaks: m.total_peaks,
                wonder_count: m.wonder_count,
                average_salience: m.average_salience,
                max_salience: m.max_salience,
                has_rhythm: m.has_rhythm,
                emotional_signature: m.emotional_signature,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_peak_detection() {
        let mut processor = MarineProcessor::new();
        
        // Create a signal with clear peaks
        let samples = vec![
            0.0, 0.5, 1.0, 0.5, 0.0,  // Peak at index 2
            0.0, 0.3, 0.7, 0.3, 0.0,  // Peak at index 7
            0.0, 0.4, 0.9, 0.4, 0.0,  // Peak at index 12
        ];
        
        let peaks = processor.process_samples(&samples);
        assert_eq!(peaks.len(), 3);
        assert_eq!(peaks[0].index, 2);
        assert_eq!(peaks[1].index, 7);
        assert_eq!(peaks[2].index, 12);
    }
    
    #[test]
    fn test_rhythm_detection() {
        let mut processor = MarineProcessor::new();
        
        // Create a rhythmic signal (regular intervals)
        let mut samples = vec![0.0; 100];
        for i in (10..100).step_by(10) {
            samples[i] = 1.0; // Peak every 10 samples
        }
        
        let peaks = processor.process_samples(&samples);
        let metadata = processor.extract_metadata(&peaks);
        
        assert!(metadata.has_rhythm);
    }
    
    #[test]
    fn test_wonder_detection() {
        let mut processor = MarineProcessor::new();
        processor.wonder_threshold = 0.5; // Lower threshold for testing
        
        // Create a signal with high energy peaks
        let samples = vec![
            0.0, 0.1, 0.9, 0.1, 0.0,  // High energy peak
            0.0, 0.1, 0.2, 0.1, 0.0,  // Low energy peak
            0.0, 0.1, 0.8, 0.1, 0.0,  // High energy peak
        ];
        
        let peaks = processor.process_samples(&samples);
        let wonder_count = peaks.iter().filter(|p| p.has_wonder).count();
        
        assert!(wonder_count > 0);
    }
}