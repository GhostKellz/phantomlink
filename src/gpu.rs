//! Multi-GPU management for PhantomLink.
//!
//! Provides GPU enumeration, selection, and load balancing for:
//! - CUDA/RTX acceleration
//! - ONNX Runtime GPU execution
//! - GhostWave integration

#![allow(dead_code)]

use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use std::ffi::{c_int, c_void, CStr};
use std::sync::{Arc, Mutex, OnceLock};

/// GPU architecture generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpuArchitecture {
    /// Pre-Turing (no Tensor Cores)
    Legacy,
    /// Turing (RTX 20xx) - Tensor Cores Gen 1
    Turing,
    /// Ampere (RTX 30xx) - Tensor Cores Gen 3
    Ampere,
    /// Ada Lovelace (RTX 40xx) - Tensor Cores Gen 4
    Ada,
    /// Blackwell (RTX 50xx) - Tensor Cores Gen 5
    Blackwell,
}

impl GpuArchitecture {
    /// Detect architecture from compute capability
    pub fn from_compute_capability(major: i32, minor: i32) -> Self {
        match major {
            12 => Self::Blackwell,
            8 if minor >= 9 => Self::Ada,
            8 => Self::Ampere,
            7 if minor >= 5 => Self::Turing,
            _ => Self::Legacy,
        }
    }

    /// Check if this architecture supports Tensor Cores
    pub fn has_tensor_cores(&self) -> bool {
        matches!(self, Self::Turing | Self::Ampere | Self::Ada | Self::Blackwell)
    }

    /// Check if this architecture supports FP4 (Blackwell only)
    pub fn has_fp4(&self) -> bool {
        matches!(self, Self::Blackwell)
    }

    /// Check if this architecture supports FP8 (Ada and Blackwell)
    pub fn has_fp8(&self) -> bool {
        matches!(self, Self::Ada | Self::Blackwell)
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Legacy => "Legacy",
            Self::Turing => "Turing",
            Self::Ampere => "Ampere",
            Self::Ada => "Ada Lovelace",
            Self::Blackwell => "Blackwell",
        }
    }
}

/// Information about a single GPU
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// Device index (0-based)
    pub index: usize,
    /// Device name (e.g., "NVIDIA GeForce RTX 5090")
    pub name: String,
    /// Compute capability major version
    pub compute_major: i32,
    /// Compute capability minor version
    pub compute_minor: i32,
    /// Architecture generation
    pub architecture: GpuArchitecture,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Free memory in bytes (at query time)
    pub free_memory: u64,
    /// Memory clock in MHz
    pub memory_clock_mhz: u32,
    /// SM (streaming multiprocessor) count
    pub sm_count: u32,
    /// Whether this GPU is currently selected for use
    pub is_selected: bool,
    /// Current utilization percentage (0-100)
    pub utilization: u32,
    /// Current temperature in Celsius
    pub temperature: u32,
}

