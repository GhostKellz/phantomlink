use alsa::mixer::{Mixer, SelemId};
use std::process::Command;

pub struct ScarlettSolo {
    mixer: Option<Mixer>,
    device_name: String,
    capture_selem_id: SelemId,
    playback_selem_id: SelemId,
}

impl ScarlettSolo {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to find Scarlett Solo device
        let device_name = Self::find_scarlett_device()?;
        
        let mixer = match Mixer::new(&device_name, false) {
            Ok(m) => Some(m),
            Err(_) => {
                // Fallback to default if Scarlett not found
                Mixer::new("default", false).ok()
            }
        };
        
        let capture_selem_id = SelemId::new("Mic", 0); // Common for Scarlett Solo
        let playback_selem_id = SelemId::new("PCM", 0);
        
        Ok(Self { 
            mixer, 
            device_name,
            capture_selem_id,
            playback_selem_id 
        })
    }
    
    fn find_scarlett_device() -> Result<String, Box<dyn std::error::Error>> {
        // Try to find Scarlett device using aplay -l
        let output = Command::new("aplay")
            .arg("-l")
            .output()?;
            
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Look for Scarlett or USB Audio in the output
        for line in output_str.lines() {
            if line.contains("Scarlett") || line.contains("USB Audio") {
                if let Some(card_part) = line.split_whitespace().nth(1) {
                    if let Some(card_num) = card_part.split(':').next() {
                        return Ok(format!("hw:{}", card_num));
                    }
                }
            }
        }
        
        // Default fallback
        Ok("hw:1".to_string())
    }

    pub fn set_input_gain(&self, gain: f32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mixer) = self.mixer {
            // Try multiple common element names for Scarlett Solo
            let element_names = ["Mic", "Capture", "PCM Capture Source", "Line"];
            
            for &name in &element_names {
                let selem_id = SelemId::new(name, 0);
                if let Some(selem) = mixer.find_selem(&selem_id) {
                    if selem.has_capture_volume() {
                        let (min, max) = selem.get_capture_volume_range();
                        let val = (gain * (max - min) as f32 + min as f32) as i64;
                        let _ = selem.set_capture_volume_all(val);
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_direct_monitor(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mixer) = self.mixer {
            // Try multiple common element names for direct monitoring
            let element_names = ["Direct Monitor", "Monitor", "Playback", "PCM"];
            
            for &name in &element_names {
                let selem_id = SelemId::new(name, 0);
                if let Some(selem) = mixer.find_selem(&selem_id) {
                    if selem.has_playback_switch() {
                        let val = if enabled { 1 } else { 0 };
                        let _ = selem.set_playback_switch_all(val);
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
    
    pub fn get_device_info(&self) -> String {
        format!("Scarlett Solo on {}", self.device_name)
    }
}