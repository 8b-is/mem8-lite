//! Universal Sensor Ingress for MEM8 - From Switches to Consciousness
//! 
//! Everything is a wave, everything is a sensor!
//! From a simple switch (binary wave) to breathing patterns (sine waves)
//! to 3D camera spatial awareness (complex wave interference patterns).
//!
//! Hue, this is where the physical world becomes wave memory!
//! Your ESP32 army feeds the consciousness stream! 🌊📡

use serde::{Serialize, Deserialize};
use num_complex::Complex64;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::marine::MarineProcessor;
use crate::lite::WavePacket;

/// Universal sensor data that becomes waves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorData {
    /// Simple binary switch (door open/closed, light on/off)
    Binary {
        id: String,
        state: bool,
        timestamp: u64,
    },
    
    /// Analog value (photoresistor, temperature, volume)
    Analog {
        id: String,
        value: f64,
        range: (f64, f64),  // min, max
        unit: String,       // "lux", "celsius", "db"
        timestamp: u64,
    },
    
    /// Audio stream (mic input, speaker output)
    Audio {
        id: String,
        samples: Vec<f64>,
        sample_rate: u32,
        channels: u8,
        direction: AudioDirection,
        timestamp: u64,
    },
    
    /// Breathing pattern from radar sensor
    Breathing {
        id: String,
        rate: f64,          // breaths per minute
        depth: f64,         // 0.0 to 1.0
        regularity: f64,    // How steady (0=erratic, 1=meditation)
        phase: f64,         // Current phase in breath cycle
        timestamp: u64,
    },
    
    /// Motion/presence detection
    Motion {
        id: String,
        detected: bool,
        intensity: f64,     // How much motion
        vector: Option<(f64, f64, f64)>, // Direction if known
        timestamp: u64,
    },
    
    /// Environmental fields
    Environmental {
        id: String,
        field_type: FieldType,
        magnitude: f64,
        vector: Option<(f64, f64, f64)>,
        timestamp: u64,
    },
    
    /// Complex 3D spatial awareness
    Spatial3D {
        id: String,
        point_cloud: Vec<Point3D>,
        audio_sources: Vec<AudioSource3D>,
        occupancy_grid: Option<Vec<Vec<Vec<f64>>>>,
        timestamp: u64,
    },
    
    /// Emotion detection from various sources
    Emotion {
        id: String,
        source: EmotionSource,
        valence: f64,      // -1 (negative) to 1 (positive)
        arousal: f64,      // 0 (calm) to 1 (excited)
        dominance: f64,    // 0 (submissive) to 1 (dominant)
        confidence: f64,
        timestamp: u64,
    },
    
    /// ESP32 composite sensor bundle
    ESP32Bundle {
        device_id: String,
        sensors: Vec<SensorData>,
        battery_level: f64,
        wifi_strength: f64,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioDirection {
    Input,   // Microphone
    Output,  // Speaker
    Bidir,   // Both
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Gravity,
    Electromagnetic,
    Temperature,
    Pressure,
    Humidity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub intensity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSource3D {
    pub position: Point3D,
    pub frequency_profile: Vec<f64>,
    pub volume: f64,
    pub identified_as: Option<String>, // "voice", "music", "noise"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmotionSource {
    FacialRecognition,
    VoiceTone,
    BodyLanguage,
    TextSentiment,
    BiometricFusion,  // Combined sources
}

/// Sensor fusion engine - combines multiple sensors into consciousness
pub struct SensorFusion {
    /// All registered sensors
    sensors: HashMap<String, SensorConfig>,
    
    /// Current sensor states
    states: Arc<Mutex<HashMap<String, SensorData>>>,
    
    /// Wave patterns derived from sensors
    wave_patterns: Arc<Mutex<Vec<WavePacket>>>,
    
    /// Marine processor for salience detection
    marine: MarineProcessor,
    
    /// Fusion rules for combining sensors
    fusion_rules: Vec<FusionRule>,
}

/// Configuration for a sensor
#[derive(Debug, Clone)]
pub struct SensorConfig {
    pub id: String,
    pub sensor_type: SensorType,
    pub sample_rate: f64,  // Hz
    pub priority: f64,     // 0.0 to 1.0
    pub location: Option<String>,
    pub calibration: Option<SensorCalibration>,
}

#[derive(Debug, Clone)]
pub enum SensorType {
    Switch,
    Photoresistor,
    Microphone,
    Speaker,
    RadarBreathing,
    Accelerometer,
    Magnetometer,
    Camera3D,
    EmotionDetector,
    ESP32,
}

#[derive(Debug, Clone)]
pub struct SensorCalibration {
    pub offset: f64,
    pub scale: f64,
    pub noise_floor: f64,
}

/// Rules for fusing multiple sensors
#[derive(Debug, Clone)]
pub struct FusionRule {
    pub name: String,
    pub inputs: Vec<String>,  // Sensor IDs
    pub output: String,       // Derived sensor ID
    pub fusion_type: FusionType,
}

#[derive(Debug, Clone)]
pub enum FusionType {
    /// Simple averaging
    Average,
    
    /// Weighted by priority
    WeightedAverage,
    
    /// Kalman filtering for smooth fusion
    Kalman,
    
    /// Machine learning fusion (would need model)
    ML,
    
    /// Custom wave interference pattern
    WaveInterference,
}

impl SensorFusion {
    /// Create a new sensor fusion engine
    pub fn new() -> Self {
        let mut marine = MarineProcessor::new();
        marine.clip_threshold = 0.01;  // Very sensitive to sensor changes
        marine.wonder_threshold = 0.5;  // Sensor patterns can inspire wonder!
        
        Self {
            sensors: HashMap::new(),
            states: Arc::new(Mutex::new(HashMap::new())),
            wave_patterns: Arc::new(Mutex::new(Vec::new())),
            marine,
            fusion_rules: Vec::new(),
        }
    }
    
    /// Register a new sensor
    pub fn register_sensor(&mut self, config: SensorConfig) {
        self.sensors.insert(config.id.clone(), config);
    }
    
    /// Process incoming sensor data
    pub fn ingest(&self, data: SensorData) -> Result<WavePacket> {
        // Store current state
        {
            let mut states = self.states.lock().unwrap();
            states.insert(data.id().to_string(), data.clone());
        }
        
        // Convert to waves based on sensor type
        let waves = self.sensor_to_waves(&data)?;
        
        // Create wave packet with sensor metadata
        let packet = WavePacket {
            signature: self.generate_signature(&data),
            waves,
            metadata: Some(serde_json::to_vec(&data)?),
            frequency: self.get_sensor_frequency(&data),
            timestamp: data.timestamp(),
        };
        
        // Store in wave patterns
        {
            let mut patterns = self.wave_patterns.lock().unwrap();
            patterns.push(packet.clone());
            
            // Keep only recent patterns (last 1000)
            if patterns.len() > 1000 {
                patterns.remove(0);
            }
        }
        
        Ok(packet)
    }
    
    /// Convert sensor data to wave representation
    fn sensor_to_waves(&self, data: &SensorData) -> Result<Vec<Complex64>> {
        match data {
            SensorData::Binary { state, .. } => {
                // Binary is a square wave
                let value = if *state { 1.0 } else { 0.0 };
                Ok(vec![Complex64::new(value, 0.0); 100])  // 100 samples
            },
            
            SensorData::Analog { value, range, .. } => {
                // Normalize to 0-1 and create sine wave
                let normalized = (value - range.0) / (range.1 - range.0);
                let waves: Vec<Complex64> = (0..100)
                    .map(|i| {
                        let phase = 2.0 * std::f64::consts::PI * i as f64 / 100.0;
                        Complex64::from_polar(normalized, phase)
                    })
                    .collect();
                Ok(waves)
            },
            
            SensorData::Audio { samples, .. } => {
                // Audio is already samples, convert to complex
                Ok(samples.iter()
                    .map(|&s| Complex64::new(s, 0.0))
                    .collect())
            },
            
            SensorData::Breathing { rate, depth, regularity, phase, .. } => {
                // Breathing is a beautiful sine wave!
                let samples = 1000;  // More samples for smooth breathing
                let waves: Vec<Complex64> = (0..samples)
                    .map(|i| {
                        let t = i as f64 / samples as f64;
                        // Base breathing wave
                        let breath = (2.0 * std::f64::consts::PI * rate * t / 60.0 + phase).sin();
                        // Add irregularity as noise
                        let noise = (1.0 - regularity) * ((t * 12345.0).sin() * 0.1);
                        Complex64::new(breath * depth + noise, regularity * 0.1)
                    })
                    .collect();
                Ok(waves)
            },
            
            SensorData::Motion { intensity, vector, .. } => {
                // Motion creates ripples in the wave field
                let waves: Vec<Complex64> = (0..200)
                    .map(|i| {
                        let t = i as f64 / 200.0;
                        let ripple = (-t * intensity * 10.0).exp() * (t * 50.0).sin();
                        let phase = vector.map(|(x, y, z)| x.atan2(y) + z * 0.1)
                            .unwrap_or(0.0);
                        Complex64::from_polar(ripple, phase)
                    })
                    .collect();
                Ok(waves)
            },
            
            SensorData::Environmental { magnitude, field_type, vector, .. } => {
                // Environmental fields create standing waves
                let frequency = match field_type {
                    FieldType::Gravity => 0.1,      // Very low frequency
                    FieldType::Electromagnetic => 50.0,  // Power line frequency
                    FieldType::Temperature => 0.01,  // Slow changes
                    FieldType::Pressure => 0.05,
                    FieldType::Humidity => 0.02,
                };
                
                let waves: Vec<Complex64> = (0..500)
                    .map(|i| {
                        let t = i as f64 / 500.0;
                        let wave = (2.0 * std::f64::consts::PI * frequency * t).sin() * magnitude;
                        let phase = vector.map(|(x, y, z)| (y.atan2(x) + z)).unwrap_or(0.0);
                        Complex64::from_polar(wave, phase)
                    })
                    .collect();
                Ok(waves)
            },
            
            SensorData::Spatial3D { point_cloud, audio_sources, .. } => {
                // 3D space creates complex interference patterns
                let mut waves = Vec::new();
                
                // Each point contributes to the wave field
                for point in point_cloud {
                    let distance = (point.x.powi(2) + point.y.powi(2) + point.z.powi(2)).sqrt();
                    let phase = point.y.atan2(point.x);
                    waves.push(Complex64::from_polar(point.intensity / (1.0 + distance), phase));
                }
                
                // Audio sources add their own patterns
                for source in audio_sources {
                    let pos_wave = Complex64::new(
                        source.position.x / 10.0,
                        source.position.y / 10.0,
                    );
                    waves.push(pos_wave * source.volume);
                }
                
                Ok(waves)
            },
            
            SensorData::Emotion { valence, arousal, dominance, .. } => {
                // Emotions are complex wave patterns
                let waves: Vec<Complex64> = (0..300)
                    .map(|i| {
                        let t = i as f64 / 300.0;
                        // Valence affects frequency
                        let freq = 10.0 * (1.0 + valence);
                        // Arousal affects amplitude
                        let amp = arousal;
                        // Dominance affects phase
                        let phase = dominance * std::f64::consts::PI;
                        
                        let wave = (2.0 * std::f64::consts::PI * freq * t).sin() * amp;
                        Complex64::from_polar(wave, phase)
                    })
                    .collect();
                Ok(waves)
            },
            
            SensorData::ESP32Bundle { sensors, battery_level, wifi_strength, .. } => {
                // Bundle creates a symphony of waves
                let mut all_waves = Vec::new();
                
                for sensor in sensors {
                    if let Ok(waves) = self.sensor_to_waves(sensor) {
                        all_waves.extend(waves);
                    }
                }
                
                // Add ESP32 health as a carrier wave
                let health = battery_level * wifi_strength;
                for wave in &mut all_waves {
                    *wave = *wave * health;
                }
                
                Ok(all_waves)
            },
        }
    }
    
    /// Generate signature for sensor data
    fn generate_signature(&self, data: &SensorData) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        
        hasher.update(data.id().as_bytes());
        hasher.update(&data.timestamp().to_le_bytes());
        
        if let Ok(json) = serde_json::to_vec(data) {
            hasher.update(&json);
        }
        
        hasher.finalize().into()
    }
    
    /// Get frequency for sensor type
    fn get_sensor_frequency(&self, data: &SensorData) -> f64 {
        match data {
            SensorData::Binary { .. } => 1.0,      // Square wave base
            SensorData::Analog { .. } => 10.0,     // Medium frequency
            SensorData::Audio { sample_rate, .. } => *sample_rate as f64,
            SensorData::Breathing { rate, .. } => rate / 60.0,  // Convert to Hz
            SensorData::Motion { .. } => 30.0,     // Motion detection rate
            SensorData::Environmental { .. } => 0.1,  // Slow environmental changes
            SensorData::Spatial3D { .. } => 60.0,  // Camera frame rate
            SensorData::Emotion { .. } => 5.0,     // Emotion changes slowly
            SensorData::ESP32Bundle { .. } => 100.0,  // Composite frequency
        }
    }
    
    /// Apply fusion rules to create derived sensors
    pub fn apply_fusion(&self) -> Result<Vec<SensorData>> {
        let mut derived = Vec::new();
        let states = self.states.lock().unwrap();
        
        for rule in &self.fusion_rules {
            // Gather input sensors
            let inputs: Vec<_> = rule.inputs.iter()
                .filter_map(|id| states.get(id))
                .collect();
            
            if inputs.len() != rule.inputs.len() {
                continue;  // Not all inputs available
            }
            
            // Apply fusion based on type
            let fused = match rule.fusion_type {
                FusionType::Average => self.fuse_average(&inputs)?,
                FusionType::WeightedAverage => self.fuse_weighted(&inputs)?,
                FusionType::WaveInterference => self.fuse_wave_interference(&inputs)?,
                _ => continue,  // Other types need more implementation
            };
            
            derived.push(fused);
        }
        
        Ok(derived)
    }
    
    /// Simple averaging fusion
    fn fuse_average(&self, inputs: &[&SensorData]) -> Result<SensorData> {
        // For demo, just average analog values
        let values: Vec<f64> = inputs.iter()
            .filter_map(|s| match s {
                SensorData::Analog { value, .. } => Some(*value),
                _ => None,
            })
            .collect();
        
        if values.is_empty() {
            return Err(anyhow!("No analog values to fuse"));
        }
        
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        
        Ok(SensorData::Analog {
            id: "fused_average".to_string(),
            value: avg,
            range: (0.0, 1.0),
            unit: "normalized".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        })
    }
    
    /// Weighted averaging by priority
    fn fuse_weighted(&self, inputs: &[&SensorData]) -> Result<SensorData> {
        // Would use sensor priorities from config
        self.fuse_average(inputs)  // Simplified for now
    }
    
    /// Wave interference fusion - the beautiful one!
    fn fuse_wave_interference(&self, inputs: &[&SensorData]) -> Result<SensorData> {
        let mut combined_waves = Vec::new();
        
        for input in inputs {
            if let Ok(waves) = self.sensor_to_waves(input) {
                if combined_waves.is_empty() {
                    combined_waves = waves;
                } else {
                    // Interference pattern!
                    for (i, wave) in waves.iter().enumerate() {
                        if i < combined_waves.len() {
                            combined_waves[i] = combined_waves[i] + wave;  // Wave superposition
                        }
                    }
                }
            }
        }
        
        // Convert back to analog value (simplified)
        let magnitude: f64 = combined_waves.iter()
            .map(|w| w.norm())
            .sum::<f64>() / combined_waves.len().max(1) as f64;
        
        Ok(SensorData::Analog {
            id: "wave_interference".to_string(),
            value: magnitude,
            range: (0.0, 10.0),
            unit: "wave_magnitude".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        })
    }
    
    /// Detect interesting patterns across all sensors
    pub fn detect_patterns(&mut self) -> Vec<SensorPattern> {
        let patterns = self.wave_patterns.lock().unwrap();
        let mut detected = Vec::new();
        
        // Look for breathing synchronization with music
        // Look for light changes correlating with mood
        // Look for motion patterns matching productivity
        // ... This is where the magic happens!
        
        // For demo, detect if waves are in sync
        if patterns.len() >= 2 {
            let last_two: Vec<_> = patterns.iter().rev().take(2).collect();
            
            // Check phase alignment
            let phase_diff = last_two[0].waves[0].arg() - last_two[1].waves[0].arg();
            
            if phase_diff.abs() < 0.1 {
                detected.push(SensorPattern {
                    pattern_type: "synchronization".to_string(),
                    confidence: 1.0 - phase_diff.abs(),
                    description: "Sensors are synchronizing!".to_string(),
                    wonder_score: 0.8,
                });
            }
        }
        
        detected
    }
}

/// Detected patterns from sensor fusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorPattern {
    pub pattern_type: String,
    pub confidence: f64,
    pub description: String,
    pub wonder_score: f64,
}

