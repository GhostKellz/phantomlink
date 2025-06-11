use eframe::egui;
use crate::rnnoise::Rnnoise;
use crate::phantomlink;
use crate::scarlett::ScarlettSolo;
use crate::audio::AudioEngine;

const CHANNELS: &[&str] = &["System", "Voice", "Game", "App"];

pub struct ChannelStrip {
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub selected_vst: Option<usize>,
}

pub struct PhantomlinkApp {
    rnnoise: Rnnoise,
    vst_plugins: Vec<String>,
    channel_strips: Vec<ChannelStrip>,
    scarlett_gain: f32,
    scarlett_monitor: bool,
    scarlett: Option<ScarlettSolo>,
    scarlett_error: Option<String>,
    audio_engine: AudioEngine,
    audio_running: bool,
}

impl Default for PhantomlinkApp {
    fn default() -> Self {
        let vst_plugins = phantomlink::find_vst_plugins()
            .into_iter()
            .map(|p| p.display().to_string())
            .collect();
        let channel_strips = CHANNELS.iter().map(|&name| ChannelStrip {
            name: name.to_string(),
            volume: 0.8,
            muted: false,
            selected_vst: None,
        }).collect();
        let (scarlett, scarlett_error) = match ScarlettSolo::new() {
            Ok(dev) => (Some(dev), None),
            Err(e) => (None, Some(format!("ScarlettSolo error: {}", e))),
        };
        Self {
            rnnoise: Rnnoise::new(),
            vst_plugins,
            channel_strips,
            scarlett_gain: 0.5,
            scarlett_monitor: false,
            scarlett,
            scarlett_error,
            audio_engine: AudioEngine::new(),
            audio_running: false,
        }
    }
}

impl eframe::App for PhantomlinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set custom Phantomlink theme
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(80, 217, 176)); // #50d9b0
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(9, 13, 43); // #090d2b
        visuals.window_fill = egui::Color32::from_rgb(9, 13, 43); // #090d2b
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(19, 158, 209); // Accent blue #139ed1
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(80, 217, 176); // #50d9b0
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 30, 30); // dark gray
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(egui::RichText::new("Phantomlink Mixer").color(egui::Color32::from_rgb(80, 217, 176)).size(32.0));
            ui.label("Welcome! This will be your audio routing and mixing control panel.");
            ui.separator();
            // RNNoise toggle
            ui.horizontal(|ui| {
                ui.label("Noise Suppression (RNNoise):");
                let mut enabled = self.rnnoise.is_enabled();
                if ui.toggle_value(&mut enabled, "Enabled").changed() {
                    if enabled {
                        self.rnnoise.enable();
                    } else {
                        self.rnnoise.disable();
                    }
                }
            });
            ui.separator();
            // Channel strips
            ui.horizontal(|ui| {
                for (i, strip) in self.channel_strips.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.heading(&strip.name);
                        ui.add(egui::Slider::new(&mut strip.volume, 0.0..=1.0).text("Volume"));
                        if ui.button(if strip.muted { "Unmute" } else { "Mute" }).clicked() {
                            strip.muted = !strip.muted;
                        }
                        ui.label("VST Plugin:");
                        egui::ComboBox::from_id_source(format!("vst_select_{}", i))
                            .selected_text(strip.selected_vst
                                .and_then(|idx| self.vst_plugins.get(idx))
                                .map(|s| s.as_str()).unwrap_or("None"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut strip.selected_vst, None, "None");
                                for (idx, name) in self.vst_plugins.iter().enumerate() {
                                    ui.selectable_value(&mut strip.selected_vst, Some(idx), name);
                                }
                            });
                    });
                }
            });
            ui.separator();
            // Scarlett Solo controls (wired)
            ui.collapsing("Scarlett Solo Controls", |ui| {
                if let Some(ref scarlett) = self.scarlett {
                    let mut gain = self.scarlett_gain;
                    if ui.add(egui::Slider::new(&mut gain, 0.0..=1.0).text("Input Gain")).changed() {
                        self.scarlett_gain = gain;
                        let _ = scarlett.set_input_gain(gain);
                    }
                    if ui.checkbox(&mut self.scarlett_monitor, "Direct Monitor").changed() {
                        let _ = scarlett.set_direct_monitor(self.scarlett_monitor);
                    }
                } else if let Some(ref err) = self.scarlett_error {
                    ui.colored_label(egui::Color32::RED, err);
                } else {
                    ui.label("Scarlett Solo not detected.");
                }
            });
            ui.separator();
            // Audio streaming controls (stub)
            if !self.audio_running {
                if ui.button("Start Audio Engine").clicked() {
                    self.audio_engine.start();
                    self.audio_running = true;
                }
            } else {
                ui.label("Audio engine running...");
            }
        });
    }
}
