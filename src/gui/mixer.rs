use eframe::egui;
use crate::gui::widgets;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MixerMode {
    MonitorMix,
    StreamMix,
    Recording,
}

#[derive(Debug, Clone)]
pub struct MixerOutput {
    pub name: String,
    pub enabled: bool,
    pub volume: f32,
    pub channels: HashMap<usize, f32>, // channel_id -> volume
}

pub struct MixerPanel {
    pub mode: MixerMode,
    pub outputs: Vec<MixerOutput>,
    pub selected_output: usize,
    pub show_routing_matrix: bool,
    pub eq_enabled: bool,
    pub compressor_enabled: bool,
    pub gate_enabled: bool,
}

impl Default for MixerPanel {
    fn default() -> Self {
        let mut outputs = vec![
            MixerOutput {
                name: "🎧 Monitor Mix".to_string(),
                enabled: true,
                volume: 0.8,
                channels: HashMap::new(),
            },
            MixerOutput {
                name: "📺 Stream Mix".to_string(),
                enabled: true,
                volume: 0.8,
                channels: HashMap::new(),
            },
            MixerOutput {
                name: "Chat Mix".to_string(),
                enabled: false,
                volume: 0.8,
                channels: HashMap::new(),
            },
            MixerOutput {
                name: "Recording".to_string(),
                enabled: false,
                volume: 0.8,
                channels: HashMap::new(),
            },
        ];

        // Initialize default routing
        for output in &mut outputs {
            for i in 0..4 {
                output.channels.insert(i, 0.8);
            }
        }

        Self {
            mode: MixerMode::MonitorMix,
            outputs,
            selected_output: 0,
            show_routing_matrix: false,
            eq_enabled: false,
            compressor_enabled: false,
            gate_enabled: false,
        }
    }
}

impl MixerPanel {
    pub fn render(&mut self, ui: &mut egui::Ui, channel_names: &[String]) {
        ui.vertical(|ui| {
            // Header with mixer mode tabs
            self.render_mixer_tabs(ui);
            
            ui.add_space(10.0);
            
            // Output selector
            self.render_output_selector(ui);
            
            ui.add_space(15.0);
            
            // Main mixer matrix
            if self.show_routing_matrix {
                self.render_routing_matrix(ui, channel_names);
            } else {
                self.render_simple_mixer(ui, channel_names);
            }
            
            ui.add_space(15.0);
            
            // Advanced controls
            self.render_advanced_controls(ui);
        });
    }
    
