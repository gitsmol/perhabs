use std::sync::mpsc::Receiver;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, Stream};
// wasm stuff only
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::console;
//
// Putting all the audio stuff here for now
//

pub struct AudioHandle(Stream);

impl AudioHandle {
    pub fn play(&self) {
        self.0.play();
        debug!("Audiohandle: Playing stream.");
    }
    pub fn pause(&self) {
        self.0.pause();
        debug!("Audiohandle: Pausing stream.");
    }
}

pub struct AudioContext {
    clock: f32,
    channels: usize,
    samplerate: f32,
    voices: Vec<Voice>,
}

impl AudioContext {
    fn add_voice(&mut self, voice: Voice) {
        self.voices.push(voice);
    }

    fn replace_voice(&mut self, voice: Voice) {
        self.voices.clear();
        self.voices.push(voice);
    }

    fn remove_voice(&mut self) {
        self.voices.pop();
    }
}

#[derive(Clone, Copy)]
pub struct Voice {
    playing: bool,
    current_frame: u64,
    pub attack: u64,
    pub sustain: u64,
    pub release: u64,
    pub freq: f32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            playing: true,
            attack: 5000,
            sustain: 5000,
            release: 5000,
            current_frame: 0,
            freq: 440.0,
        }
    }
}

impl Voice {
    fn duration(&self) -> u64 {
        self.attack + self.sustain + self.release
    }
}

pub fn beep(rx: Receiver<Voice>) -> AudioHandle {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device
        .default_output_config()
        .expect("Can't create config.");

    let audio_ctx = AudioContext {
        voices: vec![],
        channels: config.channels().into(),
        samplerate: config.sample_rate().0 as f32,
        clock: 0.,
    };

    debug!("device: {:?}", device.name().unwrap_or_default());
    debug!("config: {:?}", config.config());
    AudioHandle(match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), audio_ctx, rx),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), audio_ctx, rx),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), audio_ctx, rx),
    })
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut audio_ctx: AudioContext,
    rx: Receiver<Voice>,
) -> Stream
where
    T: Sample,
{
    #[cfg(target_arch = "wasm32")]
    let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());
    #[cfg(not(target_arch = "wasm32"))]
    let err_fn = |err| log::error!("an error occurred on stream: {}", err);

    let audio_fn = move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
        for frame in output.chunks_mut(audio_ctx.channels) {
            let mut delete_voice = vec![];
            for (i, voice) in audio_ctx.voices.iter_mut().enumerate() {
                if !voice.playing {
                    delete_voice.push(i);
                }
            }
            for i in delete_voice {
                audio_ctx.voices.swap_remove(i);
            }
            let v = Sample::from::<f32>(&next_value(&mut audio_ctx, &rx));
            for value in frame.iter_mut() {
                *value = v;
            }
        }
    };

    let stream = device
        .build_output_stream(config, audio_fn, err_fn)
        .unwrap();
    match stream.play() {
        Ok(_) => debug!("I'm playing."),
        Err(_) => error!("I can't play stream."),
    };
    stream
}

fn next_value(audio_ctx: &mut AudioContext, rx: &Receiver<Voice>) -> f32 {
    // Produce a sinusoid of maximum amplitude.
    audio_ctx.clock = (audio_ctx.clock + 1.0) % audio_ctx.samplerate;
    if let Ok(voice) = rx.try_recv() {
        debug!("Received voice.");
        audio_ctx.add_voice(voice);
    }

    // Mix all voices
    let mut value = 0.;
    for voice in audio_ctx.voices.iter_mut() {
        voice.current_frame += 1;
        if voice.current_frame >= voice.duration() {
            voice.playing = false;
            continue;
        }

        // Determine the current ADSR-stage by working backwards from the end.
        let amp = match voice.current_frame {
            // Release is after attack + sustain
            frame if frame >= (voice.attack + voice.sustain) => {
                (voice.duration() as f32 - frame as f32) / voice.release as f32
            }

            // Sustain is after attack
            frame if frame >= voice.attack => 1.,

            // Attack is first
            frame => frame as f32 / voice.attack as f32,
        };
        value += (audio_ctx.clock * voice.freq * 3.141592 / audio_ctx.samplerate).sin() * amp;
    }
    value
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
