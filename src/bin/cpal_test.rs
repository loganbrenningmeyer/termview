use cpal::traits::{
    DeviceTrait,
    HostTrait,
    StreamTrait,
};

use std::f32::consts::TAU;

fn main() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("No output device");

    let supported_config = device
        .default_output_config()
        .expect("No default output config");

    println!("{:?}", supported_config);

    let sample_rate = supported_config.sample_rate() as f32;
    let channels = supported_config.channels() as usize;

    let config: cpal::StreamConfig = supported_config.into();

    let frequency = 440.0;
    let mut phase = 0.0f32;

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [f32], _| {
                for frame in data.chunks_mut(channels) {
                    let value = (phase * TAU).sin() * 0.2;

                    phase += frequency / sample_rate;
                    phase = phase.rem_euclid(1.0);

                    for sample in frame {
                        *sample = value;
                    }
                }
            },
            move |err| {
                eprintln!("Audio error: {err}");
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();

    loop {
        std::thread::park();
    }
}