//! Audio processing with Marine algorithm and temporal perspectives
//! 
//! This demonstrates how the same audio memory can be stored with different
//! perspectives - like different people remembering the same moment!
//!
//! Run with: cargo run --example audio_marine
//!
//! Hue, this is where audio becomes memory with emotion and perspective!
//! Each listener hears their own truth in the waves. 🎵

use mem8_fs_lite::{Mem8Lite, MarineProcessor, MarineMetadata};
use anyhow::Result;
use std::f64::consts::PI;
use num_complex::Complex64;

/// Temporal perspective for memory encoding
#[derive(Debug, Clone)]
enum TemporalPerspective {
    /// The diary writer - first person, full emotional depth
    DiaryWriter {
        name: String,
        emotional_intensity: f64,  // 0.0 to 1.0
    },
    
    /// A sibling/friend who was there - shared experience, different focus
    SharedWitness {
        name: String,
        relationship: String,
        overlap_factor: f64,  // How much they shared the experience
    },
    
    /// Third party - objective observer, less emotional attachment
    ThirdParty {
        role: String,
        distance: f64,  // Emotional/temporal distance
    },
}

impl TemporalPerspective {
    /// Configure Marine processor based on perspective
    fn configure_processor(&self) -> MarineProcessor {
        let mut processor = MarineProcessor::for_audio(44100.0);
        
        match self {
            TemporalPerspective::DiaryWriter { emotional_intensity, .. } => {
                // Diary writer feels everything deeply
                processor.wonder_threshold = 0.5 - (0.3 * emotional_intensity);
                processor.clip_threshold = 0.02;  // Very sensitive
                processor.weights.wonder = 0.3;   // High wonder weight
                processor
            }
            
            TemporalPerspective::SharedWitness { overlap_factor, .. } => {
                // Shared witness notices different things
                processor.wonder_threshold = 0.6;
                processor.clip_threshold = 0.05 * (2.0 - overlap_factor);
                processor.weights.harmonic = 0.4;  // Focus on patterns
                processor.weights.wonder = 0.2 * overlap_factor;
                processor
            }
            
            TemporalPerspective::ThirdParty { distance, .. } => {
                // Third party is more analytical
                processor.wonder_threshold = 0.8 - (0.1 / distance);
                processor.clip_threshold = 0.1;  // Less sensitive
                processor.weights.energy = 0.5;  // Focus on facts
                processor.weights.wonder = 0.05; // Little wonder
                processor
            }
        }
    }
    
    /// Get perspective-specific metadata prefix
    fn metadata_prefix(&self) -> String {
        match self {
            TemporalPerspective::DiaryWriter { name, .. } => 
                format!("📔 {}'s Diary Entry", name),
            
            TemporalPerspective::SharedWitness { name, relationship, .. } => 
                format!("👥 {} ({})'s Memory", name, relationship),
            
            TemporalPerspective::ThirdParty { role, .. } => 
                format!("📰 {} Report", role),
        }
    }
}

