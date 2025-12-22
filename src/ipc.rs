//! JSON-RPC IPC Server for PhantomLink external control.
//!
//! Provides remote control interface via Unix domain sockets for:
//! - Mixer channel control (volume, mute, gain, pan)
//! - GhostWave AI denoising configuration
//! - VST plugin management
//! - System status queries
//!
//! ## Usage
//! Connect via Unix socket at `/run/user/<uid>/phantomlink.sock` or
//! `/tmp/phantomlink.sock` as fallback.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::audio::AudioEngine;
use crate::ghostwave_integration::{
    DenoiseQuality, GhostWaveIntegration, LatencyMode, PhantomLinkProfile, RtxStatus,
    StatusHealth,
};

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<serde_json::Value>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

// Standard JSON-RPC error codes
const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;

/// Channel state for IPC queries
#[derive(Debug, Serialize, Clone)]
pub struct ChannelState {
    pub index: usize,
    pub volume: f32,
    pub muted: bool,
    pub gain: f32,
    pub pan: f32,
    pub peak_level: f32,
    pub rms_level: f32,
}

/// GhostWave state for IPC queries
#[derive(Debug, Serialize, Clone)]
pub struct GhostWaveState {
    pub enabled: bool,
    pub profile: String,
    pub latency_mode: String,
    pub noise_strength: f32,
    pub denoise_quality: String,
    pub rtx_status: RtxStatusInfo,
    pub metrics: ProcessingMetricsInfo,
    pub health: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct RtxStatusInfo {
    pub available: bool,
    pub gpu_name: String,
    pub driver_version: String,
    pub precision: String,
    pub memory_used_mb: f32,
    pub memory_total_mb: f32,
    pub tensor_cores: bool,
    pub fp4_support: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProcessingMetricsInfo {
    pub latency_ms: f32,
    pub cpu_usage: f32,
    pub gpu_usage: f32,
    pub frames_processed: u64,
}

/// Full system state for IPC queries
#[derive(Debug, Serialize, Clone)]
pub struct SystemState {
    pub version: String,
    pub audio_running: bool,
    pub ghostwave: Option<GhostWaveState>,
    pub channels: Vec<ChannelState>,
    pub rtx_available: bool,
    pub gpu_name: String,
}

/// IPC Server for PhantomLink remote control
pub struct IpcServer {
    socket_path: PathBuf,
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
    audio_engine: Arc<Mutex<AudioEngine>>,
    ghostwave: Option<Arc<Mutex<GhostWaveIntegration>>>,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new(
        audio_engine: Arc<Mutex<AudioEngine>>,
        ghostwave: Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> Self {
        let socket_path = Self::get_socket_path();

        Self {
            socket_path,
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            audio_engine,
            ghostwave,
        }
    }

    /// Get the socket path for the IPC server
    fn get_socket_path() -> PathBuf {
        // Try XDG_RUNTIME_DIR first (e.g., /run/user/1000/phantomlink.sock)
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            return PathBuf::from(runtime_dir).join("phantomlink.sock");
        }

        // Fallback to /tmp
        PathBuf::from("/tmp/phantomlink.sock")
    }

    /// Start the IPC server
    pub fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Remove existing socket file if present
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .context("Failed to remove existing socket file")?;
        }

        let listener =
            UnixListener::bind(&self.socket_path).context("Failed to bind Unix socket")?;

        log::info!("IPC server listening on {:?}", self.socket_path);

        self.running.store(true, Ordering::Relaxed);
        let running = self.running.clone();
        let audio_engine = self.audio_engine.clone();
        let ghostwave = self.ghostwave.clone();

        let handle = thread::spawn(move || {
            // Set socket to non-blocking so we can check the running flag
            listener
                .set_nonblocking(true)
                .expect("Failed to set non-blocking");

            while running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _addr)) => {
                        let audio_engine = audio_engine.clone();
                        let ghostwave = ghostwave.clone();

                        // Handle each client in a separate thread
                        thread::spawn(move || {
                            if let Err(e) =
                                Self::handle_client(stream, audio_engine, ghostwave)
                            {
                                log::error!("IPC client error: {}", e);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No incoming connection, sleep briefly
                        thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(e) => {
                        log::error!("IPC accept error: {}", e);
                    }
                }
            }

            log::info!("IPC server stopped");
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Stop the IPC server
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        // Clean up socket file
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }

    /// Handle a client connection
    fn handle_client(
        stream: UnixStream,
        audio_engine: Arc<Mutex<AudioEngine>>,
        ghostwave: Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = stream;

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // Client disconnected
                Ok(_) => {
                    let response = Self::handle_request(&line, &audio_engine, &ghostwave);
                    let response_json = serde_json::to_string(&response)? + "\n";
                    writer.write_all(response_json.as_bytes())?;
                    writer.flush()?;
                }
                Err(e) => {
                    log::error!("IPC read error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    fn handle_request(
        request_str: &str,
        audio_engine: &Arc<Mutex<AudioEngine>>,
        ghostwave: &Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> JsonRpcResponse {
        // Parse JSON
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcResponse::error(None, PARSE_ERROR, format!("Parse error: {}", e));
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return JsonRpcResponse::error(
                request.id,
                INVALID_REQUEST,
                "Invalid JSON-RPC version".to_string(),
            );
        }

        // Dispatch method
        match request.method.as_str() {
            // System methods
            "system.status" => Self::handle_system_status(request.id, audio_engine, ghostwave),
            "system.version" => Self::handle_system_version(request.id),

            // Mixer methods
            "mixer.get_channels" => Self::handle_get_channels(request.id, audio_engine),
            "mixer.set_volume" => {
                Self::handle_set_volume(request.id, request.params, audio_engine)
            }
            "mixer.set_mute" => Self::handle_set_mute(request.id, request.params, audio_engine),
            "mixer.set_gain" => Self::handle_set_gain(request.id, request.params, audio_engine),
            "mixer.set_pan" => Self::handle_set_pan(request.id, request.params, audio_engine),

            // GhostWave methods
            "ghostwave.status" => Self::handle_ghostwave_status(request.id, ghostwave),
            "ghostwave.enable" => Self::handle_ghostwave_enable(request.id, request.params, ghostwave),
            "ghostwave.set_profile" => {
                Self::handle_ghostwave_set_profile(request.id, request.params, ghostwave)
            }
            "ghostwave.set_strength" => {
                Self::handle_ghostwave_set_strength(request.id, request.params, ghostwave)
            }
            "ghostwave.set_latency_mode" => {
                Self::handle_ghostwave_set_latency_mode(request.id, request.params, ghostwave)
            }
            "ghostwave.restart_gpu" => Self::handle_ghostwave_restart_gpu(request.id, ghostwave),

            // Unknown method
            _ => JsonRpcResponse::error(
                request.id,
                METHOD_NOT_FOUND,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    // ===== System Methods =====

    fn handle_system_status(
        id: Option<serde_json::Value>,
        audio_engine: &Arc<Mutex<AudioEngine>>,
        ghostwave: &Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> JsonRpcResponse {
        let audio_running = audio_engine
            .lock()
            .map(|e| e.is_running())
            .unwrap_or(false);

        let gw_state = ghostwave.as_ref().and_then(|gw| {
            gw.lock().ok().map(|g| GhostWaveState {
                enabled: g.is_enabled(),
                profile: g.get_profile().name().to_string(),
                latency_mode: g.get_latency_mode().name().to_string(),
                noise_strength: g.get_noise_strength(),
                denoise_quality: g.get_denoise_quality().name().to_string(),
                rtx_status: {
                    let s = g.get_rtx_status();
                    RtxStatusInfo {
                        available: s.available,
                        gpu_name: s.gpu_name.clone(),
                        driver_version: s.driver_version.clone(),
                        precision: s.precision.clone(),
                        memory_used_mb: s.memory_used_mb,
                        memory_total_mb: s.memory_total_mb,
                        tensor_cores: s.tensor_cores,
                        fp4_support: s.fp4_support,
                    }
                },
                metrics: {
                    let m = g.get_metrics();
                    ProcessingMetricsInfo {
                        latency_ms: m.latency_ms,
                        cpu_usage: m.cpu_usage,
                        gpu_usage: m.gpu_usage,
                        frames_processed: m.frames_processed,
                    }
                },
                health: g.get_status_health().name().to_string(),
            })
        });

        let gpu_name = ghostwave
            .as_ref()
            .and_then(|gw| gw.lock().ok().map(|g| g.get_rtx_status().gpu_name.clone()))
            .unwrap_or_else(|| "Unknown".to_string());

        let rtx_available = ghostwave
            .as_ref()
            .and_then(|gw| gw.lock().ok().map(|g| g.get_rtx_status().available))
            .unwrap_or(false);

        // Get channel states
        let channels: Vec<ChannelState> = (0..4)
            .filter_map(|i| {
                audio_engine.lock().ok().and_then(|e| {
                    e.get_channel_levels(i).map(|levels| ChannelState {
                        index: i,
                        volume: 0.8, // TODO: Get actual volume
                        muted: false,
                        gain: 0.0,
                        pan: 0.0,
                        peak_level: levels[0],
                        rms_level: levels[1],
                    })
                })
            })
            .collect();

        let state = SystemState {
            version: env!("CARGO_PKG_VERSION").to_string(),
            audio_running,
            ghostwave: gw_state,
            channels,
            rtx_available,
            gpu_name,
        };

        JsonRpcResponse::success(id, serde_json::to_value(state).unwrap())
    }

    fn handle_system_version(id: Option<serde_json::Value>) -> JsonRpcResponse {
        let version = serde_json::json!({
            "name": "PhantomLink",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Professional audio mixer with RTX AI noise suppression"
        });
        JsonRpcResponse::success(id, version)
    }

    // ===== Mixer Methods =====

    fn handle_get_channels(
        id: Option<serde_json::Value>,
        audio_engine: &Arc<Mutex<AudioEngine>>,
    ) -> JsonRpcResponse {
        let channels: Vec<ChannelState> = (0..4)
            .filter_map(|i| {
                audio_engine.lock().ok().and_then(|e| {
                    e.get_channel_levels(i).map(|levels| ChannelState {
                        index: i,
                        volume: 0.8,
                        muted: false,
                        gain: 0.0,
                        pan: 0.0,
                        peak_level: levels[0],
                        rms_level: levels[1],
                    })
                })
            })
            .collect();

        JsonRpcResponse::success(id, serde_json::to_value(channels).unwrap())
    }

    fn handle_set_volume(
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
        audio_engine: &Arc<Mutex<AudioEngine>>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string())
            }
        };

        let channel: usize = match params.get("channel").and_then(|v| v.as_u64()) {
            Some(c) => c as usize,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing channel parameter".to_string(),
                )
            }
        };

        let volume: f32 = match params.get("volume").and_then(|v| v.as_f64()) {
            Some(v) => v as f32,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing volume parameter".to_string(),
                )
            }
        };

        if let Ok(engine) = audio_engine.lock() {
            engine.update_channel(channel, volume, false);
            JsonRpcResponse::success(id, serde_json::json!({"success": true}))
        } else {
            JsonRpcResponse::error(id, INTERNAL_ERROR, "Failed to lock audio engine".to_string())
        }
    }

    fn handle_set_mute(
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
        audio_engine: &Arc<Mutex<AudioEngine>>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string())
            }
        };

        let channel: usize = match params.get("channel").and_then(|v| v.as_u64()) {
            Some(c) => c as usize,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing channel parameter".to_string(),
                )
            }
        };

        let muted: bool = match params.get("muted").and_then(|v| v.as_bool()) {
            Some(m) => m,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing muted parameter".to_string(),
                )
            }
        };

        if let Ok(engine) = audio_engine.lock() {
            engine.update_channel(channel, 0.8, muted); // TODO: Preserve volume
            JsonRpcResponse::success(id, serde_json::json!({"success": true}))
        } else {
            JsonRpcResponse::error(id, INTERNAL_ERROR, "Failed to lock audio engine".to_string())
        }
    }

    fn handle_set_gain(
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
        audio_engine: &Arc<Mutex<AudioEngine>>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string())
            }
        };

        let channel: usize = match params.get("channel").and_then(|v| v.as_u64()) {
            Some(c) => c as usize,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing channel parameter".to_string(),
                )
            }
        };

        let gain: f32 = match params.get("gain").and_then(|v| v.as_f64()) {
            Some(g) => g as f32,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing gain parameter".to_string(),
                )
            }
        };

        if let Ok(engine) = audio_engine.lock() {
            engine.update_channel_advanced(channel, 0.8, false, gain, 0.0);
            JsonRpcResponse::success(id, serde_json::json!({"success": true}))
        } else {
            JsonRpcResponse::error(id, INTERNAL_ERROR, "Failed to lock audio engine".to_string())
        }
    }

    fn handle_set_pan(
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
        audio_engine: &Arc<Mutex<AudioEngine>>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string())
            }
        };

        let channel: usize = match params.get("channel").and_then(|v| v.as_u64()) {
            Some(c) => c as usize,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing channel parameter".to_string(),
                )
            }
        };

        let pan: f32 = match params.get("pan").and_then(|v| v.as_f64()) {
            Some(p) => p as f32,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing pan parameter".to_string(),
                )
            }
        };

        if let Ok(engine) = audio_engine.lock() {
            engine.update_channel_advanced(channel, 0.8, false, 0.0, pan);
            JsonRpcResponse::success(id, serde_json::json!({"success": true}))
        } else {
            JsonRpcResponse::error(id, INTERNAL_ERROR, "Failed to lock audio engine".to_string())
        }
    }

    // ===== GhostWave Methods =====

    fn handle_ghostwave_status(
        id: Option<serde_json::Value>,
        ghostwave: &Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> JsonRpcResponse {
        match ghostwave {
            Some(gw) => match gw.lock() {
                Ok(g) => {
                    let state = GhostWaveState {
                        enabled: g.is_enabled(),
                        profile: g.get_profile().name().to_string(),
                        latency_mode: g.get_latency_mode().name().to_string(),
                        noise_strength: g.get_noise_strength(),
                        denoise_quality: g.get_denoise_quality().name().to_string(),
                        rtx_status: {
                            let s = g.get_rtx_status();
                            RtxStatusInfo {
                                available: s.available,
                                gpu_name: s.gpu_name.clone(),
                                driver_version: s.driver_version.clone(),
                                precision: s.precision.clone(),
                                memory_used_mb: s.memory_used_mb,
                                memory_total_mb: s.memory_total_mb,
                                tensor_cores: s.tensor_cores,
                                fp4_support: s.fp4_support,
                            }
                        },
                        metrics: {
                            let m = g.get_metrics();
                            ProcessingMetricsInfo {
                                latency_ms: m.latency_ms,
                                cpu_usage: m.cpu_usage,
                                gpu_usage: m.gpu_usage,
                                frames_processed: m.frames_processed,
                            }
                        },
                        health: g.get_status_health().name().to_string(),
                    };
                    JsonRpcResponse::success(id, serde_json::to_value(state).unwrap())
                }
                Err(_) => JsonRpcResponse::error(
                    id,
                    INTERNAL_ERROR,
                    "Failed to lock GhostWave".to_string(),
                ),
            },
            None => JsonRpcResponse::error(
                id,
                INTERNAL_ERROR,
                "GhostWave not available".to_string(),
            ),
        }
    }

    fn handle_ghostwave_enable(
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
        ghostwave: &Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string())
            }
        };

        let enabled: bool = match params.get("enabled").and_then(|v| v.as_bool()) {
            Some(e) => e,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing enabled parameter".to_string(),
                )
            }
        };

        match ghostwave {
            Some(gw) => match gw.lock() {
                Ok(mut g) => {
                    g.set_enabled(enabled);
                    JsonRpcResponse::success(id, serde_json::json!({"success": true}))
                }
                Err(_) => JsonRpcResponse::error(
                    id,
                    INTERNAL_ERROR,
                    "Failed to lock GhostWave".to_string(),
                ),
            },
            None => JsonRpcResponse::error(
                id,
                INTERNAL_ERROR,
                "GhostWave not available".to_string(),
            ),
        }
    }

    fn handle_ghostwave_set_profile(
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
        ghostwave: &Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string())
            }
        };

        let profile_str: &str = match params.get("profile").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing profile parameter".to_string(),
                )
            }
        };

        let profile = match profile_str.to_lowercase().as_str() {
            "xlr_studio" | "xlrstudio" | "xlr studio" => PhantomLinkProfile::XlrStudio,
            "streaming" => PhantomLinkProfile::Streaming,
            "balanced" => PhantomLinkProfile::Balanced,
            "music" => PhantomLinkProfile::Music,
            _ => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    format!("Unknown profile: {}", profile_str),
                )
            }
        };

        match ghostwave {
            Some(gw) => match gw.lock() {
                Ok(mut g) => match g.set_profile(profile) {
                    Ok(_) => JsonRpcResponse::success(id, serde_json::json!({"success": true})),
                    Err(e) => JsonRpcResponse::error(
                        id,
                        INTERNAL_ERROR,
                        format!("Failed to set profile: {}", e),
                    ),
                },
                Err(_) => JsonRpcResponse::error(
                    id,
                    INTERNAL_ERROR,
                    "Failed to lock GhostWave".to_string(),
                ),
            },
            None => JsonRpcResponse::error(
                id,
                INTERNAL_ERROR,
                "GhostWave not available".to_string(),
            ),
        }
    }

    fn handle_ghostwave_set_strength(
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
        ghostwave: &Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string())
            }
        };

        let strength: f32 = match params.get("strength").and_then(|v| v.as_f64()) {
            Some(s) => s as f32,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing strength parameter".to_string(),
                )
            }
        };

        match ghostwave {
            Some(gw) => match gw.lock() {
                Ok(mut g) => match g.set_noise_strength(strength) {
                    Ok(_) => JsonRpcResponse::success(id, serde_json::json!({"success": true})),
                    Err(e) => JsonRpcResponse::error(
                        id,
                        INTERNAL_ERROR,
                        format!("Failed to set strength: {}", e),
                    ),
                },
                Err(_) => JsonRpcResponse::error(
                    id,
                    INTERNAL_ERROR,
                    "Failed to lock GhostWave".to_string(),
                ),
            },
            None => JsonRpcResponse::error(
                id,
                INTERNAL_ERROR,
                "GhostWave not available".to_string(),
            ),
        }
    }

    fn handle_ghostwave_set_latency_mode(
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
        ghostwave: &Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string())
            }
        };

        let mode_str: &str = match params.get("mode").and_then(|v| v.as_str()) {
            Some(m) => m,
            None => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    "Missing mode parameter".to_string(),
                )
            }
        };

        let mode = match mode_str.to_lowercase().as_str() {
            "low_latency" | "lowlatency" | "low latency" => LatencyMode::LowLatency,
            "balanced" => LatencyMode::Balanced,
            "high_quality" | "highquality" | "high quality" => LatencyMode::HighQuality,
            _ => {
                return JsonRpcResponse::error(
                    id,
                    INVALID_PARAMS,
                    format!("Unknown latency mode: {}", mode_str),
                )
            }
        };

        match ghostwave {
            Some(gw) => match gw.lock() {
                Ok(mut g) => {
                    g.set_latency_mode(mode);
                    JsonRpcResponse::success(id, serde_json::json!({"success": true}))
                }
                Err(_) => JsonRpcResponse::error(
                    id,
                    INTERNAL_ERROR,
                    "Failed to lock GhostWave".to_string(),
                ),
            },
            None => JsonRpcResponse::error(
                id,
                INTERNAL_ERROR,
                "GhostWave not available".to_string(),
            ),
        }
    }

    fn handle_ghostwave_restart_gpu(
        id: Option<serde_json::Value>,
        ghostwave: &Option<Arc<Mutex<GhostWaveIntegration>>>,
    ) -> JsonRpcResponse {
        match ghostwave {
            Some(gw) => match gw.lock() {
                Ok(mut g) => match g.restart_gpu() {
                    Ok(_) => JsonRpcResponse::success(id, serde_json::json!({"success": true})),
                    Err(e) => JsonRpcResponse::error(
                        id,
                        INTERNAL_ERROR,
                        format!("Failed to restart GPU: {}", e),
                    ),
                },
                Err(_) => JsonRpcResponse::error(
                    id,
                    INTERNAL_ERROR,
                    "Failed to lock GhostWave".to_string(),
                ),
            },
            None => JsonRpcResponse::error(
                id,
                INTERNAL_ERROR,
                "GhostWave not available".to_string(),
            ),
        }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = IpcServer::get_socket_path();
        assert!(path.to_string_lossy().contains("phantomlink.sock"));
    }

    #[test]
    fn test_json_rpc_response() {
        let response = JsonRpcResponse::success(Some(serde_json::json!(1)), serde_json::json!({"test": true}));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let error = JsonRpcResponse::error(Some(serde_json::json!(2)), -32600, "Test error".to_string());
        assert!(error.result.is_none());
        assert!(error.error.is_some());
    }
}