    fn render_mixer_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("MIXER MODE").size(14.0).color(egui::Color32::from_rgb(80, 217, 176)));
            
            ui.add_space(20.0);
            
            // Tab buttons
            let monitor_color = if self.mode == MixerMode::MonitorMix {
                egui::Color32::from_rgb(80, 217, 176)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("🎧 MONITOR", monitor_color)).clicked() {
                self.mode = MixerMode::MonitorMix;
                self.selected_output = 0;
            }
            
            let stream_color = if self.mode == MixerMode::StreamMix {
                egui::Color32::from_rgb(80, 217, 176)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("📺 STREAM", stream_color)).clicked() {
                self.mode = MixerMode::StreamMix;
                self.selected_output = 1;
            }
            
            let rec_color = if self.mode == MixerMode::Recording {
                egui::Color32::from_rgb(255, 100, 100)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("🔴 RECORD", rec_color)).clicked() {
                self.mode = MixerMode::Recording;
                self.selected_output = 3;
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let matrix_text = if self.show_routing_matrix { "SIMPLE" } else { "MATRIX" };
                if ui.add(widgets::glow_button(matrix_text, egui::Color32::from_rgb(19, 158, 209))).clicked() {
                    self.show_routing_matrix = !self.show_routing_matrix;
                }
            });
        });
    }
    
    fn render_output_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("OUTPUT:");
            
            egui::ComboBox::from_id_source("output_selector")
                .selected_text(&self.outputs[self.selected_output].name)
                .show_ui(ui, |ui| {
                    for (i, output) in self.outputs.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_output, i, &output.name);
                    }
                });
            
            ui.add_space(10.0);
            
            // Output enable/disable
            let mut enabled = self.outputs[self.selected_output].enabled;
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                self.outputs[self.selected_output].enabled = enabled;
            }
            
            ui.add_space(10.0);
            
            // Master output volume
            ui.label("Master:");
            ui.add(egui::Slider::new(&mut self.outputs[self.selected_output].volume, 0.0..=1.0)
                .step_by(0.01)
                .show_value(false));
        });
    }
    
    fn render_simple_mixer(&mut self, ui: &mut egui::Ui, channel_names: &[String]) {
        ui.horizontal(|ui| {
            for (i, channel_name) in channel_names.iter().enumerate() {
                ui.vertical(|ui| {
                    ui.set_width(80.0);
                    
                    // Channel name
                    ui.label(egui::RichText::new(channel_name).size(12.0).color(egui::Color32::from_rgb(80, 217, 176)));
                    
                    ui.add_space(5.0);
                    
                    // Channel level to this output
                    let current_volume = self.outputs[self.selected_output]
                        .channels.get(&i).copied().unwrap_or(0.0);
                    
                    let mut volume = current_volume;
                    if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0)
                        .vertical()
                        .step_by(0.01)
                        .show_value(false)).changed() {
                        self.outputs[self.selected_output].channels.insert(i, volume);
                    }
                    
                    ui.add_space(5.0);
                    
                    // Mute button for this channel in this output
                    let is_muted = volume == 0.0;
                    let mute_color = if is_muted {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else {
                        egui::Color32::from_rgb(100, 100, 100)
                    };
                    
                    if ui.add(widgets::glow_button("M", mute_color)).clicked() {
                        if is_muted {
                            self.outputs[self.selected_output].channels.insert(i, 0.8);
                        } else {
                            self.outputs[self.selected_output].channels.insert(i, 0.0);
                        }
                    }
                    
                    // Solo button
                    if ui.add(widgets::glow_button("S", egui::Color32::from_rgb(255, 200, 100))).clicked() {
                        // Solo logic - mute all other channels
                        for (channel_id, _) in self.outputs[self.selected_output].channels.clone() {
                            if channel_id != i {
                                self.outputs[self.selected_output].channels.insert(channel_id, 0.0);
                            }
                        }
                        self.outputs[self.selected_output].channels.insert(i, 0.8);
                    }
                });
                
                if i < channel_names.len() - 1 {
                    ui.add_space(10.0);
                }
            }
        });
    }
    
    fn render_routing_matrix(&mut self, ui: &mut egui::Ui, channel_names: &[String]) {
        ui.label(egui::RichText::new("ROUTING MATRIX").size(16.0).color(egui::Color32::from_rgb(80, 217, 176)));
        ui.add_space(10.0);
        
        // Matrix grid
        egui::Grid::new("routing_matrix")
            .num_columns(channel_names.len() + 1)
            .spacing([5.0, 5.0])
            .show(ui, |ui| {
                // Header row
                ui.label("OUTPUT \\ INPUT");
                for channel_name in channel_names {
                    ui.label(egui::RichText::new(channel_name).size(10.0));
                }
                ui.end_row();
                
                // Output rows
                for (output_idx, output) in self.outputs.iter_mut().enumerate() {
                    ui.label(&output.name);
                    
                    for channel_idx in 0..channel_names.len() {
                        let current_level = output.channels.get(&channel_idx).copied().unwrap_or(0.0);
                        let mut level = current_level;
                        
                        if ui.add(egui::Slider::new(&mut level, 0.0..=1.0)
                            .step_by(0.01)
                            .show_value(false)).changed() {
                            output.channels.insert(channel_idx, level);
                        }
                    }
                    ui.end_row();
                }
            });
    }
    
    fn render_advanced_controls(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("AUDIO PROCESSING", |ui| {
            ui.horizontal(|ui| {
                // EQ Toggle
                let eq_color = if self.eq_enabled {
                    egui::Color32::from_rgb(80, 217, 176)
                } else {
                    egui::Color32::from_rgb(100, 100, 100)
                };
                
                if ui.add(widgets::glow_button("EQ", eq_color)).clicked() {
                    self.eq_enabled = !self.eq_enabled;
                }
                
                // Compressor Toggle
                let comp_color = if self.compressor_enabled {
                    egui::Color32::from_rgb(80, 217, 176)
                } else {
                    egui::Color32::from_rgb(100, 100, 100)
                };
                
                if ui.add(widgets::glow_button("COMP", comp_color)).clicked() {
                    self.compressor_enabled = !self.compressor_enabled;
                }
                
                // Gate Toggle
                let gate_color = if self.gate_enabled {
                    egui::Color32::from_rgb(80, 217, 176)
                } else {
                    egui::Color32::from_rgb(100, 100, 100)
                };
                
                if ui.add(widgets::glow_button("GATE", gate_color)).clicked() {
                    self.gate_enabled = !self.gate_enabled;
                }
            });
            
            if self.eq_enabled {
                ui.separator();
                ui.label("3-Band EQ");
                ui.horizontal(|ui| {
                    ui.label("Low:");
                    ui.add(egui::Slider::new(&mut 0.0f32, -12.0..=12.0).suffix(" dB"));
                    ui.label("Mid:");
                    ui.add(egui::Slider::new(&mut 0.0f32, -12.0..=12.0).suffix(" dB"));
                    ui.label("High:");
                    ui.add(egui::Slider::new(&mut 0.0f32, -12.0..=12.0).suffix(" dB"));
                });
            }
            
            if self.compressor_enabled {
                ui.separator();
                ui.label("Compressor");
                ui.horizontal(|ui| {
                    ui.label("Threshold:");
                    ui.add(egui::Slider::new(&mut 0.0f32, -40.0..=0.0).suffix(" dB"));
                    ui.label("Ratio:");
                    ui.add(egui::Slider::new(&mut 2.0f32, 1.0..=10.0).suffix(":1"));
                });
            }
            
            if self.gate_enabled {
                ui.separator();
                ui.label("Noise Gate");
                ui.horizontal(|ui| {
                    ui.label("Threshold:");
                    ui.add(egui::Slider::new(&mut -30.0f32, -60.0..=0.0).suffix(" dB"));
                    ui.label("Hold:");
                    ui.add(egui::Slider::new(&mut 10.0f32, 1.0..=1000.0).suffix(" ms"));
                });
            }
        });
    }
    
    pub fn get_channel_output_level(&self, channel_idx: usize, output_idx: usize) -> f32 {
        if output_idx < self.outputs.len() {
            self.outputs[output_idx].channels.get(&channel_idx).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    }
    
    pub fn is_output_enabled(&self, output_idx: usize) -> bool {
        if output_idx < self.outputs.len() {
            self.outputs[output_idx].enabled
        } else {
            false
        }
    }
}