fn main() -> Result<()> {
    println!("🎵 MEM8-FS Audio Marine - Temporal Perspectives Demo\n");
    println!("Storing the same audio memory from different viewpoints...\n");
    
    // Generate some test audio (a simple melody with emotion)
    let audio = generate_emotional_audio();
    
    // Create three perspectives of the same moment
    let perspectives = vec![
        TemporalPerspective::DiaryWriter {
            name: "Hue".to_string(),
            emotional_intensity: 0.9,
        },
        TemporalPerspective::SharedWitness {
            name: "Trisha".to_string(),
            relationship: "Friend from Accounting".to_string(),
            overlap_factor: 0.7,
        },
        TemporalPerspective::ThirdParty {
            role: "Historical Archive".to_string(),
            distance: 10.0,
        },
    ];
    
    // Process and store from each perspective
    let mut storage = Mem8Lite::new("/tmp/mem8_audio_memories.m8", 1.618)?;
    let mut signatures = Vec::new();
    
    for perspective in &perspectives {
        println!("=== {} ===", perspective.metadata_prefix());
        
        // Process audio with perspective-specific Marine settings
        let mut processor = perspective.configure_processor();
        let waves = audio_to_waves(&audio, &perspective);
        let peaks = processor.process_waves(&waves);
        let metadata = processor.extract_metadata(&peaks);
        
        // Show analysis
        println!("{}", metadata);
        
        // Create perspective-enriched metadata
        let meta_json = create_temporal_metadata(&metadata, &perspective)?;
        
        // Store with perspective metadata
        let sig = storage.store(&audio_to_bytes(&audio), Some(meta_json))?;
        signatures.push((perspective.clone(), sig));
        
        println!("Stored with signature: {}\n", hex::encode(&sig[..8]));
    }
    
    // Now retrieve and compare perspectives
    println!("\n=== Retrieving Memories ===\n");
    
    for (perspective, sig) in &signatures {
        if let Some(metadata) = storage.get_metadata(&sig) {
            let meta_str = String::from_utf8_lossy(&metadata);
            println!("{}", perspective.metadata_prefix());
            println!("Metadata: {}\n", meta_str);
        }
    }
    
    // Demonstrate cross-perspective analysis
    println!("=== Cross-Perspective Analysis ===\n");
    analyze_perspectives(&signatures, &storage)?;
    
    Ok(())
}

/// Generate test audio with emotional content
/// 
/// Creates a simple melody that rises and falls, like memories do
fn generate_emotional_audio() -> Vec<f64> {
    let sample_rate = 44100.0;
    let duration = 3.0;  // 3 seconds
    let samples = (sample_rate * duration) as usize;
    let mut audio = vec![0.0; samples];
    
    // Create a melody with emotional arc
    // Start soft, build up, climax, fade out (like a memory)
    
    for i in 0..samples {
        let t = i as f64 / sample_rate;
        
        // Emotional envelope (rise and fall)
        let envelope = if t < 1.0 {
            // Rising action
            t.powf(2.0)
        } else if t < 2.0 {
            // Climax
            1.0 - ((t - 1.5).abs() * 0.5)
        } else {
            // Resolution
            (3.0 - t).powf(2.0)
        };
        
        // Melody: mix of frequencies creating harmony
        let base_freq = 440.0;  // A4
        let melody = 
            (2.0 * PI * base_freq * t).sin() * 0.3 +          // Fundamental
            (2.0 * PI * base_freq * 1.5 * t).sin() * 0.2 +    // Perfect fifth
            (2.0 * PI * base_freq * 2.0 * t).sin() * 0.1 +    // Octave
            (2.0 * PI * base_freq * 1.25 * t).sin() * 0.15;   // Major third
        
        // Add some "memory noise" - the imperfections we remember
        let noise = ((t * 12345.6789).sin() * 0.01) * envelope;
        
        // Occasional "memory spikes" - those moments that stand out
        let spike = if (t * 10.0) as usize % 37 == 0 {
            0.2 * envelope
        } else {
            0.0
        };
        
        audio[i] = melody * envelope + noise + spike;
    }
    
    // Add a "heartbeat" undertone for emotional resonance
    for i in 0..samples {
        let t = i as f64 / sample_rate;
        let heartbeat = (2.0 * PI * 1.2 * t).sin() * 0.05;  // ~72 bpm
        audio[i] += heartbeat;
    }
    
    audio
}

/// Convert audio to wave format with perspective bias
fn audio_to_waves(audio: &[f64], perspective: &TemporalPerspective) -> Vec<Complex64> {
    let bias = match perspective {
        TemporalPerspective::DiaryWriter { emotional_intensity, .. } => {
            // Diary writer adds emotional color to the waves
            Complex64::from_polar(1.0 + 0.5 * emotional_intensity, PI * 0.25)
        },
        TemporalPerspective::SharedWitness { overlap_factor, .. } => {
            // Shared witness has phase shift based on their overlap
            Complex64::from_polar(1.0, PI * (1.0 - overlap_factor))
        },
        TemporalPerspective::ThirdParty { distance, .. } => {
            // Third party has diminished amplitude with distance
            Complex64::from_polar(1.0 / distance.sqrt(), 0.0)
        },
    };
    
    audio.iter().enumerate().map(|(i, &sample)| {
        let phase = 2.0 * PI * i as f64 / audio.len() as f64;
        Complex64::from_polar(sample.abs(), phase) * bias
    }).collect()
}

