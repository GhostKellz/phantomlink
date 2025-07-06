# Advanced Noise Suppression for PhantomLink

## Overview

PhantomLink currently uses RNNoise via the `nnnoiseless` crate for basic noise suppression. To provide RTX Voice-like functionality on Linux, we'll implement a multi-tier noise suppression system with neural network-based enhancement.

## Current Implementation

- **Basic Denoiser**: RNNoise via `nnnoiseless` (already implemented)
- **Processing**: Real-time audio frames (480 samples at 48kHz)
- **Integration**: Embedded in audio processing pipeline

## Advanced Denoising Architecture

### Tier 1: Enhanced RNNoise
- Current implementation using `nnnoiseless`
- Good for basic background noise (fans, air conditioning)
- Low latency (~10ms)
- Low CPU usage

### Tier 2: Deep Learning Denoiser
Multiple options for neural network-based enhancement:

#### Option A: Candle + ONNX Runtime
- Use `candle-core` and `candle-onnx` for efficient inference
- Load pre-trained ONNX models (Facebook Denoiser, DNS Challenge models)
- GPU acceleration support (CUDA/ROCm)
- Medium latency (~30-50ms)

#### Option B: TensorFlow Lite
- Use `tflitec` crate for TensorFlow Lite inference
- Smaller model size, optimized for real-time
- CPU optimized
- Low-medium latency (~20-40ms)

#### Option C: Native PyTorch Models
- Use Facebook Denoiser implementation
- Highest quality but higher latency
- Requires `tch` crate (PyTorch bindings)

### Tier 3: Spectral Enhancement
- Advanced spectral masking
- Wiener filtering
- Adaptive noise estimation
- Custom implementation using `realfft`

## Implementation Plan

### Phase 1: Enhanced Architecture
1. **Multi-tier Denoiser Interface**
   ```rust
   pub trait AdvancedDenoiser {
       fn process_frame(&mut self, input: &[f32]) -> Vec<f32>;
       fn set_mode(&mut self, mode: DenoisingMode);
       fn get_latency(&self) -> f32; // in milliseconds
       fn get_cpu_usage(&self) -> f32; // percentage
   }
   
   pub enum DenoisingMode {
       Basic,     // RNNoise only
       Enhanced,  // RNNoise + Deep Learning
       Maximum,   // All tiers enabled
       Custom { tiers: Vec<DenoiserTier> },
   }
   ```

2. **Model Loading System**
   - Download and cache pre-trained models
   - Support for multiple model formats (ONNX, TensorFlow Lite)
   - Automatic model selection based on CPU/GPU capabilities

3. **Adaptive Processing**
   - Real-time quality assessment
   - Automatic tier switching based on performance
   - User-configurable quality vs. latency trade-offs

### Phase 2: Deep Learning Integration

#### Facebook Denoiser (ONNX)
- Download pre-trained models from Facebook Research
- Implement `candle-onnx` inference pipeline
- Optimize for real-time processing

#### DNS Challenge Models
- Use Microsoft DNS Challenge winning models
- Implement multi-model ensemble for best quality
- Support for different model sizes (small/medium/large)

#### Custom Training Pipeline (Future)
- Implement training infrastructure using `candle-core`
- Support for custom dataset training
- Transfer learning from existing models

### Phase 3: GPU Acceleration
- CUDA support via `candle-cuda-backend`
- ROCm support for AMD GPUs
- Automatic GPU detection and fallback

### Phase 4: GUI Integration
- Real-time denoising mode selection
- Visual feedback for processing load
- A/B testing interface for quality comparison
- Real-time spectrograms showing before/after

## Technical Specifications

### Model Requirements
- **Input**: 48kHz mono audio, 480-sample frames (10ms)
- **Output**: Denoised audio at same sample rate
- **Latency Target**: <50ms total (including buffering)
- **CPU Target**: <25% single core usage
- **Memory**: <500MB total model storage

### Dependencies to Add
```toml
# Neural Network Inference
candle-core = "0.9"
candle-onnx = "0.9"
tflitec = "0.6"

# Model Management
reqwest = { version = "0.11", features = ["blocking"] }
tokio = { version = "1.0", features = ["full"] }
dirs = "5.0"

# Performance Monitoring
sysinfo = "0.30"
```

### Model Sources
1. **Facebook Denoiser**: https://github.com/facebookresearch/denoiser
2. **Microsoft DNS Challenge**: https://github.com/microsoft/DNS-Challenge
3. **ONNX Model Zoo**: https://github.com/onnx/models
4. **Hugging Face Audio Models**: https://huggingface.co/models?pipeline_tag=audio-classification

## Benefits Over RTX Voice

1. **Open Source**: Fully auditable and customizable
2. **Multi-GPU Support**: Not limited to NVIDIA hardware
3. **Modular Design**: Users can choose quality vs. performance
4. **Real-time Adaptation**: Automatic quality adjustment
5. **Custom Training**: Support for domain-specific models
6. **Cross-Platform**: Works on any Linux distribution

## Performance Expectations

### Basic Mode (RNNoise only)
- Latency: ~10ms
- CPU: ~5-10%
- Quality: Good for steady background noise

### Enhanced Mode (RNNoise + Deep Learning)
- Latency: ~30-50ms
- CPU: ~15-25%
- Quality: Excellent for complex noise scenarios

### Maximum Mode (All tiers)
- Latency: ~50-80ms
- CPU: ~25-40%
- Quality: Best possible, comparable to RTX Voice

## Quality Metrics

We'll implement objective quality measurements:
- **PESQ**: Perceptual Evaluation of Speech Quality
- **STOI**: Short-Time Objective Intelligibility
- **SI-SNR**: Scale-Invariant Signal-to-Noise Ratio
- **Real-time Factor**: Processing speed vs. real-time

## Future Enhancements

1. **Voice Activity Detection (VAD)**: Advanced VAD using neural networks
2. **Speaker Adaptation**: Personalized models for individual users
3. **Multi-channel Processing**: Support for stereo and multi-mic setups
4. **Echo Cancellation**: Integration with acoustic echo cancellation
5. **Speech Enhancement**: Beyond noise removal - clarity improvement
6. **Real-time Training**: Continuous model improvement during use

This architecture will provide PhantomLink with state-of-the-art noise suppression capabilities that match or exceed proprietary solutions like RTX Voice, while maintaining the open-source, customizable nature of the project.
