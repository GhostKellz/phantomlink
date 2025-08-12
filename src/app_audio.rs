use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use crossbeam_channel::{Receiver, Sender, bounded};

#[derive(Debug, Clone)]
pub struct AudioApplication {
    pub process_name: String,
    pub display_name: String,
    pub pid: u32,
    pub volume: f32,
    pub muted: bool,
    pub output_routing: OutputRouting,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputRouting {
    Headphones,      // Route to headphones only
    Stream,          // Route to stream/recording only  
    Both,            // Route to both
    None,            // Muted/disabled
}

impl Default for OutputRouting {
    fn default() -> Self {
        Self::Both
    }
}

pub struct ApplicationAudioRouter {
    applications: Arc<Mutex<HashMap<String, AudioApplication>>>,
    audio_streams: HashMap<String, Stream>,
    routing_sender: Option<Sender<AudioRoutingCommand>>,
    monitoring_active: bool,
}

#[derive(Debug, Clone)]
pub enum AudioRoutingCommand {
    SetApplicationVolume { app_name: String, volume: f32 },
    SetApplicationMute { app_name: String, muted: bool },
    SetApplicationRouting { app_name: String, routing: OutputRouting },
    RefreshApplications,
}

impl ApplicationAudioRouter {
    pub fn new() -> Self {
        let applications = Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver) = bounded(100);
        
        // Start background thread for processing routing commands
        let apps_clone = Arc::clone(&applications);
        thread::spawn(move || {
            Self::routing_thread(receiver, apps_clone);
        });
        
        Self {
            applications,
            audio_streams: HashMap::new(),
            routing_sender: Some(sender),
            monitoring_active: false,
        }
    }
    
    /// Start monitoring applications and their audio streams
    pub fn start_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.monitoring_active {
            return Ok(());
        }
        
        println!("Starting application audio monitoring...");
        
        // Initial scan for applications
        self.scan_audio_applications()?;
        