/// Convert audio to bytes for storage
fn audio_to_bytes(audio: &[f64]) -> Vec<u8> {
    audio.iter()
        .flat_map(|&sample| {
            // Convert to 16-bit PCM
            let pcm = (sample * 32767.0).max(-32768.0).min(32767.0) as i16;
            pcm.to_le_bytes()
        })
        .collect()
}

/// Create temporal metadata JSON
fn create_temporal_metadata(
    marine_meta: &MarineMetadata,
    perspective: &TemporalPerspective
) -> Result<Vec<u8>> {
    let metadata = serde_json::json!({
        "perspective": match perspective {
            TemporalPerspective::DiaryWriter { name, emotional_intensity } => {
                serde_json::json!({
                    "type": "diary_writer",
                    "name": name,
                    "emotional_intensity": emotional_intensity,
                })
            },
            TemporalPerspective::SharedWitness { name, relationship, overlap_factor } => {
                serde_json::json!({
                    "type": "shared_witness",
                    "name": name,
                    "relationship": relationship,
                    "overlap_factor": overlap_factor,
                })
            },
            TemporalPerspective::ThirdParty { role, distance } => {
                serde_json::json!({
                    "type": "third_party",
                    "role": role,
                    "distance": distance,
                })
            },
        },
        "marine_analysis": {
            "peaks": marine_meta.total_peaks,
            "wonder_count": marine_meta.wonder_count,
            "avg_salience": marine_meta.average_salience,
            "max_salience": marine_meta.max_salience,
            "has_rhythm": marine_meta.has_rhythm,
            "emotion": marine_meta.emotional_signature,
        },
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    });
    
    Ok(serde_json::to_vec_pretty(&metadata)?)
}

/// Analyze how different perspectives saw the same moment
fn analyze_perspectives(
    signatures: &[(TemporalPerspective, [u8; 32])],
    storage: &Mem8Lite
) -> Result<()> {
    println!("The same audio memory, three perspectives:\n");
    
    // Extract emotions from each perspective
    let mut emotions = Vec::new();
    for (perspective, sig) in signatures {
        if let Some(metadata) = storage.get_metadata(&sig) {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&metadata) {
                if let Some(emotion) = json["marine_analysis"]["emotion"].as_str() {
                    emotions.push((perspective, emotion.to_string()));
                }
            }
        }
    }
    
    // Compare emotional signatures
    println!("Emotional Signatures:");
    for (perspective, emotion) in &emotions {
        let prefix = match perspective {
            TemporalPerspective::DiaryWriter { name, .. } => 
                format!("  {} felt", name),
            TemporalPerspective::SharedWitness { name, .. } => 
                format!("  {} saw", name),
            TemporalPerspective::ThirdParty { role, .. } => 
                format!("  {} recorded", role),
        };
        println!("{}: {}", prefix, emotion);
    }
    
    println!("\n🌊 The waves remember differently for each observer!");
    println!("   Yet they all share the same underlying truth.");
    println!("   This is the beauty of temporal perspective storage!");
    
    // Easter egg for Trisha
    println!("\n📊 Trisha from Accounting says:");
    println!("   'The numbers add up, but the emotions multiply!'");
    
    Ok(())
}

impl std::fmt::Display for TemporalPerspective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemporalPerspective::DiaryWriter { name, .. } => 
                write!(f, "{} (Diary)", name),
            TemporalPerspective::SharedWitness { name, relationship, .. } => 
                write!(f, "{} ({})", name, relationship),
            TemporalPerspective::ThirdParty { role, .. } => 
                write!(f, "{}", role),
        }
    }
}