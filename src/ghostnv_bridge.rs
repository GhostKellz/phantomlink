use crate::ghostnv::GhostNVProcessor;
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{warn, error};

pub struct AudioProcessingRequest {
    pub user_id: u32,
    pub audio_data: Vec<f32>,
    pub response_channel: Sender<Vec<f32>>,
}

pub struct GhostNVBridge {
    request_sender: Sender<AudioProcessingRequest>,
    processor: Arc<Mutex<Option<GhostNVProcessor>>>,
}

impl GhostNVBridge {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = unbounded();
        let processor = Arc::new(Mutex::new(None));
        
        // Spawn async processing task
        let processor_clone = Arc::clone(&processor);
        tokio::spawn(async move {
            Self::processing_task(request_receiver, processor_clone).await;
        });
        
        Self {
            request_sender,
            processor,
        }
    }
    
    pub async fn initialize(&self) -> anyhow::Result<()> {
        let ghostnv = GhostNVProcessor::new().await?;
        let mut processor = self.processor.lock().await;
        *processor = Some(ghostnv);
        Ok(())
    }
    
    pub fn process_audio_sync(&self, user_id: u32, audio_data: Vec<f32>) -> Option<Vec<f32>> {
        let (response_sender, response_receiver) = unbounded();
        
        let request = AudioProcessingRequest {
            user_id,
            audio_data,
            response_channel: response_sender,
        };
        
        // Send request to async task
        if self.request_sender.send(request).is_ok() {
            // Wait for response with timeout
            match response_receiver.recv_timeout(std::time::Duration::from_millis(5)) {
                Ok(processed_audio) => Some(processed_audio),
                Err(_) => {
                    warn!("GHOSTNV processing timeout for user {}", user_id);
                    None
                }
            }
        } else {
            None
        }
    }
    
    pub async fn create_session(&self, user_id: u32, enhancement_mode: crate::ghostnv_mock::EnhancementMode) -> anyhow::Result<()> {
        let mut processor = self.processor.lock().await;
        if let Some(ref mut ghostnv) = processor.as_mut() {
            ghostnv.create_session(user_id, enhancement_mode).await?;
        }
        Ok(())
    }
    
    pub async fn set_enabled(&self, _enabled: bool) {
        let mut processor = self.processor.lock().await;
        if let Some(ref mut ghostnv) = processor.as_mut() {
            ghostnv.set_enabled(_enabled);
        }
    }
    
    pub async fn is_enabled(&self) -> bool {
        let processor = self.processor.lock().await;
        if let Some(ref ghostnv) = *processor {
            ghostnv.is_enabled()
        } else {
            false
        }
    }
    
    async fn processing_task(
        request_receiver: Receiver<AudioProcessingRequest>,
        processor: Arc<Mutex<Option<GhostNVProcessor>>>,
    ) {
        while let Ok(request) = request_receiver.recv() {
            let processor_guard = processor.lock().await;
            
            if let Some(ref ghostnv) = *processor_guard {
                match ghostnv.process_audio(request.user_id, &request.audio_data, None).await {
                    Ok((processed_audio, _stats)) => {
                        let _ = request.response_channel.send(processed_audio);
                    }
                    Err(e) => {
                        error!("GHOSTNV processing error for user {}: {}", request.user_id, e);
                        let _ = request.response_channel.send(request.audio_data); // Fallback to original
                    }
                }
            } else {
                // No processor available, return original audio
                let _ = request.response_channel.send(request.audio_data);
            }
        }
    }
}