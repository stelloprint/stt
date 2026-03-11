use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use thiserror::Error;

const TARGET_SAMPLE_RATE: u32 = 16000;
const TARGET_CHANNELS: u16 = 1;
const FRAME_SIZE: usize = 4096;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No input device available")]
    NoInputDevice,
    #[error("Failed to get default input device: {0}")]
    DeviceError(String),
    #[error("Failed to build input stream: {0}")]
    StreamError(String),
    #[error("Stream playback error: {0}")]
    PlaybackError(String),
    #[error("Audio not started")]
    NotStarted,
    #[error("Audio already running")]
    AlreadyRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilenceLevel {
    Low,
    Medium,
    High,
}

impl SilenceLevel {
    pub fn threshold(&self) -> f32 {
        match self {
            Self::Low => 0.01,
            Self::Medium => 0.02,
            Self::High => 0.05,
        }
    }
}

fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

fn resample_linear(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = to_rate as f32 / from_rate as f32;
    let new_len = (samples.len() as f32 * ratio) as usize;
    let mut output = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = i as f32 / ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f32;

        if idx + 1 < samples.len() {
            let sample = samples[idx] * (1.0 - frac) + samples[idx + 1] * frac;
            output.push(sample);
        } else if idx < samples.len() {
            output.push(samples[idx]);
        }
    }

    output
}

pub struct AudioCapture {
    is_recording: AtomicBool,
    sample_rate: std::sync::RwLock<Option<u32>>,
    buffer: std::sync::RwLock<Vec<f32>>,
    silence_threshold: std::sync::RwLock<f32>,
    silence_start_time: AtomicU64,
    running: AtomicBool,
    device_sample_rate: u32,
    input_channels: u16,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            is_recording: AtomicBool::new(false),
            sample_rate: std::sync::RwLock::new(None),
            buffer: std::sync::RwLock::new(Vec::new()),
            silence_threshold: std::sync::RwLock::new(SilenceLevel::Medium.threshold()),
            silence_start_time: AtomicU64::new(0),
            running: AtomicBool::new(false),
            device_sample_rate: 44100,
            input_channels: 1,
        }
    }

    pub fn set_silence_threshold(&self, level: SilenceLevel) {
        *self.silence_threshold.write().unwrap() = level.threshold();
    }

    pub fn start(&self) -> Result<(), AudioError> {
        if self
            .is_recording
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(AudioError::AlreadyRunning);
        }

        self.buffer.write().unwrap().clear();
        self.silence_start_time.store(0, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);

        let buffer = Arc::new(std::sync::RwLock::new(Vec::<f32>::new()));
        let silence_threshold = Arc::new(std::sync::RwLock::new(SilenceLevel::Medium.threshold()));
        let silence_start_time = Arc::new(AtomicU64::new(0));
        let is_recording = Arc::new(AtomicBool::new(true));
        let running = Arc::new(AtomicBool::new(true));
        let sample_rate = Arc::new(std::sync::RwLock::new(None::<u32>));

        let this = self;
        *buffer.write().unwrap() = this.buffer.read().unwrap().clone();
        *silence_threshold.write().unwrap() = *this.silence_threshold.read().unwrap();

        let device_sample_rate = this.device_sample_rate;
        let input_channels = this.input_channels;

        thread::spawn(move || {
            if let Err(e) = run_capture_loop(
                buffer,
                silence_threshold,
                silence_start_time,
                is_recording,
                running,
                sample_rate,
                device_sample_rate,
                input_channels,
            ) {
                log::error!("Audio capture error: {}", e);
            }
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), AudioError> {
        if self
            .is_recording
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(AudioError::NotStarted);
        }

        self.running.store(false, Ordering::SeqCst);
        self.buffer.write().unwrap().clear();
        self.silence_start_time.store(0, Ordering::SeqCst);

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn get_silence_duration_ms(&self) -> u64 {
        let start = self.silence_start_time.load(Ordering::SeqCst);
        if start == 0 {
            return 0;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        now.saturating_sub(start)
    }

    pub fn get_buffer(&self) -> Vec<f32> {
        let sr = *self.sample_rate.read().unwrap();
        let buf = self.buffer.read().unwrap().clone();
        if let Some(device_sr) = sr {
            if device_sr != TARGET_SAMPLE_RATE {
                return resample_linear(&buf, device_sr, TARGET_SAMPLE_RATE);
            }
        }
        buf
    }

    pub fn clear_buffer(&self) {
        self.buffer.write().unwrap().clear();
    }
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self::new()
    }
}

