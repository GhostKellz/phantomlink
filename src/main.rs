mod phantomlink;
mod gui;
mod rnnoise;
mod audio;
mod scarlett;
mod config;
mod jack_client;
mod vst_host;
mod advanced_denoising;
mod app_audio;
mod ghostnv_mock;
mod ghostnv;
mod ghostnv_bridge;

use eframe::egui;

#[tokio::main]
async fn main() {
    // Initialize tracing for GHOSTNV
    tracing_subscriber::fmt::init();
    // Enable high DPI support and modern GUI features
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_transparent(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "PhantomLink - Professional Audio Mixer",
        options,
        Box::new(|cc| {
            // Configure egui for better rendering
            cc.egui_ctx.set_pixels_per_point(1.0);
            Box::new(gui::PhantomlinkApp::default())
        }),
    ).unwrap();
}
