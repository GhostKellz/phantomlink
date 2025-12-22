mod phantomlink;
mod gui;
mod rnnoise;
mod audio;
mod scarlett;
mod config;
mod jack_client;
mod vst_host;
mod advanced_denoising;
mod ghostwave_integration;
mod pipewire;
mod ipc;
mod gpu;

use std::sync::{Arc, Mutex};
use eframe::egui;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("PhantomLink v{} starting...", env!("CARGO_PKG_VERSION"));

    // Initialize GPU manager
    {
        let manager = gpu::GpuManager::global().lock().unwrap();
        if manager.is_cuda_available() {
            log::info!(
                "GPU(s) detected: {} ({} device(s))",
                manager.gpu_names(),
                manager.gpu_count()
            );
            if let Some(gpu) = manager.get_selected_gpu() {
                log::info!(
                    "Selected GPU: {} ({} SM, {:.1} GB, {})",
                    gpu.name,
                    gpu.sm_count,
                    gpu.total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
                    gpu.architecture.name()
                );
            }
        } else {
            log::warn!("No CUDA-capable GPU detected, running in CPU mode");
        }
    }

    // Initialize audio engine
    let audio_engine = Arc::new(Mutex::new(audio::AudioEngine::new()));

    // Get GhostWave reference for IPC
    let ghostwave = audio_engine
        .lock()
        .ok()
        .and_then(|e| e.get_ghostwave().cloned());

    // Start IPC server in background
    let mut ipc_server = ipc::IpcServer::new(audio_engine.clone(), ghostwave);
    if let Err(e) = ipc_server.start() {
        log::warn!("Failed to start IPC server: {}", e);
    } else {
        log::info!("IPC server started");
    }

    // Enable high DPI support and modern GUI features
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_transparent(true),
        ..Default::default()
    };

    log::info!("Starting GUI...");

    eframe::run_native(
        "PhantomLink - Professional Audio Mixer",
        options,
        Box::new(|cc| {
            // Configure egui for better rendering
            cc.egui_ctx.set_pixels_per_point(1.0);
            Box::new(gui::PhantomlinkApp::default())
        }),
    )
    .unwrap();

    // Cleanup
    ipc_server.stop();
    log::info!("PhantomLink shutdown complete");
}