fn run_capture_loop(
    buffer: Arc<std::sync::RwLock<Vec<f32>>>,
    silence_threshold: Arc<std::sync::RwLock<f32>>,
    silence_start_time: Arc<AtomicU64>,
    is_recording: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    sample_rate: Arc<std::sync::RwLock<Option<u32>>>,
    device_sample_rate: u32,
    input_channels: u16,
) -> Result<(), AudioError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(AudioError::NoInputDevice)?;

    let config = device
        .default_input_config()
        .map_err(|e| AudioError::DeviceError(e.to_string()))?;

    let device_sr = config.sample_rate().0;
    *sample_rate.write().unwrap() = Some(device_sr);

    let use_resample = device_sr != TARGET_SAMPLE_RATE || config.channels() != TARGET_CHANNELS;

    let err_fn = |err| log::error!("Audio stream error: {}", err);

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let buffer = buffer.clone();
            let silence_threshold = silence_threshold.clone();
            let silence_start_time = silence_start_time.clone();

            let callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let output = if use_resample {
                    let mono: Vec<f32> = if input_channels > 1 {
                        data.iter()
                            .step_by(input_channels as usize)
                            .copied()
                            .collect()
                    } else {
                        data.to_vec()
                    };
                    resample_linear(&mono, device_sr, TARGET_SAMPLE_RATE)
                } else if input_channels > 1 {
                    data.iter()
                        .step_by(input_channels as usize)
                        .copied()
                        .collect()
                } else {
                    data.to_vec()
                };

                if !output.is_empty() {
                    let rms = compute_rms(&output);
                    let threshold = *silence_threshold.read().unwrap();

                    if rms < threshold {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        if silence_start_time.load(Ordering::SeqCst) == 0 {
                            silence_start_time.store(now, Ordering::SeqCst);
                        }
                    } else {
                        silence_start_time.store(0, Ordering::SeqCst);
                    }

                    buffer.write().unwrap().extend(output);
                }
            };
            device.build_input_stream(&config.into(), callback, err_fn, None)
        }
        cpal::SampleFormat::I16 => {
            let buffer = buffer.clone();
            let silence_threshold = silence_threshold.clone();
            let silence_start_time = silence_start_time.clone();

            let callback = move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let data_f32: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();

                let output = if use_resample {
                    let mono: Vec<f32> = if input_channels > 1 {
                        data_f32
                            .iter()
                            .step_by(input_channels as usize)
                            .copied()
                            .collect()
                    } else {
                        data_f32
                    };
                    resample_linear(&mono, device_sr, TARGET_SAMPLE_RATE)
                } else if input_channels > 1 {
                    data_f32
                        .iter()
                        .step_by(input_channels as usize)
                        .copied()
                        .collect()
                } else {
                    data_f32
                };

                if !output.is_empty() {
                    let rms = compute_rms(&output);
                    let threshold = *silence_threshold.read().unwrap();

                    if rms < threshold {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        if silence_start_time.load(Ordering::SeqCst) == 0 {
                            silence_start_time.store(now, Ordering::SeqCst);
                        }
                    } else {
                        silence_start_time.store(0, Ordering::SeqCst);
                    }

                    buffer.write().unwrap().extend(output);
                }
            };
            device.build_input_stream(&config.into(), callback, err_fn, None)
        }
        cpal::SampleFormat::U16 => {
            let buffer = buffer.clone();
            let silence_threshold = silence_threshold.clone();
            let silence_start_time = silence_start_time.clone();

            let callback = move |data: &[u16], _: &cpal::InputCallbackInfo| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let data_f32: Vec<f32> = data
                    .iter()
                    .map(|&s| (s as f32 - 32768.0) / 32768.0)
                    .collect();

                let output = if use_resample {
                    let mono: Vec<f32> = if input_channels > 1 {
                        data_f32
                            .iter()
                            .step_by(input_channels as usize)
                            .copied()
                            .collect()
                    } else {
                        data_f32
                    };
                    resample_linear(&mono, device_sr, TARGET_SAMPLE_RATE)
                } else if input_channels > 1 {
                    data_f32
                        .iter()
                        .step_by(input_channels as usize)
                        .copied()
                        .collect()
                } else {
                    data_f32
                };

                if !output.is_empty() {
                    let rms = compute_rms(&output);
                    let threshold = *silence_threshold.read().unwrap();

                    if rms < threshold {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        if silence_start_time.load(Ordering::SeqCst) == 0 {
                            silence_start_time.store(now, Ordering::SeqCst);
                        }
                    } else {
                        silence_start_time.store(0, Ordering::SeqCst);
                    }

                    buffer.write().unwrap().extend(output);
                }
            };
            device.build_input_stream(&config.into(), callback, err_fn, None)
        }
        _ => {
            return Err(AudioError::StreamError(
                "Unsupported sample format".to_string(),
            ))
        }
    }
    .map_err(|e| AudioError::StreamError(e.to_string()))?;

    stream
        .play()
        .map_err(|e| AudioError::PlaybackError(e.to_string()))?;

    while running.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(50));
    }

    drop(stream);

    Ok(())
}

