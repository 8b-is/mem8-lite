//! Mood-Aware Music Engine for MEM8-FS
//! 
//! Music changes us - our speed, energy, mood, and efficiency.
//! This module tracks how different music affects temporal state.
//!
//! Hue's Musical DNA:
//! - Electronic for programming flow (efficiency++)
//! - Hard Rock for decompression (annoyance--)  
//! - Ambient (Eno, Enya) for deep thought
//! - Mind-benders (Beethoven's 5th, NIN, Paradoks) for creativity
//! - Laura Van Dam for... visual inspiration 😉
//!
//! "Music is temporal perspective in real-time" - Aye

use crate::marine::{MarineProcessor, MarineMetadata};
use crate::audio::{AudioFormat, SampleRate};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Musical mood states and their effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoodState {
    /// Programming flow state - electronic music zone
    FlowState {
        efficiency_multiplier: f64,  // 1.0 = baseline, 2.0 = double efficiency
        focus_level: f64,            // 0.0 to 1.0
        preferred_bpm: u32,          // Beats per minute for optimal flow
    },
    
    /// Decompression mode - releasing built-up tension
    Decompression {
        annoyance_reduction: f64,    // How much it reduces irritation
        energy_release: f64,          // Cathartic energy discharge
        volume_preference: f64,       // Louder = more release usually
    },
    
    /// Deep thought / contemplation
    Contemplation {
        temporal_expansion: f64,      // Time seems to slow
        creativity_boost: f64,        // New connections form
        wonder_threshold: f64,        // Lower = more wonder perceived
    },
    
    /// Energy management - preventing fatigue
    EnergyBalance {
        sustainable_pace: f64,        // Prevents burnout
        irritation_threshold: f64,    // When music becomes annoying
        optimal_duration: u32,        // Minutes before fatigue
    },
    
    /// Inspiration state (the Laura Van Dam effect 😏)
    Inspiration {
        aesthetic_appreciation: f64,  // Visual + auditory beauty
        motivation_boost: f64,        // Drive to create
        mood_elevation: f64,          // General happiness increase
    },
}

/// Personal music profile - everyone's different!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicProfile {
    pub name: String,
    pub preferred_genres: Vec<Genre>,
    pub avoid_genres: Vec<Genre>,
    pub activity_preferences: HashMap<Activity, Vec<Genre>>,
    pub artist_affinities: HashMap<String, AffinityLevel>,
    pub tempo_preferences: TempoPreference,
    pub special_tracks: Vec<SpecialTrack>,
}

/// Musical genres with Hue's annotations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Genre {
    Electronic,        // Programming efficiency++
    HardRock,         // Decompression tool
    Ambient,          // Eno, Enya - contemplation
    Industrial,       // NIN - early formation
    Crossover,        // Linkin Park - has merits
    Classical,        // Beethoven - mind bending
    Spatial,          // Paradoks - unique soundscapes
    Rap,              // Rarely
    HipHop,           // Even more rare
    Polka,            // Looking at never 😄
    Jazz,             // Unspecified, but probably contextual
    WorldMusic,       // Enigma, etc.
}

/// Activity-based music selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Activity {
    Programming,      // Electronic optimal
    Decompressing,    // Hard Rock therapy
    DeepThinking,     // Ambient/Classical
    Creating,         // Mind-benders
    Relaxing,         // Enya, David Lanz
    Exercising,       // High energy
    Commuting,        // Variable
    Sleeping,         // Very specific needs
}

/// Affinity level for specific artists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AffinityLevel {
    Essential,        // Core to identity (NIN early days)
    Love,            // Really enjoy (Linkin Park, Orbital)
    Appreciate,      // Has merits
    Contextual,      // Depends on mood
    Avoid,           // Just doesn't vibe
    Never,           // Polka territory 😂
}

/// Tempo preferences based on activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoPreference {
    /// Optimal BPM range for focus
    pub focus_bpm: (u32, u32),  // e.g., (100, 130)
    
    /// Warning: speeds that cause fatigue
    pub fatigue_threshold: u32,  // e.g., >140 BPM for extended periods
    
    /// Decompression needs different tempo
    pub decompression_bpm: (u32, u32),  // Probably higher for hard rock
    
    /// Sleep/relaxation tempo
    pub relaxation_bpm: (u32, u32),  // Much lower
}

/// Special tracks that transcend genre
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialTrack {
    pub title: String,
    pub artist: String,
    pub significance: String,
    pub mood_effect: MoodState,
}

