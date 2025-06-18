use eframe::egui;
use crate::gui::widgets;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AudioApplication {
    pub name: String,
    pub process_name: String,
    pub icon: String,
    pub volume: f32,
    pub muted: bool,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub input_enabled: bool,
    pub output_enabled: bool,
}

pub struct ApplicationManager {
    pub applications: Vec<AudioApplication>,
    pub detected_apps: HashMap<String, AudioApplication>,
    pub auto_detect: bool,
    pub show_system_apps: bool,
    pub filter_text: String,
    pub new_app_name: String,
    pub new_app_process: String,
}

impl Default for ApplicationManager {
    fn default() -> Self {
        let applications = vec![
            AudioApplication {
                name: "Discord".to_string(),
                process_name: "Discord".to_string(),
                icon: "🎮".to_string(),
                volume: 0.8,
                muted: false,
                is_running: false,
                pid: None,
                input_enabled: true,
                output_enabled: true,
            },
            AudioApplication {
                name: "Spotify".to_string(),
                process_name: "spotify".to_string(),
                icon: "🎵".to_string(),
                volume: 0.8,
                muted: false,
                is_running: false,
                pid: None,
                input_enabled: false,
                output_enabled: true,
            },
            AudioApplication {
                name: "Firefox".to_string(),
                process_name: "firefox".to_string(),
                icon: "🌐".to_string(),
                volume: 0.8,
                muted: false,
                is_running: false,
                pid: None,
                input_enabled: true,
                output_enabled: true,
            },
            AudioApplication {
                name: "OBS Studio".to_string(),
                process_name: "obs".to_string(),
                icon: "📹".to_string(),
                volume: 0.8,
                muted: false,
                is_running: false,
                pid: None,
                input_enabled: true,
                output_enabled: true,
            },
            AudioApplication {
                name: "Steam".to_string(),
                process_name: "steam".to_string(),
                icon: "🎮".to_string(),
                volume: 0.8,
                muted: false,
                is_running: false,
                pid: None,
                input_enabled: false,
                output_enabled: true,
            },
        ];

        Self {
            applications,
            detected_apps: HashMap::new(),
            auto_detect: true,
            show_system_apps: false,
            filter_text: String::new(),
            new_app_name: String::new(),
            new_app_process: String::new(),
        }
    }
}

impl ApplicationManager {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header controls
            self.render_header(ui);
            
            ui.add_space(10.0);
            
            // Application list
            self.render_application_list(ui);
            
            ui.add_space(10.0);
            