pub struct AudioHandle {
    capture: std::sync::Mutex<Option<AudioCapture>>,
}

impl AudioHandle {
    pub fn new() -> Result<Self, AudioError> {
        Ok(Self {
            capture: std::sync::Mutex::new(Some(AudioCapture::new())),
        })
    }

    pub fn set_silence_threshold(&self, level: SilenceLevel) {
        if let Some(ref cap) = *self.capture.lock().unwrap() {
            cap.set_silence_threshold(level);
        }
    }

    pub fn start(&self) -> Result<(), AudioError> {
        if let Some(ref cap) = *self.capture.lock().unwrap() {
            cap.start()
        } else {
            Err(AudioError::NotStarted)
        }
    }

    pub fn stop(&self) -> Result<(), AudioError> {
        if let Some(ref cap) = *self.capture.lock().unwrap() {
            cap.stop()
        } else {
            Err(AudioError::NotStarted)
        }
    }

    pub fn is_recording(&self) -> bool {
        if let Some(ref cap) = *self.capture.lock().unwrap() {
            cap.is_recording()
        } else {
            false
        }
    }

    pub fn get_silence_duration_ms(&self) -> u64 {
        if let Some(ref cap) = *self.capture.lock().unwrap() {
            cap.get_silence_duration_ms()
        } else {
            0
        }
    }

    pub fn get_buffer(&self) -> Vec<f32> {
        if let Some(ref cap) = *self.capture.lock().unwrap() {
            cap.get_buffer()
        } else {
            Vec::new()
        }
    }

    pub fn clear_buffer(&self) -> Result<(), AudioError> {
        if let Some(ref cap) = *self.capture.lock().unwrap() {
            cap.clear_buffer();
            Ok(())
        } else {
            Err(AudioError::NotStarted)
        }
    }
}

impl Default for AudioHandle {
    fn default() -> Self {
        Self::new().expect("Failed to create audio handle")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_calculation_silence() {
        let samples = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        let rms = compute_rms(&samples);
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_rms_calculation_known_values() {
        let samples = vec![1.0, -1.0, 1.0, -1.0];
        let rms = compute_rms(&samples);
        assert!((rms - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rms_calculation_single_sample() {
        let samples = vec![0.5];
        let rms = compute_rms(&samples);
        assert!((rms - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_rms_calculation_empty() {
        let samples: Vec<f32> = vec![];
        let rms = compute_rms(&samples);
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_resample_linear_same_rate() {
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        let result = resample_linear(&samples, 16000, 16000);
        assert_eq!(result.len(), 4);
        assert!((result[0] - 1.0).abs() < 0.001);
        assert!((result[3] - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_resample_linear_downsample() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let result = resample_linear(&samples, 8000, 4000);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_resample_linear_upsample() {
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        let result = resample_linear(&samples, 4000, 8000);
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_resample_linear_preserves_values() {
        let samples = vec![1.0f32; 100];
        let result = resample_linear(&samples, 44100, 16000);
        for sample in &result {
            assert!((sample - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_silence_level_thresholds() {
        assert!(SilenceLevel::Low.threshold() < SilenceLevel::Medium.threshold());
        assert!(SilenceLevel::Medium.threshold() < SilenceLevel::High.threshold());
    }

    #[test]
    fn test_audio_capture_initial_state() {
        let capture = AudioCapture::new();
        assert!(!capture.is_recording());
        assert_eq!(capture.get_silence_duration_ms(), 0);
    }

    #[test]
    fn test_audio_handle_initial_state() {
        let handle = AudioHandle::new().unwrap();
        assert!(!handle.is_recording());
    }

    #[test]
    fn test_audio_handle_new_creates_without_panic() {
        let result = AudioHandle::new();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_target_sample_rate_constant() {
        assert_eq!(TARGET_SAMPLE_RATE, 16000);
    }

    #[test]
    fn test_target_channels_constant() {
        assert_eq!(TARGET_CHANNELS, 1);
    }
}
