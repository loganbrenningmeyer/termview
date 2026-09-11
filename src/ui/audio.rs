use std::sync::{
    Arc, atomic::{AtomicBool, AtomicU32, Ordering},
};
use cpal::traits::{
    DeviceTrait,
    HostTrait,
    StreamTrait,
};


pub struct AudioEngine {
    pub sample_rate: f32,
    pub channels: usize,
    pub stream: cpal::Stream,
    frequency: Arc<AtomicU32>,
    published_phase: Arc<AtomicU32>,
    playing: Arc<AtomicBool>,
}

impl AudioEngine {
    /**
     * AudioEngine is given a sample_callback which returns an amplitude
     * value given a phase value passed from the AudioEngine, as well
     * as a copy of a frequency atomic to allow changing frequency
     * outside of the AudioEngine
     */
    pub fn new<F>(
        amp_callback: F, 
        freq_atomic: Arc<AtomicU32>, 
        playing_atomic: Arc<AtomicBool>,
    ) -> Result<Self, String> 
    where 
        F: Fn(f32) -> f32 + Send + 'static,
    {
        // -------------------------
        // Setup CPAL audio stream
        // -------------------------
        let host = cpal::default_host();
        
        let device = host
            .default_output_device()
            .expect("No output device");

        let supported_config = device
            .default_output_config()
            .expect("No default output config");

        // -------------------------
        // Get device-supported sample rate / channel count
        // -------------------------
        let sample_rate = supported_config.sample_rate() as f32;
        let channels = supported_config.channels() as usize;

        let config: cpal::StreamConfig = supported_config.into();

        // Initialize phase to 0
        let mut phase = 0.0f32;

        // Set frequency, phase, and playing shared Atomics
        let frequency = Arc::clone(&freq_atomic);

        let published_phase = Arc::new(AtomicU32::new(
            0.0_f32.to_bits(),
        ));
        let callback_phase = Arc::clone(&published_phase);

        let playing = Arc::clone(&playing_atomic);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _| {
                    for frame in data.chunks_mut(channels) {
                        // Load frequency atomic
                        let frequency = f32::from_bits(
                            freq_atomic.load(Ordering::Relaxed)
                        );

                        // If playing atomic false, shut sound off and return
                        if !playing_atomic.load(Ordering::Relaxed) {
                            data.fill(0.0);
                            return;
                        }

                        // Callback to WaveformController for amplitude given phase
                        let amplitude = amp_callback(phase);

                        frame.fill(amplitude);

                        // Step phase forward
                        phase += frequency / sample_rate;
                        phase = phase.rem_euclid(1.0);
                    }

                    // Publish the shared atomic phase
                    callback_phase.store(phase.to_bits(), Ordering::Relaxed);
                },
                move |err| {
                    eprintln!("Audio error: {err}");
                },
                None,
            )
            .unwrap();

        stream.play().expect("Failed to start audio stream");

        Ok(Self {
            sample_rate,
            channels,
            stream,
            frequency,
            published_phase,
            playing,
        })
    }

    pub fn phase(&self) -> f32 {
        f32::from_bits(
            self.published_phase.load(Ordering::Relaxed),
        )
    }

    pub fn set_frequency(&self, frequency: f32) {
        self.frequency.store(
            frequency.to_bits(),
            Ordering::Relaxed,
        );
    }

    pub fn set_playing(&self, playing: bool) {
        self.playing.store(
            playing,
            Ordering::Relaxed,
        );
    }
}