            // Add custom application
            self.render_add_application(ui);
        });
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("APPLICATION MIXER").size(16.0).color(egui::Color32::from_rgb(80, 217, 176)));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Refresh button
                if ui.add(widgets::glow_button("REFRESH", egui::Color32::from_rgb(19, 158, 209))).clicked() {
                    self.scan_for_applications();
                }
                
                ui.add_space(10.0);
                
                // Auto-detect toggle
                ui.checkbox(&mut self.auto_detect, "Auto-detect");
                
                ui.add_space(10.0);
                
                // Show system apps toggle
                ui.checkbox(&mut self.show_system_apps, "System apps");
            });
        });
        
        // Filter input
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);
        });
    }
    
    fn render_application_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                // Collect indices of applications to render to avoid borrowing issues
                let app_indices: Vec<usize> = self.applications
                    .iter()
                    .enumerate()
                    .filter_map(|(i, app)| {
                        if self.filter_text.is_empty() || 
                           app.name.to_lowercase().contains(&self.filter_text.to_lowercase()) {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                
                for &i in &app_indices {
                    Self::render_application_strip_static(ui, &mut self.applications[i]);
                    ui.add_space(5.0);
                }
                
                // Show detected apps if auto-detect is enabled
                if self.auto_detect {
                    ui.separator();
                    ui.label(egui::RichText::new("DETECTED APPLICATIONS").size(12.0));
                    
                    let detected_apps: Vec<String> = self.detected_apps
                        .iter()
                        .filter_map(|(name, app)| {
                            if app.is_running {
                                Some(name.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    for app_name in detected_apps {
                        if let Some(app) = self.detected_apps.get_mut(&app_name) {
                            Self::render_application_strip_static(ui, app);
                            ui.add_space(5.0);
                        }
                    }
                }
            });
    }
    
    fn render_application_strip_static(ui: &mut egui::Ui, app: &mut AudioApplication) {
        ui.horizontal(|ui| {
            // Status indicator
            let status_color = if app.is_running {
                egui::Color32::from_rgb(0, 255, 100)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            ui.colored_label(status_color, "●");
            
            // Icon and name
            ui.label(&app.icon);
            ui.label(&app.name);
            
            ui.add_space(10.0);
            
            // Input/Output toggles
            ui.label("IN:");
            ui.checkbox(&mut app.input_enabled, "");
            ui.label("OUT:");
            ui.checkbox(&mut app.output_enabled, "");
            
            ui.add_space(10.0);
            
            // Volume control
            ui.label("Vol:");
            ui.add(egui::Slider::new(&mut app.volume, 0.0..=1.0)
                .step_by(0.01)
                .show_value(false));
            
            ui.add_space(5.0);
            
            // Mute button
            let mute_color = if app.muted {
                egui::Color32::from_rgb(255, 100, 100)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("M", mute_color)).clicked() {
                app.muted = !app.muted;
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Remove button for custom apps
                if ui.button("✖").clicked() {
                    // Mark for removal
                }
                
                // Route button
                if ui.add(widgets::glow_button("ROUTE", egui::Color32::from_rgb(80, 217, 176))).clicked() {
                    // Open routing configuration
                }
            });
        });
    }
    
    fn render_add_application(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("ADD CUSTOM APPLICATION", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.new_app_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("Process:");
                ui.text_edit_singleline(&mut self.new_app_process);
            });
            
            ui.horizontal(|ui| {
                if ui.add(widgets::glow_button("ADD", egui::Color32::from_rgb(80, 217, 176))).clicked() {
                    if !self.new_app_name.is_empty() && !self.new_app_process.is_empty() {
                        let new_app = AudioApplication {
                            name: self.new_app_name.clone(),
                            process_name: self.new_app_process.clone(),
                            icon: "📱".to_string(),
                            volume: 0.8,
                            muted: false,
                            is_running: false,
                            pid: None,
                            input_enabled: true,
                            output_enabled: true,
                        };
                        self.applications.push(new_app);
                        self.new_app_name.clear();
                        self.new_app_process.clear();
                    }
                }
                
                if ui.button("Browse").clicked() {
                    // Open file browser for executable selection
                }
            });
        });
    }
    
    pub fn scan_for_applications(&mut self) {
        // Scan for running processes using ps command
        if let Ok(output) = Command::new("ps")
            .args(&["-eo", "pid,comm"])
            .output() {
            
            let ps_output = String::from_utf8_lossy(&output.stdout);
            
            // Update running status for known applications
            for app in &mut self.applications {
                app.is_running = false;
                app.pid = None;
                
                for line in ps_output.lines() {
                    if line.to_lowercase().contains(&app.process_name.to_lowercase()) {
                        app.is_running = true;
                        if let Ok(pid) = line.split_whitespace().next().unwrap_or("0").parse::<u32>() {
                            app.pid = Some(pid);
                        }
                        break;
                    }
                }
            }
            
            // Detect new applications with audio capabilities
            if self.auto_detect {
                self.detect_audio_applications(&ps_output);
            }
        }
    }
    
    fn detect_audio_applications(&mut self, ps_output: &str) {
        let audio_apps = [
            ("chrome", "🌐"),
            ("chromium", "🌐"),
            ("vlc", "📺"),
            ("mpv", "📺"),
            ("audacity", "🎤"),
            ("pavucontrol", "🔊"),
            ("pulseaudio", "🔊"),
            ("pipewire", "🔊"),
            ("qemu", "💻"),
            ("virtualbox", "💻"),
        ];
        
        for (process, icon) in &audio_apps {
            if ps_output.to_lowercase().contains(process) {
                let app_name = process.to_string();
                if !self.detected_apps.contains_key(&app_name) &&
                   !self.applications.iter().any(|a| a.process_name.to_lowercase() == *process) {
                    
                    let detected_app = AudioApplication {
                        name: app_name.clone(),
                        process_name: process.to_string(),
                        icon: icon.to_string(),
                        volume: 0.8,
                        muted: false,
                        is_running: true,
                        pid: None,
                        input_enabled: false,
                        output_enabled: true,
                    };
                    
                    self.detected_apps.insert(app_name, detected_app);
                }
            }
        }
    }
    
    pub fn get_application_volume(&self, process_name: &str) -> f32 {
        for app in &self.applications {
            if app.process_name == process_name {
                return if app.muted { 0.0 } else { app.volume };
            }
        }
        
        if let Some(app) = self.detected_apps.get(process_name) {
            return if app.muted { 0.0 } else { app.volume };
        }
        
        1.0
    }
}