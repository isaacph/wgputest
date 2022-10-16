use std::{io::{BufReader, Cursor}, cell::Cell};

use rodio::{*, source::SamplesConverter};

pub struct Audio {
    timer: Cell<f32>,
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
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
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let music1 = include_bytes!("Cursed_Church_Set_V2.mp3").to_vec();
        let music2 = include_bytes!("Cursed_Church_Boss.mp3").to_vec();
        // stream_handle.play_raw(music1.convert_samples()).unwrap();
        println!("Should be playing");

        Self {
            timer: Cell::new(0.0),
            stream,
            stream_handle,
            music1,
            music2,
            sink: None,
            current_song: None,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        if let Some(song) = &self.current_song {
            let limit = match song {
                Song::Church => CHURCH_TIME,
                Song::Boss => BOSS_TIME,
            };
            self.timer.set(self.timer.get() + delta_time);
            if self.timer.get() >= limit {
                self.timer.set(0.0);
                self.play(*song);
            }
        }
    }

    pub fn play(&mut self, song: Song) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.timer.set(0.0);
        let file = match song {
            Song::Church => self.music1.clone(),
            Song::Boss => self.music2.clone(),
        };
        let music = Decoder::new(Cursor::new(file)).unwrap();
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        sink.append(music);
        self.sink = Some(sink);
    }
}