        // Start periodic scanning
        let apps_clone = Arc::clone(&self.applications);
        let sender = self.routing_sender.as_ref().unwrap().clone();
        
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(2));
                let _ = sender.send(AudioRoutingCommand::RefreshApplications);
            }
        });
        
        self.monitoring_active = true;
        println!("Application audio monitoring started");
        Ok(())
    }
    
    /// Scan for applications that are currently producing audio
    fn scan_audio_applications(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Use pactl to get PulseAudio application streams
        let output = Command::new("pactl")
            .args(&["list", "sink-inputs"])
            .output();
            
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_pulseaudio_applications(&stdout)?;
        } else {
            // Fallback: try to detect common applications
            self.detect_common_applications()?;
        }
        
        Ok(())
    }
    
    /// Parse PulseAudio sink inputs to find audio applications
    fn parse_pulseaudio_applications(&self, output: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_app: Option<AudioApplication> = None;
        let mut apps = self.applications.lock().unwrap();
        
        for line in output.lines() {
            let line = line.trim();
            
            if line.starts_with("Sink Input #") {
                // Save previous app if exists
                if let Some(app) = current_app.take() {
                    apps.insert(app.process_name.clone(), app);
                }
                current_app = Some(AudioApplication {
                    process_name: String::new(),
                    display_name: String::new(),
                    pid: 0,
                    volume: 1.0,
                    muted: false,
                    output_routing: OutputRouting::Both,
                    is_active: true,
                });
            } else if let Some(ref mut app) = current_app {
                if line.starts_with("application.name = ") {
                    app.process_name = line.split('"').nth(1).unwrap_or("Unknown").to_string();
                    app.display_name = app.process_name.clone();
                } else if line.starts_with("application.process.id = ") {
                    if let Ok(pid) = line.split('"').nth(1).unwrap_or("0").parse() {
                        app.pid = pid;
                    }
                } else if line.starts_with("application.process.binary = ") {
                    let binary = line.split('"').nth(1).unwrap_or("").to_string();
                    if !binary.is_empty() {
                        app.process_name = binary;
                        
                        // Create user-friendly display names
                        app.display_name = match app.process_name.as_str() {
                            "firefox" => "🦊 Firefox".to_string(),
                            "discord" => "💬 Discord".to_string(),
                            "spotify" => "🎵 Spotify".to_string(),
                            "obs" => "🎥 OBS Studio".to_string(),
                            "steam" => "🎮 Steam".to_string(),
                            "vlc" => "🎬 VLC Media Player".to_string(),
                            "chrome" | "chromium" => "🌐 Chrome".to_string(),
                            name => format!("🔊 {}", name.replace("-", " ").replace("_", " ")),
                        };
                    }
                }
            }
        }
        
        // Save last app
        if let Some(app) = current_app {
            apps.insert(app.process_name.clone(), app);
        }
        
        Ok(())
    }
    
    /// Fallback detection for common applications
    fn detect_common_applications(&self) -> Result<(), Box<dyn std::error::Error>> {
        let common_apps = [
            ("discord", "💬 Discord"),
            ("firefox", "🦊 Firefox"),
            ("spotify", "🎵 Spotify"),
            ("obs", "🎥 OBS Studio"),
            ("steam", "🎮 Steam"),
            ("vlc", "🎬 VLC"),
            ("chrome", "🌐 Chrome"),
            ("pulseaudio", "🔊 System Audio"),
        ];
        
        let mut apps = self.applications.lock().unwrap();
        
        for (process_name, display_name) in &common_apps {
            // Check if process is running
            let output = Command::new("pgrep")
                .arg(process_name)
                .output();
                
            if let Ok(output) = output {
                if !output.stdout.is_empty() {
                    let pid_str = String::from_utf8_lossy(&output.stdout);
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        apps.insert(process_name.to_string(), AudioApplication {
                            process_name: process_name.to_string(),
                            display_name: display_name.to_string(),
                            pid,
                            volume: 1.0,
                            muted: false,
                            output_routing: OutputRouting::Both,
                            is_active: true,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Background thread for processing routing commands
    fn routing_thread(receiver: Receiver<AudioRoutingCommand>, applications: Arc<Mutex<HashMap<String, AudioApplication>>>) {
        while let Ok(command) = receiver.recv() {
            match command {
                AudioRoutingCommand::SetApplicationVolume { app_name, volume } => {
                    if let Ok(mut apps) = applications.lock() {
                        if let Some(app) = apps.get_mut(&app_name) {
                            app.volume = volume;
                            // Apply volume via PulseAudio
                            Self::set_pulseaudio_volume(&app_name, volume);
                        }
                    }
                }
                AudioRoutingCommand::SetApplicationMute { app_name, muted } => {
                    if let Ok(mut apps) = applications.lock() {
                        if let Some(app) = apps.get_mut(&app_name) {
                            app.muted = muted;
                            // Apply mute via PulseAudio
                            Self::set_pulseaudio_mute(&app_name, muted);
                        }
                    }
                }
                AudioRoutingCommand::SetApplicationRouting { app_name, routing } => {
                    if let Ok(mut apps) = applications.lock() {
                        if let Some(app) = apps.get_mut(&app_name) {
                            app.output_routing = routing.clone();
                            // Apply routing via PulseAudio
                            Self::set_pulseaudio_routing(&app_name, &routing);
                        }
                    }
                }
                AudioRoutingCommand::RefreshApplications => {
                    // Re-scan applications
                    let router = ApplicationAudioRouter {
                        applications: Arc::clone(&applications),
                        audio_streams: HashMap::new(),
                        routing_sender: None,
                        monitoring_active: false,
                    };
                    let _ = router.scan_audio_applications();
                }
            }
        }
    }
    
    /// Set application volume via PulseAudio
    fn set_pulseaudio_volume(app_name: &str, volume: f32) {
        let volume_percent = (volume * 100.0) as u32;
        let _ = Command::new("pactl")
            .args(&["set-sink-input-volume", app_name, &format!("{}%", volume_percent)])
            .output();
    }
    
    /// Set application mute via PulseAudio
    fn set_pulseaudio_mute(app_name: &str, muted: bool) {
        let mute_arg = if muted { "1" } else { "0" };
        let _ = Command::new("pactl")
            .args(&["set-sink-input-mute", app_name, mute_arg])
            .output();
    }
    
    /// Set application routing via PulseAudio (move to different sinks)
    fn set_pulseaudio_routing(app_name: &str, routing: &OutputRouting) {
        let sink_name = match routing {
            OutputRouting::Headphones => "headphones", // Assuming headphones sink
            OutputRouting::Stream => "stream_output",   // Assuming stream sink
            OutputRouting::Both => "combined_output",   // Combined sink
            OutputRouting::None => return, // Don't route anywhere
        };
        
        let _ = Command::new("pactl")
            .args(&["move-sink-input", app_name, sink_name])
            .output();
    }
    
    /// Get list of currently detected applications
    pub fn get_applications(&self) -> Vec<AudioApplication> {
        if let Ok(apps) = self.applications.lock() {
            apps.values().cloned().collect()
        } else {
            vec![]
        }
    }
    
    /// Set application volume
    pub fn set_application_volume(&self, app_name: &str, volume: f32) {
        if let Some(sender) = &self.routing_sender {
            let _ = sender.send(AudioRoutingCommand::SetApplicationVolume {
                app_name: app_name.to_string(),
                volume,
            });
        }
    }
    
    /// Set application mute state
    pub fn set_application_mute(&self, app_name: &str, muted: bool) {
        if let Some(sender) = &self.routing_sender {
            let _ = sender.send(AudioRoutingCommand::SetApplicationMute {
                app_name: app_name.to_string(),
                muted,
            });
        }
    }
    
    /// Set application output routing
    pub fn set_application_routing(&self, app_name: &str, routing: OutputRouting) {
        if let Some(sender) = &self.routing_sender {
            let _ = sender.send(AudioRoutingCommand::SetApplicationRouting {
                app_name: app_name.to_string(),
                routing,
            });
        }
    }
    
    /// Refresh application list
    pub fn refresh_applications(&self) {
        if let Some(sender) = &self.routing_sender {
            let _ = sender.send(AudioRoutingCommand::RefreshApplications);
        }
    }
}

impl Default for ApplicationAudioRouter {
    fn default() -> Self {
        Self::new()
    }
}