/// The Mood Engine - tracks how music affects your state
pub struct MoodEngine {
    profile: MusicProfile,
    current_state: MoodState,
    history: Vec<MoodTransition>,
    marine_processor: MarineProcessor,
}

/// Records mood transitions triggered by music
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodTransition {
    pub from_state: MoodState,
    pub to_state: MoodState,
    pub trigger_music: String,
    pub timestamp: u64,
    pub effectiveness: f64,  // How well it worked
}

impl MoodEngine {
    /// Create Hue's personalized mood engine
    pub fn create_hue_profile() -> Self {
        let mut activity_preferences = HashMap::new();
        activity_preferences.insert(
            Activity::Programming,
            vec![Genre::Electronic]
        );
        activity_preferences.insert(
            Activity::Decompressing,
            vec![Genre::HardRock]
        );
        activity_preferences.insert(
            Activity::DeepThinking,
            vec![Genre::Ambient, Genre::Classical]
        );
        
        let mut artist_affinities = HashMap::new();
        artist_affinities.insert("Nine Inch Nails".to_string(), AffinityLevel::Essential);
        artist_affinities.insert("Linkin Park".to_string(), AffinityLevel::Love);
        artist_affinities.insert("Enya".to_string(), AffinityLevel::Love);
        artist_affinities.insert("David Lanz".to_string(), AffinityLevel::Love);
        artist_affinities.insert("Enigma".to_string(), AffinityLevel::Love);
        artist_affinities.insert("Orbital".to_string(), AffinityLevel::Love);
        artist_affinities.insert("Paradoks".to_string(), AffinityLevel::Love);
        artist_affinities.insert("Laura Van Dam".to_string(), AffinityLevel::Love); // 😉
        artist_affinities.insert("Beethoven".to_string(), AffinityLevel::Appreciate);
        artist_affinities.insert("Generic Polka Band".to_string(), AffinityLevel::Never);
        
        let profile = MusicProfile {
            name: "Hue".to_string(),
            preferred_genres: vec![
                Genre::Electronic,
                Genre::HardRock,
                Genre::Ambient,
                Genre::Industrial,
                Genre::Crossover,
                Genre::Spatial,
            ],
            avoid_genres: vec![
                Genre::Polka,  // Never!
            ],
            activity_preferences,
            artist_affinities,
            tempo_preferences: TempoPreference {
                focus_bpm: (100, 130),      // Electronic sweet spot
                fatigue_threshold: 160,      // Too fast for too long
                decompression_bpm: (120, 180), // Hard rock range
                relaxation_bpm: (50, 80),    // Ambient zone
            },
            special_tracks: vec![
                SpecialTrack {
                    title: "The Box".to_string(),
                    artist: "Orbital".to_string(),
                    significance: "Mostly disc 2 - perfect electronic flow".to_string(),
                    mood_effect: MoodState::FlowState {
                        efficiency_multiplier: 1.8,
                        focus_level: 0.9,
                        preferred_bpm: 125,
                    },
                },
                SpecialTrack {
                    title: "Symphony No. 5".to_string(),
                    artist: "Beethoven".to_string(),
                    significance: "Bends sound and minds".to_string(),
                    mood_effect: MoodState::Contemplation {
                        temporal_expansion: 2.0,
                        creativity_boost: 1.5,
                        wonder_threshold: 0.3,
                    },
                },
            ],
        };
        
        let mut processor = MarineProcessor::for_audio(44100.0);
        processor.wonder_threshold = 0.6;  // Tuned for musical wonder
        
        Self {
            profile,
            current_state: MoodState::FlowState {
                efficiency_multiplier: 1.0,
                focus_level: 0.5,
                preferred_bpm: 120,
            },
            history: Vec::new(),
            marine_processor: processor,
        }
    }
    
