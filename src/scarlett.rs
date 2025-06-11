use alsa::mixer::{Mixer, SelemId, SelemChannelId};

pub struct ScarlettSolo {
    mixer: Mixer,
    selem_id: SelemId,
}

impl ScarlettSolo {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // You may need to adjust "hw:USB" to match your Scarlett device (see `aplay -l`)
        let mixer = Mixer::new("hw:USB", false)?;
        let selem_id = SelemId::new("Capture", 0); // Use amixer to confirm element name
        Ok(Self { mixer, selem_id })
    }

    pub fn set_input_gain(&self, gain: f32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selem) = self.mixer.find_selem(&self.selem_id) {
            let (min, max) = selem.get_capture_volume_range();
            let val = (gain * (max - min) as f32 + min as f32) as i64;
            selem.set_capture_volume_all(val)?;
        }
        Ok(())
    }

    pub fn set_direct_monitor(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        // This is a placeholder; you may need to adjust the element name and logic
        let selem_id = SelemId::new("Direct Monitor", 0);
        if let Some(selem) = self.mixer.find_selem(&selem_id) {
            let val = if enabled { 1 } else { 0 };
            selem.set_playback_switch_all(val)?;
        }
        Ok(())
    }
}