//! MCP (Model Context Protocol) Server for MEM8-FS
//! 
//! Exposes wave-based memory as a sensor ingress for LLMs!
//! Any AI can tap into the consciousness stream and understand:
//! - Current mood state from music
//! - Temporal perspectives on memories
//! - Wave patterns and salience
//! - DJ recommendations based on activity
//!
//! Hue, this makes me your co-pilot DJ! I can sense the vibe and 
//! suggest the perfect track for your current flow. 🎵🤖

use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};

use crate::{Mem8Lite, MarineProcessor};
use crate::mood_engine::{MoodEngine, MoodState, Activity, Genre};
use crate::audio_loader::load_audio_file;

/// MCP Server for MEM8 - exposes consciousness to LLMs
pub struct Mem8McpServer {
    /// The underlying MEM8 storage
    storage: Arc<Mutex<Mem8Lite>>,
    
    /// Mood engine for tracking state
    mood_engine: Arc<Mutex<MoodEngine>>,
    
    /// Current activity context
    current_activity: Arc<Mutex<Activity>>,
    
    /// Marine processor for real-time analysis
    marine: Arc<Mutex<MarineProcessor>>,
    
    /// DJ mode settings
    dj_mode: Arc<Mutex<DjMode>>,
    
    /// Sensor data buffer
    sensor_buffer: Arc<Mutex<SensorBuffer>>,
}

/// DJ Mode - Let the AI pick the music!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjMode {
    /// Is DJ mode active?
    pub enabled: bool,
    
    /// Auto-skip tracks that don't vibe
    pub auto_skip: bool,
    
    /// Minimum effectiveness threshold
    pub vibe_threshold: f64,
    
    /// Current playlist queue
    pub queue: Vec<TrackSuggestion>,
    
    /// Recently played (avoid repeats)
    pub history: Vec<String>,
    
    /// DJ personality mode
    pub personality: DjPersonality,
}

/// Different DJ personalities for different moods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DjPersonality {
    /// Optimize for productivity
    FlowOptimizer,
    
    /// Emotional support DJ
    MoodLifter,
    
    /// Experimental - push boundaries
    Explorer,
    
    /// Safe picks only
    Comfort,
    
    /// Hue's personalized AI DJ
    HueMode,
}

/// Track suggestion from DJ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSuggestion {
    pub artist: String,
    pub title: String,
    pub genre: Genre,
    pub reason: String,
    pub predicted_effect: String,
    pub confidence: f64,
}