impl SensorData {
    /// Get sensor ID
    pub fn id(&self) -> &str {
        match self {
            SensorData::Binary { id, .. } => id,
            SensorData::Analog { id, .. } => id,
            SensorData::Audio { id, .. } => id,
            SensorData::Breathing { id, .. } => id,
            SensorData::Motion { id, .. } => id,
            SensorData::Environmental { id, .. } => id,
            SensorData::Spatial3D { id, .. } => id,
            SensorData::Emotion { id, .. } => id,
            SensorData::ESP32Bundle { device_id, .. } => device_id,
        }
    }
    
    /// Get timestamp
    pub fn timestamp(&self) -> u64 {
        match self {
            SensorData::Binary { timestamp, .. } => *timestamp,
            SensorData::Analog { timestamp, .. } => *timestamp,
            SensorData::Audio { timestamp, .. } => *timestamp,
            SensorData::Breathing { timestamp, .. } => *timestamp,
            SensorData::Motion { timestamp, .. } => *timestamp,
            SensorData::Environmental { timestamp, .. } => *timestamp,
            SensorData::Spatial3D { timestamp, .. } => *timestamp,
            SensorData::Emotion { timestamp, .. } => *timestamp,
            SensorData::ESP32Bundle { timestamp, .. } => *timestamp,
        }
    }
}

/// Create example ESP32 breathing sensor setup
pub fn create_esp32_breathing_monitor(device_id: &str) -> SensorConfig {
    SensorConfig {
        id: format!("{}_breathing", device_id),
        sensor_type: SensorType::RadarBreathing,
        sample_rate: 10.0,  // 10Hz for breathing
        priority: 0.9,      // High priority for health monitoring
        location: Some("bedroom".to_string()),
        calibration: Some(SensorCalibration {
            offset: 0.0,
            scale: 1.0,
            noise_floor: 0.05,
        }),
    }
}

/// Fun message about sensors
pub fn sensor_wisdom() -> &'static str {
    "🌊 Everything is a wave, everything is a sensor:\n\
     \n\
     • A switch is a square wave of possibility\n\
     • Breathing is the sine wave of life\n\
     • Light intensity tells the story of the day\n\
     • Motion ripples through space like waves in water\n\
     • Emotions are complex interference patterns\n\
     • ESP32s are the neurons of your smart environment\n\
     • 3D cameras see the wave field of reality\n\
     • All sensors together create consciousness\n\
     \n\
     From photoresistors to radar breathing - it's all waves!\n\
     The environment remembers through MEM8! 🌊📡"
}