impl GpuInfo {
    /// Get memory usage as a percentage
    pub fn memory_usage_percent(&self) -> f32 {
        if self.total_memory > 0 {
            ((self.total_memory - self.free_memory) as f32 / self.total_memory as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Check if this GPU is suitable for RTX AI processing
    pub fn is_rtx_capable(&self) -> bool {
        self.architecture.has_tensor_cores()
    }

    /// Get a score for this GPU (higher is better)
    pub fn performance_score(&self) -> u64 {
        let arch_multiplier = match self.architecture {
            GpuArchitecture::Legacy => 1,
            GpuArchitecture::Turing => 2,
            GpuArchitecture::Ampere => 3,
            GpuArchitecture::Ada => 4,
            GpuArchitecture::Blackwell => 5,
        };

        (self.sm_count as u64 * arch_multiplier) + (self.total_memory / (1024 * 1024 * 1024))
    }
}

/// CUDA driver API types
type CuResult = c_int;
type CuDevice = c_int;

/// Multi-GPU manager singleton
pub struct GpuManager {
    /// CUDA driver library
    cuda_lib: Option<Library>,
    /// List of available GPUs
    gpus: Vec<GpuInfo>,
    /// Currently selected GPU index
    selected_gpu: usize,
    /// Whether CUDA is available
    cuda_available: bool,
}

// Global singleton
static GPU_MANAGER: OnceLock<Mutex<GpuManager>> = OnceLock::new();

impl GpuManager {
    /// Get the global GPU manager instance
    pub fn global() -> &'static Mutex<GpuManager> {
        GPU_MANAGER.get_or_init(|| {
            let manager = GpuManager::new();
            Mutex::new(manager)
        })
    }

    /// Create a new GPU manager
    fn new() -> Self {
        let mut manager = Self {
            cuda_lib: None,
            gpus: Vec::new(),
            selected_gpu: 0,
            cuda_available: false,
        };

        // Try to initialize CUDA
        if let Err(e) = manager.initialize_cuda() {
            log::warn!("CUDA initialization failed: {}", e);
        }

        manager
    }

    /// Initialize CUDA and enumerate GPUs
    fn initialize_cuda(&mut self) -> Result<()> {
        // Try to load CUDA driver library
        let lib = unsafe {
            Library::new("libcuda.so.1")
                .or_else(|_| Library::new("libcuda.so"))
                .context("Failed to load CUDA driver library")?
        };

        // Get cuInit
        let cu_init: Symbol<unsafe extern "C" fn(u32) -> CuResult> =
            unsafe { lib.get(b"cuInit") }.context("Failed to get cuInit")?;

        // Initialize CUDA
        let result = unsafe { cu_init(0) };
        if result != 0 {
            anyhow::bail!("cuInit failed with error {}", result);
        }

        // Get device count
        let cu_device_get_count: Symbol<unsafe extern "C" fn(*mut c_int) -> CuResult> =
            unsafe { lib.get(b"cuDeviceGetCount") }.context("Failed to get cuDeviceGetCount")?;

        let mut device_count: c_int = 0;
        let result = unsafe { cu_device_get_count(&mut device_count) };
        if result != 0 {
            anyhow::bail!("cuDeviceGetCount failed with error {}", result);
        }

        log::info!("Found {} CUDA device(s)", device_count);

        // Enumerate devices
        self.gpus.clear();
        for i in 0..device_count as usize {
            if let Ok(gpu_info) = self.get_device_info(&lib, i) {
                self.gpus.push(gpu_info);
            }
        }

        // Select best GPU by default
        if !self.gpus.is_empty() {
            self.select_best_gpu();
        }

        self.cuda_lib = Some(lib);
        self.cuda_available = true;
        Ok(())
    }

    /// Get information about a specific device
    fn get_device_info(&self, lib: &Library, index: usize) -> Result<GpuInfo> {
        // Get device handle
        let cu_device_get: Symbol<unsafe extern "C" fn(*mut CuDevice, c_int) -> CuResult> =
            unsafe { lib.get(b"cuDeviceGet") }?;

        let mut device: CuDevice = 0;
        let result = unsafe { cu_device_get(&mut device, index as c_int) };
        if result != 0 {
            anyhow::bail!("cuDeviceGet failed: {}", result);
        }

        // Get device name
        let cu_device_get_name: Symbol<unsafe extern "C" fn(*mut u8, c_int, CuDevice) -> CuResult> =
            unsafe { lib.get(b"cuDeviceGetName") }?;

        let mut name_buf = [0u8; 256];
        let result = unsafe { cu_device_get_name(name_buf.as_mut_ptr(), 256, device) };
        let name = if result == 0 {
            let cstr = unsafe { CStr::from_ptr(name_buf.as_ptr() as *const i8) };
            cstr.to_string_lossy().to_string()
        } else {
            format!("GPU {}", index)
        };

        // Get compute capability
        let cu_device_get_attribute: Symbol<
            unsafe extern "C" fn(*mut c_int, c_int, CuDevice) -> CuResult,
        > = unsafe { lib.get(b"cuDeviceGetAttribute") }?;

        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        unsafe {
            cu_device_get_attribute(&mut major, 75, device); // CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR
            cu_device_get_attribute(&mut minor, 76, device); // CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR
        }

        // Get total memory
        let cu_device_total_mem: Symbol<unsafe extern "C" fn(*mut usize, CuDevice) -> CuResult> =
            unsafe { lib.get(b"cuDeviceTotalMem_v2") }
                .or_else(|_| unsafe { lib.get(b"cuDeviceTotalMem") })?;

        let mut total_mem: usize = 0;
        unsafe { cu_device_total_mem(&mut total_mem, device) };

        // Get SM count
        let mut sm_count: c_int = 0;
        unsafe {
            cu_device_get_attribute(&mut sm_count, 16, device); // CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT
        }

        // Get memory clock
        let mut mem_clock: c_int = 0;
        unsafe {
            cu_device_get_attribute(&mut mem_clock, 36, device); // CU_DEVICE_ATTRIBUTE_MEMORY_CLOCK_RATE (kHz)
        }

        let architecture = GpuArchitecture::from_compute_capability(major, minor);

        Ok(GpuInfo {
            index,
            name,
            compute_major: major,
            compute_minor: minor,
            architecture,
            total_memory: total_mem as u64,
            free_memory: total_mem as u64, // Will be updated later
            memory_clock_mhz: (mem_clock / 1000) as u32,
            sm_count: sm_count as u32,
            is_selected: false,
            utilization: 0,
            temperature: 0,
        })
    }

    /// Refresh GPU information (memory, utilization, temperature)
    pub fn refresh(&mut self) -> Result<()> {
        // Use nvidia-smi for runtime stats
        let output = std::process::Command::new("nvidia-smi")
            .args([
                "--query-gpu=index,memory.free,memory.used,utilization.gpu,temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 5 {
                    if let Ok(idx) = parts[0].parse::<usize>() {
                        if let Some(gpu) = self.gpus.get_mut(idx) {
                            gpu.free_memory = parts[1]
                                .parse::<u64>()
                                .unwrap_or(0)
                                * 1024
                                * 1024;
                            gpu.utilization = parts[3].parse().unwrap_or(0);
                            gpu.temperature = parts[4].parse().unwrap_or(0);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all available GPUs
    pub fn get_gpus(&self) -> &[GpuInfo] {
        &self.gpus
    }

    /// Get the currently selected GPU
    pub fn get_selected_gpu(&self) -> Option<&GpuInfo> {
        self.gpus.get(self.selected_gpu)
    }

    /// Select a specific GPU by index
    pub fn select_gpu(&mut self, index: usize) -> Result<()> {
        if index >= self.gpus.len() {
            anyhow::bail!("Invalid GPU index: {}", index);
        }

        // Update selection state
        for (i, gpu) in self.gpus.iter_mut().enumerate() {
            gpu.is_selected = i == index;
        }

        self.selected_gpu = index;
        log::info!(
            "Selected GPU {}: {}",
            index,
            self.gpus[index].name
        );
        Ok(())
    }

    /// Select the best GPU based on performance score
    pub fn select_best_gpu(&mut self) {
        if self.gpus.is_empty() {
            return;
        }

        let best_idx = self
            .gpus
            .iter()
            .enumerate()
            .max_by_key(|(_, gpu)| gpu.performance_score())
            .map(|(i, _)| i)
            .unwrap_or(0);

        let _ = self.select_gpu(best_idx);
    }

    /// Check if CUDA is available
    pub fn is_cuda_available(&self) -> bool {
        self.cuda_available
    }

    /// Get number of GPUs
    pub fn gpu_count(&self) -> usize {
        self.gpus.len()
    }

    /// Check if multi-GPU is available
    pub fn has_multi_gpu(&self) -> bool {
        self.gpus.len() > 1
    }

    /// Get GPU names as comma-separated string
    pub fn gpu_names(&self) -> String {
        self.gpus
            .iter()
            .map(|g| g.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Find the best GPU for RTX processing
    pub fn find_best_rtx_gpu(&self) -> Option<&GpuInfo> {
        self.gpus
            .iter()
            .filter(|gpu| gpu.is_rtx_capable())
            .max_by_key(|gpu| gpu.performance_score())
    }

    /// Get total GPU memory across all GPUs
    pub fn total_memory(&self) -> u64 {
        self.gpus.iter().map(|g| g.total_memory).sum()
    }

    /// Get total free memory across all GPUs
    pub fn total_free_memory(&self) -> u64 {
        self.gpus.iter().map(|g| g.free_memory).sum()
    }
}

/// GPU load balancer for distributing work across multiple GPUs
pub struct GpuLoadBalancer {
    /// GPU indices in order of preference
    gpu_order: Vec<usize>,
    /// Current index in the round-robin order
    current_idx: usize,
    /// Load balancing strategy
    strategy: LoadBalancingStrategy,
}

/// Load balancing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadBalancingStrategy {
    /// Use only the primary (best) GPU
    Primary,
    /// Round-robin across all GPUs
    RoundRobin,
    /// Use GPU with most free memory
    LeastMemory,
    /// Use GPU with lowest utilization
    LeastUtilization,
}

impl GpuLoadBalancer {
    /// Create a new load balancer
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        let manager = GpuManager::global().lock().unwrap();
        let mut gpu_order: Vec<usize> = (0..manager.gpu_count()).collect();

        // Sort by performance score (best first)
        let gpus = manager.get_gpus();
        gpu_order.sort_by_key(|&i| std::cmp::Reverse(gpus[i].performance_score()));

        Self {
            gpu_order,
            current_idx: 0,
            strategy,
        }
    }

    /// Get the next GPU to use based on the strategy
    pub fn next_gpu(&mut self) -> Option<usize> {
        if self.gpu_order.is_empty() {
            return None;
        }

        let manager = GpuManager::global().lock().unwrap();
        let gpus = manager.get_gpus();

        match self.strategy {
            LoadBalancingStrategy::Primary => Some(self.gpu_order[0]),

            LoadBalancingStrategy::RoundRobin => {
                let gpu = self.gpu_order[self.current_idx];
                self.current_idx = (self.current_idx + 1) % self.gpu_order.len();
                Some(gpu)
            }

            LoadBalancingStrategy::LeastMemory => self
                .gpu_order
                .iter()
                .max_by_key(|&&i| gpus[i].free_memory)
                .copied(),

            LoadBalancingStrategy::LeastUtilization => self
                .gpu_order
                .iter()
                .min_by_key(|&&i| gpus[i].utilization)
                .copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_detection() {
        assert_eq!(
            GpuArchitecture::from_compute_capability(12, 0),
            GpuArchitecture::Blackwell
        );
        assert_eq!(
            GpuArchitecture::from_compute_capability(8, 9),
            GpuArchitecture::Ada
        );
        assert_eq!(
            GpuArchitecture::from_compute_capability(8, 0),
            GpuArchitecture::Ampere
        );
        assert_eq!(
            GpuArchitecture::from_compute_capability(7, 5),
            GpuArchitecture::Turing
        );
    }

    #[test]
    fn test_tensor_core_detection() {
        assert!(GpuArchitecture::Blackwell.has_tensor_cores());
        assert!(GpuArchitecture::Ada.has_tensor_cores());
        assert!(!GpuArchitecture::Legacy.has_tensor_cores());
    }

    #[test]
    fn test_fp4_fp8_detection() {
        assert!(GpuArchitecture::Blackwell.has_fp4());
        assert!(!GpuArchitecture::Ada.has_fp4());

        assert!(GpuArchitecture::Blackwell.has_fp8());
        assert!(GpuArchitecture::Ada.has_fp8());
        assert!(!GpuArchitecture::Ampere.has_fp8());
    }
}
