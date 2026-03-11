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

        let buffer = Arc::new(self.buffer.clone());
        let silence_threshold = Arc::new(self.silence_threshold.clone());
        let silence_start_time = &self.silence_start_time;
        let is_recording = &self.is_recording;
        let running = &self.running;
        let sample_rate = &self.sample_rate;
        let device_sample_rate = self.device_sample_rate;
        let input_channels = self.input_channels;

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
        self.buffer.read().unwrap().clone()
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
    silence_start_time: &AtomicU64,
    is_recording: &AtomicBool,
    running: &AtomicBool,
    sample_rate: &std::sync::RwLock<Option<u32>>,
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

    *sample_rate.write().unwrap() = Some(config.sample_rate().0);

    let use_resample =
        config.sample_rate().0 != TARGET_SAMPLE_RATE || config.channels() != TARGET_CHANNELS;

    let resampler = if use_resample {
        Some(rubato::FftFixedIn::<f32>::new(
            config.sample_rate().0 as usize,
            TARGET_SAMPLE_RATE as usize,
            FRAME_SIZE,
            config.channels() as usize,
            TARGET_CHANNELS as usize,
        ))
    } else {
        None
    };

    let err_fn = |err| log::error!("Audio stream error: {}", err);

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let buffer = buffer.clone();
            let silence_threshold = silence_threshold.clone();
            let silence_start_time = silence_start_time;

            let callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let output = if let Some(ref rs) = resampler {
                    let mut out = Vec::new();
                    let _ = rs.process(data, &mut out);
                    out
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
            let silence_start_time = silence_start_time;

            let callback = move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let data_f32: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();

                let output = if let Some(ref rs) = resampler {
                    let mut out = Vec::new();
                    let _ = rs.process(&data_f32, &mut out);
                    out
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
            let silence_start_time = silence_start_time;

            let callback = move |data: &[u16], _: &cpal::InputCallbackInfo| {
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                let data_f32: Vec<f32> = data
                    .iter()
                    .map(|&s| (s as f32 - 32768.0) / 32768.0)
                    .collect();

                let output = if let Some(ref rs) = resampler {
                    let mut out = Vec::new();
                    let _ = rs.process(&data_f32, &mut out);
                    out
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
    capture: Arc<AudioCapture>,
}

impl AudioHandle {
    pub fn new() -> Result<Self, AudioError> {
        Ok(Self {
            capture: Arc::new(AudioCapture::new()),
        })
    }

    pub fn set_silence_threshold(&self, level: SilenceLevel) {
        self.capture.set_silence_threshold(level);
    }

    pub fn start(&self) -> Result<(), AudioError> {
        self.capture.start()
    }

    pub fn stop(&self) -> Result<(), AudioError> {
        self.capture.stop()
    }

    pub fn is_recording(&self) -> bool {
        self.capture.is_recording()
    }

    pub fn get_silence_duration_ms(&self) -> u64 {
        self.capture.get_silence_duration_ms()
    }

    pub fn get_buffer(&self) -> Vec<f32> {
        self.capture.get_buffer()
    }

    pub fn clear_buffer(&self) {
        self.capture.clear_buffer();
    }

    pub fn get_capture(&self) -> Arc<AudioCapture> {
        Arc::clone(&self.capture)
    }
}

impl Default for AudioHandle {
    fn default() -> Self {
        Self::new().expect("Failed to create audio handle")
    }
}