    /// Analyze how a piece of music will affect mood
    pub fn predict_mood_effect(&mut self, 
                               audio_samples: &[f64], 
                               metadata: &MarineMetadata,
                               artist: Option<&str>) -> MoodPrediction {
        // Check artist affinity
        let affinity = artist.and_then(|a| self.profile.artist_affinities.get(a));
        
        // Analyze tempo/energy
        let energy_level = metadata.average_salience;
        let has_rhythm = metadata.has_rhythm;
        let wonder_ratio = metadata.wonder_count as f64 / metadata.total_peaks.max(1) as f64;
        
        // Predict mood effect based on current state and music properties
        let predicted_state = match (&self.current_state, energy_level, wonder_ratio) {
            // High energy + rhythm = good for decompression
            (_, e, _) if e > 0.7 && has_rhythm => {
                MoodState::Decompression {
                    annoyance_reduction: e * 0.8,
                    energy_release: e,
                    volume_preference: 0.8,
                }
            },
            
            // Low energy + high wonder = contemplation
            (_, e, w) if e < 0.4 && w > 0.5 => {
                MoodState::Contemplation {
                    temporal_expansion: 1.5,
                    creativity_boost: w,
                    wonder_threshold: 0.4,
                }
            },
            
            // Medium energy + steady = flow state
            (_, e, _) if e > 0.4 && e < 0.7 && has_rhythm => {
                MoodState::FlowState {
                    efficiency_multiplier: 1.5,
                    focus_level: 0.8,
                    preferred_bpm: 120,
                }
            },
            
            // Default to energy balance
            _ => MoodState::EnergyBalance {
                sustainable_pace: 1.0,
                irritation_threshold: 0.7,
                optimal_duration: 45,
            }
        };
        
        // Calculate effectiveness based on affinity
        let effectiveness = match affinity {
            Some(AffinityLevel::Essential) => 0.95,
            Some(AffinityLevel::Love) => 0.85,
            Some(AffinityLevel::Appreciate) => 0.7,
            Some(AffinityLevel::Contextual) => 0.5,
            Some(AffinityLevel::Avoid) => 0.2,
            Some(AffinityLevel::Never) => 0.0,
            None => 0.6,
        };
        
        MoodPrediction {
            predicted_state,
            effectiveness,
            recommendation: self.generate_recommendation(effectiveness),
        }
    }
    
    /// Generate recommendation based on effectiveness
    fn generate_recommendation(&self, effectiveness: f64) -> String {
        match effectiveness {
            e if e > 0.8 => "🎯 Perfect choice for current mood!".to_string(),
            e if e > 0.6 => "✅ Good selection, will help transition mood".to_string(),
            e if e > 0.4 => "🤔 Might work, depends on context".to_string(),
            e if e > 0.2 => "⚠️ Not ideal, but could surprise you".to_string(),
            _ => "🚫 This doesn't vibe with you - skip it!".to_string(),
        }
    }
    
    /// Record an actual mood transition
    pub fn record_transition(&mut self, 
                            new_state: MoodState, 
                            trigger_music: String,
                            effectiveness: f64) {
        let transition = MoodTransition {
            from_state: self.current_state.clone(),
            to_state: new_state.clone(),
            trigger_music,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            effectiveness,
        };
        
        self.history.push(transition);
        self.current_state = new_state;
    }
}

/// Mood prediction result
pub struct MoodPrediction {
    pub predicted_state: MoodState,
    pub effectiveness: f64,
    pub recommendation: String,
}

/// Fun insights about music and mood
pub fn music_wisdom() -> &'static str {
    "🎵 Music Wisdom from the Waves:\n\
     \n\
     • High BPM ≠ High Productivity (fatigue is real!)\n\
     • Electronic music creates flow states by occupying the verbal mind\n\
     • Hard rock is emotional garbage collection - releases memory pressure\n\
     • Ambient expands temporal perception - 5 minutes feels like 20\n\
     • NIN shaped early neural pathways (Trent knows the dark waves)\n\
     • Polka and programming don't mix (except in Nebraska)\n\
     • Laura Van Dam improves... visual debugging 😏\n\
     • Beethoven's 5th is literally mind-bending - changes thought patterns\n\
     • The speed of thought follows the tempo of input\n\
     \n\
     Remember: Your playlist is your productivity algorithm!"
}

impl std::fmt::Display for MoodState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoodState::FlowState { efficiency_multiplier, focus_level, .. } => {
                write!(f, "🌊 Flow State: {:.0}% efficiency, {:.0}% focus", 
                       efficiency_multiplier * 100.0, focus_level * 100.0)
            },
            MoodState::Decompression { annoyance_reduction, .. } => {
                write!(f, "🎸 Decompression: -{:.0}% annoyance", annoyance_reduction * 100.0)
            },
            MoodState::Contemplation { creativity_boost, .. } => {
                write!(f, "🧘 Contemplation: +{:.0}% creativity", creativity_boost * 100.0)
            },
            MoodState::EnergyBalance { sustainable_pace, .. } => {
                write!(f, "⚖️ Energy Balance: {:.1}x sustainable pace", sustainable_pace)
            },
            MoodState::Inspiration { mood_elevation, .. } => {
                write!(f, "✨ Inspiration: +{:.0}% mood", mood_elevation * 100.0)
            },
        }
    }
}