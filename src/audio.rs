use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Sample};
use cpal::{StreamError, Stream};
use cpal::Data;
use crossbeam_channel::{unbounded, Sender, Receiver, TryRecvError};

pub struct Audio {
    timer: f32,
    current_song: Option<Song>,
    audio_sender: Sender<AudioCommand>,
    audio_error_receiver: Receiver<StreamError>,
    stream: Stream,
}

#[derive(Copy, Clone)]
pub enum Song {
    Church,
    Boss
}

#[derive(Copy, Clone)]
enum AudioCommand {
    Play(Song),
    Stop,
}

const CHURCH_TIME: f32 = 88.0;
const BOSS_TIME: f32 = 117.0;

use symphonia_core::audio::RawSampleBuffer;
use std::result::Result;

fn decode(bytes: Vec<u8>) -> Result<RawSampleBuffer<f32>, symphonia_core::errors::Error> {
    use symphonia::default;
    use symphonia_core::io::{MediaSourceStream, MediaSourceStreamOptions};
    use symphonia_core::probe::Hint;
    use symphonia_core::meta::MetadataOptions;
    use symphonia_core::formats::FormatOptions;
    use symphonia_core::codecs::DecoderOptions;
    use symphonia_core::audio::SignalSpec;
    use std::io::Cursor;
    let registry = default::get_codecs();
    let probe = default::get_probe();
    let med_source_stream_options = MediaSourceStreamOptions::default();
    let cursor = Cursor::new(bytes);
    let stream = MediaSourceStream::new(Box::new(cursor), med_source_stream_options);
    let mut probe_result = probe
        .format(
            &Hint::new(),
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default())
        .unwrap();
    let track = probe_result.format.default_track().unwrap();
    let track_id = track.id;
    let mut decoder = registry.make(&track.codec_params, &DecoderOptions::default()).unwrap();
    let time_base = track.codec_params.time_base;
    let duration = track.codec_params.n_frames.map(|frames| track.codec_params.start_ts + frames).unwrap();
    let signal_spec = SignalSpec::new(
        track.codec_params.sample_rate.unwrap(),
        track.codec_params.channels.unwrap()
    );
    let mut song_buf = RawSampleBuffer::<f32>::new(duration, signal_spec);
    let result = loop {
        let packet = match probe_result.format.next_packet() {
            Err(err) => break err,
            Ok(packet) => packet,
        };

        // from Symphonia-play example - discard incorrect packets
        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                song_buf.copy_interleaved_ref(decoded);
            },
            Err(symphonia_core::errors::Error::DecodeError(err)) => {
                println!("Audio decode error: {}", err);
            },
            Err(err) => break err,
        }
    };
    match result {
        symphonia_core::errors::Error::ResetRequired => Ok(song_buf),
        _ => Err(result),
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, receiver: Receiver<AudioCommand>, sender: Sender<StreamError>) -> Result<Stream, symphonia_core::errors::Error>
where
    T: cpal::Sample,
{
    // song info
    let music1 = decode(include_bytes!("Cursed_Church_Set_V2.wav").to_vec())?;
    let music2 = decode(include_bytes!("Cursed_Church_Boss.wav").to_vec())?;

    struct State {
        command: AudioCommand,
        sample: i32,
    };
    let mut state = State {
        command: AudioCommand::Stop,
        sample: 0,
    };
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // react to stream events and read or write stream data here.
            use AudioCommand::*;
            use TryRecvError::*;
            let state = &mut state;
            let play = |song| {
                // grab the next sample and assign it
                for sample in data.iter_mut() {
                    *sample = T::from_sample(
                }
            };
            let mut silence = || {
                for sample in data.iter_mut() {
                    *sample = Sample::from(&0.0);
                }
            };
            match (state.command, receiver.try_recv()) {
                (_, Err(Disconnected)) => (),
                (_, Ok(Play(song))) => {
                    // (re)start playing the given song
                    state.sample = 0;
                    play(song);
                },
                (_, Ok(Stop)) => {
                    // start silence
                    state.command = Stop;
                    silence();
                },
                (Stop, Err(Empty)) => silence(),
                (Play(song), Err(Empty)) => play(song),
            }
        },
        move |err| {
            // react to errors here.
            sender.send(err).ok();
            // if we get an error sending the error, there's nothing we can do (i think)
        },
    );
    Ok(stream.unwrap())
}

impl Audio {
    pub fn new() -> Self {
        // stream_handle.play_raw(music1.convert_samples()).unwrap();
        println!("Should be playing");

        let (audio_sender, receiver) = unbounded();
        let (sender, audio_error_receiver) = unbounded();

        let host = cpal::default_host();
        let device = host.default_output_device()
            .expect("Failed to find default audio output device");
        let mut supported_configs_range = device.supported_output_configs()
            .expect("error while querying configs");
        let config = supported_configs_range.next()
            .expect("no supported config?!")
            .with_max_sample_rate();
        let sample_format = config.sample_format();
        let stream = match sample_format {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), receiver, sender),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), receiver, sender),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), receiver, sender),
        };

        Self {
            timer: 0.0,
            current_song: None,
            stream,
            audio_sender,
            audio_error_receiver,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        use TryRecvError::*;
        match self.audio_error_receiver.try_recv() {
            Err(Empty) => (),
            Err(Disconnected) => println!("Audio player died!"),
            Ok(err) => {
                println!("Audio error: {}", err)
            },
        };

        if let Some(song) = &self.current_song {
            let limit = match song {
                Song::Church => CHURCH_TIME,
                Song::Boss => BOSS_TIME,
            };
            self.timer += delta_time;
            if self.timer >= limit {
                self.timer = 0.0;
                self.play(*song);
            }
        }
    }

    pub fn play(&mut self, song: Song) {
        self.timer = 0.0;
        // let file = match song {
        //     Song::Church => self.music1.clone(),
        //     Song::Boss => self.music2.clone(),
        // };
    }
}
