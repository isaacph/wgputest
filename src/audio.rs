use std::{io::{BufReader, Cursor}, cell::Cell};

use rodio::{*, source::SamplesConverter};

pub struct Audio {
    timer: f32,
    stream: Option<(OutputStream, OutputStreamHandle)>,
    music1: Vec<u8>,
    music2: Vec<u8>,
    current_song: Option<Song>,
    sink: Option<Sink>,
}

#[derive(Copy, Clone)]
pub enum Song {
    Church,
    Boss
}

const CHURCH_TIME: f32 = 88.0;
const BOSS_TIME: f32 = 117.0;

impl Audio {
    pub fn new() -> Self {
        // use rodio::cpal::traits::{HostTrait};
        // let host = cpal::default_host();
        // let devices = host.output_devices().unwrap();
        // let mut string = String::new();
        // for device in devices {
        //     string += &(device.name().unwrap() + &String::from(", "));
        // }
        // panic!("devices: {}", string);
        let music1 = include_bytes!("Cursed_Church_Set_V2.mp3").to_vec();
        // let music2 = include_bytes!("Cursed_Church_Boss.mp3").to_vec();
        let music2 = vec![];
        // stream_handle.play_raw(music1.convert_samples()).unwrap();

        Self {
            timer: 0.0,
            stream: None,
            music1,
            music2,
            sink: None,
            current_song: None,
        }
    }

    pub fn init_audio(&mut self) -> Option<()> {
        if self.stream.is_none() {
            let stream = Some(OutputStream::try_default().unwrap());
            self.stream = stream;
            self.current_song.map(|song| self.play(song));
            self.timer = 0.0;
            return Some(())
        }
        return None
    }
    
    pub fn update(&mut self, delta_time: f32) {
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
        if let Some(sink) = &self.sink {
            sink.stop();
            self.sink = None;
        }
        self.current_song = Some(song);
        self.timer = 0.0;
        self.stream.as_mut().map(|(_, stream_handle)| {
            let file = match song {
                Song::Church => self.music1.clone(),
                Song::Boss => self.music2.clone(),
            };
            let music = Decoder::new(Cursor::new(file)).unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(music);
            self.sink = Some(sink);
        });
    }
}
