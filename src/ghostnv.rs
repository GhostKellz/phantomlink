// Use mock implementation until real GHOSTNV FFI is complete
// When ready, uncomment this line and remove mock import:
// use ghostnv_rtx_voice::{RtxVoice, PhantomLink, SessionConfig, EnhancementMode, AudioBuffer, SampleRate, UserAudioInput, AudioResult};

use crate::ghostnv_mock::{RtxVoice, PhantomLink, SessionConfig, EnhancementMode, AudioBuffer, SampleRate, UserAudioInput, AudioResult};
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, warn, error};

pub struct GhostNVProcessor {
    phantomlink: Arc<PhantomLink>,
    sessions: HashMap<u32, SessionConfig>,
    enabled: bool,
    sample_rate: f32,
}

impl GhostNVProcessor {
    pub async fn new() -> Result<Self> {
        info!("Initializing GHOSTNV RTX Voice processor");
        
        // Initialize GHOSTNV library first (mock doesn't need this)
        // ghostnv_rtx_voice::init().await
        //     .map_err(|e| anyhow::anyhow!("Failed to initialize GHOSTNV library: {:?}", e))?;
        
        // Initialize RTX Voice
        let rtx_voice = RtxVoice::new().await
            .map_err(|e| anyhow::anyhow!("Failed to initialize RTX Voice: {:?}", e))?;
        
        // Create PhantomLink integration
        let phantomlink = Arc::new(PhantomLink::new(rtx_voice).await
            .map_err(|e| anyhow::anyhow!("Failed to create PhantomLink: {:?}", e))?);
        
        info!("🎮 GHOSTNV RTX Voice initialized successfully");
        
        Ok(Self {
            phantomlink,
            sessions: HashMap::new(),
            enabled: true,
            sample_rate: 48000.0,
        })
    }
    
    pub async fn create_session(&mut self, user_id: u32, enhancement_mode: EnhancementMode) -> Result<()> {
        let config = match enhancement_mode {
            EnhancementMode::Aggressive => SessionConfig::voice_chat()
                .with_low_latency()
                .with_enhancement(EnhancementMode::Aggressive),
            EnhancementMode::Balanced => SessionConfig::voice_chat()
                .with_enhancement(EnhancementMode::Balanced),
            EnhancementMode::StudioQuality => SessionConfig::live_streaming()
                .with_enhancement(EnhancementMode::StudioQuality),
        };
        
        self.phantomlink.create_session(user_id, config.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to create session for user {}: {:?}", user_id, e))?;
        
        self.sessions.insert(user_id, config);
        info!("Created GHOSTNV session for user {} with {:?} enhancement", user_id, enhancement_mode);
        
        Ok(())
    }
    
    pub async fn process_audio(&self, user_id: u32, audio_data: &[f32], background_music: Option<&[f32]>) -> Result<(Vec<f32>, AudioResult)> {
        if !self.enabled || !self.sessions.contains_key(&user_id) {
            // Return original audio if disabled or no session
            return Ok((audio_data.to_vec(), AudioResult::default()));
        }
        
        // Convert to GHOSTNV AudioBuffer
        let audio_buffer = AudioBuffer::from_f32_slice(audio_data);
        let music_buffer = background_music.map(|music| AudioBuffer::from_f32_slice(music));
        
        // Process with GHOSTNV
        let (enhanced_audio, stats) = self.phantomlink
            .process_user_audio(user_id, &audio_buffer, music_buffer.as_ref())
            .await
            .map_err(|e| anyhow::anyhow!("GHOSTNV processing failed for user {}: {:?}", user_id, e))?;
        
        // Log performance metrics
        if stats.latency_ms > 5.0 {
            warn!("High GHOSTNV latency detected for user {}: {:.1}ms", user_id, stats.latency_ms);
        }
        
        Ok((enhanced_audio.to_f32_vec(), stats))
    }
    
    pub async fn process_multi_user(&self, users: Vec<(u32, &[f32])>, background_music: Option<&[f32]>) -> Result<(Vec<f32>, Vec<AudioResult>)> {
        if !self.enabled {
            // Return mixed original audio if disabled
            let mut mixed = vec![0.0; users.first().map(|(_, audio)| audio.len()).unwrap_or(0)];
            for (_, audio) in &users {
                for (i, &sample) in audio.iter().enumerate() {
                    if i < mixed.len() {
                        mixed[i] += sample / users.len() as f32;
                    }
                }
            }
            return Ok((mixed, vec![]));
        }
        
        // Convert to UserAudioInput
        let user_inputs: Vec<UserAudioInput> = users
            .into_iter()
            .map(|(user_id, audio)| UserAudioInput::new(user_id, AudioBuffer::from_f32_slice(audio)))
            .collect();
        
        let music_buffer = background_music.map(|music| AudioBuffer::from_f32_slice(music));
        
        // Process with GHOSTNV multi-user processing
        let (mixed_output, stats) = self.phantomlink
            .process_multi_user(&user_inputs, music_buffer.as_ref())
            .await
            .map_err(|e| anyhow::anyhow!("GHOSTNV multi-user processing failed: {:?}", e))?;
        
        info!("📊 Processed {} voices, {:.1}ms latency", 
              stats.active_voice_count, stats.total_latency_ms);
        
        Ok((mixed_output.to_f32_vec(), vec![stats]))
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        info!("GHOSTNV processing {}", if enabled { "enabled" } else { "disabled" });
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn get_active_sessions(&self) -> Vec<u32> {
        self.sessions.keys().cloned().collect()
    }
}