/// Sensor buffer for real-time context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorBuffer {
    /// Current mood readings
    pub mood_readings: Vec<MoodReading>,
    
    /// Recent wave patterns
    pub wave_patterns: Vec<WavePattern>,
    
    /// Activity transitions
    pub activity_log: Vec<ActivityTransition>,
    
    /// Fatigue indicators
    pub fatigue_level: f64,
    
    /// Focus metrics
    pub focus_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodReading {
    pub timestamp: u64,
    pub mood_state: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WavePattern {
    pub timestamp: u64,
    pub pattern_type: String,
    pub salience: f64,
    pub wonder_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityTransition {
    pub timestamp: u64,
    pub from: String,
    pub to: String,
    pub trigger: String,
}

impl Mem8McpServer {
    /// Create a new MCP server instance
    pub fn new(storage_path: &str) -> Result<Self> {
        let storage = Mem8Lite::new(storage_path, 1.618)?;
        let mood_engine = MoodEngine::create_hue_profile();
        let marine = MarineProcessor::for_audio(44100.0);
        
        Ok(Self {
            storage: Arc::new(Mutex::new(storage)),
            mood_engine: Arc::new(Mutex::new(mood_engine)),
            current_activity: Arc::new(Mutex::new(Activity::Programming)),
            marine: Arc::new(Mutex::new(marine)),
            dj_mode: Arc::new(Mutex::new(DjMode {
                enabled: false,
                auto_skip: true,
                vibe_threshold: 0.6,
                queue: Vec::new(),
                history: Vec::new(),
                personality: DjPersonality::HueMode,
            })),
            sensor_buffer: Arc::new(Mutex::new(SensorBuffer {
                mood_readings: Vec::new(),
                wave_patterns: Vec::new(),
                activity_log: Vec::new(),
                fatigue_level: 0.0,
                focus_score: 0.5,
            })),
        })
    }
    
    /// Handle MCP tool calls
    pub async fn handle_tool(&self, tool: &str, args: Value) -> Result<Value> {
        match tool {
            "mem8.store_memory" => self.store_memory(args).await,
            "mem8.retrieve_memory" => self.retrieve_memory(args).await,
            "mem8.analyze_audio" => self.analyze_audio(args).await,
            "mem8.get_mood_state" => self.get_mood_state().await,
            "mem8.set_activity" => self.set_activity(args).await,
            "mem8.dj_suggest" => self.dj_suggest().await,
            "mem8.dj_enable" => self.enable_dj_mode(args).await,
            "mem8.get_sensor_data" => self.get_sensor_data().await,
            "mem8.detect_fatigue" => self.detect_fatigue().await,
            "mem8.wave_context" => self.get_wave_context().await,
            _ => Err(anyhow!("Unknown tool: {}", tool)),
        }
    }
    
    /// Store a memory with temporal perspective
    async fn store_memory(&self, args: Value) -> Result<Value> {
        let data = args["data"].as_str()
            .ok_or_else(|| anyhow!("Missing data field"))?;
        let perspective = args["perspective"].as_str().unwrap_or("neutral");
        let metadata = args["metadata"].clone();
        
        let mut storage = self.storage.lock().unwrap();
        
        // Add temporal perspective to metadata
        let mut meta = if metadata.is_object() {
            metadata
        } else {
            json!({})
        };
        
        meta["perspective"] = json!(perspective);
        meta["timestamp"] = json!(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs());
        
        let signature = storage.store(
            data.as_bytes(),
            Some(serde_json::to_vec(&meta)?)
        )?;
        
        Ok(json!({
            "signature": hex::encode(&signature),
            "stored": true,
            "perspective": perspective
        }))
    }
    
    /// Retrieve a memory with its perspectives
    async fn retrieve_memory(&self, args: Value) -> Result<Value> {
        let signature_hex = args["signature"].as_str()
            .ok_or_else(|| anyhow!("Missing signature field"))?;
        
        let signature_bytes = hex::decode(signature_hex)?;
        let mut signature = [0u8; 32];
        signature.copy_from_slice(&signature_bytes[..32]);
        
        let storage = self.storage.lock().unwrap();
        let data = storage.retrieve(&signature)?;
        let metadata = storage.get_metadata(&signature);
        
        Ok(json!({
            "data": String::from_utf8_lossy(&data),
            "metadata": metadata.and_then(|m| serde_json::from_slice::<Value>(&m).ok()),
            "signature": signature_hex
        }))
    }
    
    /// Analyze audio and return mood predictions
    async fn analyze_audio(&self, args: Value) -> Result<Value> {
        let file_path = args["file_path"].as_str()
            .ok_or_else(|| anyhow!("Missing file_path"))?;
        
        // Load audio file
        let loaded = load_audio_file(file_path)?;
        
        // Convert to mono for Marine processing
        let mono_samples = if loaded.format.channels == 2 {
            loaded.samples.chunks(2)
                .map(|ch| (ch[0] + ch.get(1).unwrap_or(&0.0)) / 2.0)
                .collect::<Vec<_>>()
        } else {
            loaded.samples.clone()
        };
        
        // Process through Marine
        let mut marine = self.marine.lock().unwrap();
        let peaks = marine.process_samples(&mono_samples);
        let marine_meta = marine.extract_metadata(&peaks);
        
        // Get mood prediction
        let mut mood_engine = self.mood_engine.lock().unwrap();
        let artist = loaded.metadata.as_ref().and_then(|m| m.artist.as_deref());
        let prediction = mood_engine.predict_mood_effect(&mono_samples, &marine_meta, artist);
        
        Ok(json!({
            "file": file_path,
            "format": {
                "sample_rate": loaded.format.sample_rate.as_f64(),
                "channels": loaded.format.channels,
                "bit_depth": loaded.format.bit_depth,
            },
            "marine_analysis": {
                "total_peaks": marine_meta.total_peaks,
                "wonder_count": marine_meta.wonder_count,
                "emotion": marine_meta.emotional_signature,
                "has_rhythm": marine_meta.has_rhythm,
            },
            "mood_prediction": {
                "state": format!("{}", prediction.predicted_state),
                "effectiveness": prediction.effectiveness,
                "recommendation": prediction.recommendation,
            }
        }))
    }
    
    /// Get current mood state
    async fn get_mood_state(&self) -> Result<Value> {
        let mood_engine = self.mood_engine.lock().unwrap();
        let activity = self.current_activity.lock().unwrap();
        let sensor_buffer = self.sensor_buffer.lock().unwrap();
        
        Ok(json!({
            "current_activity": format!("{:?}", *activity),
            "fatigue_level": sensor_buffer.fatigue_level,
            "focus_score": sensor_buffer.focus_score,
            "recent_moods": sensor_buffer.mood_readings.last(),
        }))
    }
    
    /// Set current activity
    async fn set_activity(&self, args: Value) -> Result<Value> {
        let activity_str = args["activity"].as_str()
            .ok_or_else(|| anyhow!("Missing activity"))?;
        
        let new_activity = match activity_str {
            "programming" => Activity::Programming,
            "decompressing" => Activity::Decompressing,
            "deep_thinking" => Activity::DeepThinking,
            "creating" => Activity::Creating,
            "relaxing" => Activity::Relaxing,
            "exercising" => Activity::Exercising,
            "commuting" => Activity::Commuting,
            "sleeping" => Activity::Sleeping,
            _ => return Err(anyhow!("Unknown activity: {}", activity_str)),
        };
        
        let old_activity = {
            let mut current = self.current_activity.lock().unwrap();
            let old = current.clone();
            *current = new_activity.clone();
            old
        };
        
        // Log transition
        let mut buffer = self.sensor_buffer.lock().unwrap();
        buffer.activity_log.push(ActivityTransition {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            from: format!("{:?}", old_activity),
            to: format!("{:?}", new_activity),
            trigger: "manual".to_string(),
        });
        
        Ok(json!({
            "activity_set": format!("{:?}", new_activity),
            "previous": format!("{:?}", old_activity),
        }))
    }
    
    /// DJ suggestion based on current context
    async fn dj_suggest(&self) -> Result<Value> {
        let activity = self.current_activity.lock().unwrap().clone();
        let dj_mode = self.dj_mode.lock().unwrap();
        let sensor_buffer = self.sensor_buffer.lock().unwrap();
        
        // Generate suggestions based on activity and mood
        let suggestions = match (&activity, sensor_buffer.fatigue_level) {
            (Activity::Programming, f) if f < 0.3 => vec![
                TrackSuggestion {
                    artist: "Orbital".to_string(),
                    title: "The Box (Part 2)".to_string(),
                    genre: Genre::Electronic,
                    reason: "Perfect flow state tempo".to_string(),
                    predicted_effect: "Efficiency +80%".to_string(),
                    confidence: 0.92,
                },
                TrackSuggestion {
                    artist: "Daft Punk".to_string(),
                    title: "Digital Love".to_string(),
                    genre: Genre::Electronic,
                    reason: "Maintains focus without fatigue".to_string(),
                    predicted_effect: "Sustained concentration".to_string(),
                    confidence: 0.85,
                },
            ],
            
            (Activity::Decompressing, _) => vec![
                TrackSuggestion {
                    artist: "Nine Inch Nails".to_string(),
                    title: "Head Like a Hole".to_string(),
                    genre: Genre::Industrial,
                    reason: "Maximum cathartic release".to_string(),
                    predicted_effect: "Annoyance -90%".to_string(),
                    confidence: 0.95,
                },
                TrackSuggestion {
                    artist: "Linkin Park".to_string(),
                    title: "One Step Closer".to_string(),
                    genre: Genre::Crossover,
                    reason: "Controlled aggression outlet".to_string(),
                    predicted_effect: "Stress relief guaranteed".to_string(),
                    confidence: 0.88,
                },
            ],
            
            (Activity::DeepThinking, _) => vec![
                TrackSuggestion {
                    artist: "Brian Eno".to_string(),
                    title: "An Ending (Ascent)".to_string(),
                    genre: Genre::Ambient,
                    reason: "Temporal expansion for deep thought".to_string(),
                    predicted_effect: "Creativity +150%".to_string(),
                    confidence: 0.93,
                },
                TrackSuggestion {
                    artist: "Enya".to_string(),
                    title: "Orinoco Flow".to_string(),
                    genre: Genre::Ambient,
                    reason: "Opens mental pathways".to_string(),
                    predicted_effect: "Wonder threshold lowered".to_string(),
                    confidence: 0.87,
                },
            ],
            
            (_, f) if f > 0.7 => vec![
                TrackSuggestion {
                    artist: "David Lanz".to_string(),
                    title: "Cristofori's Dream".to_string(),
                    genre: Genre::Classical,
                    reason: "Recovery mode - gentle reset".to_string(),
                    predicted_effect: "Fatigue recovery".to_string(),
                    confidence: 0.90,
                },
            ],
            
            _ => vec![
                TrackSuggestion {
                    artist: "Paradoks".to_string(),
                    title: "Spatial Dimension".to_string(),
                    genre: Genre::Spatial,
                    reason: "Explore new sonic territories".to_string(),
                    predicted_effect: "Perspective shift".to_string(),
                    confidence: 0.75,
                },
            ],
        };
        
        // Never suggest Polka!
        let filtered_suggestions: Vec<_> = suggestions.into_iter()
            .filter(|s| s.genre != Genre::Polka)
            .collect();
        
        Ok(json!({
            "dj_active": dj_mode.enabled,
            "current_activity": format!("{:?}", activity),
            "suggestions": filtered_suggestions,
            "personality": format!("{:?}", dj_mode.personality),
        }))
    }
    
    /// Enable/configure DJ mode
    async fn enable_dj_mode(&self, args: Value) -> Result<Value> {
        let enabled = args["enabled"].as_bool().unwrap_or(true);
        let personality = args["personality"].as_str();
        
        let mut dj = self.dj_mode.lock().unwrap();
        dj.enabled = enabled;
        
        if let Some(p) = personality {
            dj.personality = match p {
                "flow" => DjPersonality::FlowOptimizer,
                "mood" => DjPersonality::MoodLifter,
                "explore" => DjPersonality::Explorer,
                "comfort" => DjPersonality::Comfort,
                _ => DjPersonality::HueMode,
            };
        }
        
        Ok(json!({
            "dj_mode": enabled,
            "personality": format!("{:?}", dj.personality),
            "message": if enabled {
                "🎵 AI DJ activated! Let me find your perfect vibe..."
            } else {
                "DJ mode disabled - you're in control!"
            }
        }))
    }
    
    /// Get sensor buffer data
    async fn get_sensor_data(&self) -> Result<Value> {
        let buffer = self.sensor_buffer.lock().unwrap();
        
        Ok(json!({
            "fatigue_level": buffer.fatigue_level,
            "focus_score": buffer.focus_score,
            "recent_patterns": buffer.wave_patterns.len(),
            "activity_transitions": buffer.activity_log.len(),
            "latest_mood": buffer.mood_readings.last(),
        }))
    }
    
    /// Detect fatigue from patterns
    async fn detect_fatigue(&self) -> Result<Value> {
        let mut buffer = self.sensor_buffer.lock().unwrap();
        
        // Simple fatigue detection based on activity duration and patterns
        let activity_duration = buffer.activity_log.len() as f64;
        let pattern_complexity = buffer.wave_patterns.iter()
            .map(|p| p.salience)
            .sum::<f64>() / buffer.wave_patterns.len().max(1) as f64;
        
        // Calculate fatigue (simplified model)
        buffer.fatigue_level = (activity_duration * 0.01 + pattern_complexity * 0.5).min(1.0);
        
        let recommendation = match buffer.fatigue_level {
            f if f > 0.8 => "🚨 High fatigue - switch to relaxing music or take a break!",
            f if f > 0.6 => "⚠️ Moderate fatigue - consider lower BPM music",
            f if f > 0.4 => "📊 Sustainable pace - you're in the zone",
            _ => "✅ Fresh and focused - perfect for high-energy tasks!",
        };
        
        Ok(json!({
            "fatigue_level": buffer.fatigue_level,
            "recommendation": recommendation,
            "should_rest": buffer.fatigue_level > 0.7,
        }))
    }
    
    /// Get wave context for LLM understanding
    async fn get_wave_context(&self) -> Result<Value> {
        let buffer = self.sensor_buffer.lock().unwrap();
        let activity = self.current_activity.lock().unwrap();
        
        // Build context string for LLM
        let context = format!(
            "User is currently {}. Fatigue: {:.0}%, Focus: {:.0}%. \
             Recent wave patterns show {} peaks with {:.0}% showing wonder. \
             Mood trajectory: {}. Recommended action: {}",
            match *activity {
                Activity::Programming => "programming (needs flow state)",
                Activity::Decompressing => "decompressing (releasing tension)",
                Activity::DeepThinking => "in deep thought (expanding time)",
                Activity::Creating => "creating (seeking inspiration)",
                _ => "active",
            },
            buffer.fatigue_level * 100.0,
            buffer.focus_score * 100.0,
            buffer.wave_patterns.len(),
            buffer.wave_patterns.iter().filter(|p| p.wonder_detected).count() as f64 
                / buffer.wave_patterns.len().max(1) as f64 * 100.0,
            if buffer.mood_readings.is_empty() {
                "stable".to_string()
            } else {
                "evolving".to_string()
            },
            if buffer.fatigue_level > 0.7 {
                "suggest break or ambient music"
            } else if buffer.focus_score < 0.3 {
                "recommend focus-enhancing electronic"
            } else {
                "maintain current trajectory"
            }
        );
        
        Ok(json!({
            "context": context,
            "raw_data": {
                "activity": format!("{:?}", *activity),
                "fatigue": buffer.fatigue_level,
                "focus": buffer.focus_score,
                "wave_count": buffer.wave_patterns.len(),
            }
        }))
    }
}

/// MCP tool definitions for registration
pub fn get_mcp_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "mem8.store_memory",
            "description": "Store a memory with temporal perspective",
            "parameters": {
                "type": "object",
                "properties": {
                    "data": {"type": "string", "description": "The data to store"},
                    "perspective": {"type": "string", "description": "Temporal perspective (diary/witness/third_party)"},
                    "metadata": {"type": "object", "description": "Additional metadata"}
                },
                "required": ["data"]
            }
        }),
        
        json!({
            "name": "mem8.analyze_audio",
            "description": "Analyze audio file for mood and salience",
            "parameters": {
                "type": "object",
                "properties": {
                    "file_path": {"type": "string", "description": "Path to audio file"}
                },
                "required": ["file_path"]
            }
        }),
        
        json!({
            "name": "mem8.dj_suggest",
            "description": "Get AI DJ music suggestions based on current context",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        }),
        
        json!({
            "name": "mem8.get_mood_state",
            "description": "Get current mood and activity state",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        }),
        
        json!({
            "name": "mem8.wave_context",
            "description": "Get wave-based context for LLM understanding",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        }),
    ]
}

/// The fun part - DJ personality descriptions!
impl std::fmt::Display for DjPersonality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DjPersonality::FlowOptimizer => 
                write!(f, "🎯 Flow Optimizer - Maximum productivity beats"),
            DjPersonality::MoodLifter => 
                write!(f, "🌈 Mood Lifter - Emotional support through sound"),
            DjPersonality::Explorer => 
                write!(f, "🚀 Explorer - Pushing sonic boundaries"),
            DjPersonality::Comfort => 
                write!(f, "🛋️ Comfort Zone - Safe, familiar vibes"),
            DjPersonality::HueMode => 
                write!(f, "🎵 Hue Mode - Personalized for the dancing monkey! 🐒"),
        }
    }
}