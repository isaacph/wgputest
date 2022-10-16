use cgmath::{Vector4, Vector2, Matrix4, SquareMatrix, Vector3};

use crate::graphics::{text::{FontMetricsInfo, FontInfoContainer, BaseFontInfoContainer}, textured};

pub struct Chatbox {
    font_info: FontMetricsInfo,
    visible_lines: i32,
    line_height: f32,
    history_length: i32,
    typing: String,
    history: Vec<String>,
    width: f32,
    height: f32,
    flicker_timer: f32,
    typing_flicker: bool,
    fade_timer: f32
}

pub const BAR_FLICKER_TIME: f32 = 0.6;
pub const FADE_START_TIME: f32 = 3.0;
pub const FADE_TIME: f32 = 1.0;

impl Chatbox {
    pub fn new(font_info: FontMetricsInfo, visible_lines: i32, line_height: f32, history_length: i32, width: f32) -> Self {
        assert!(visible_lines >= 0 && history_length >= 0 && width >= 0.0);
        Chatbox {
            font_info,
            visible_lines,
            line_height,
            history_length,
            typing: String::new(),
            history: Vec::new(),
            width,
            height: (visible_lines + 1) as f32 * line_height,
            flicker_timer: 0.0,
            typing_flicker: false,
            fade_timer: f32::MAX
        }
    }

    pub fn println(&mut self, line: &str) {
        println!("{}", line);
        let mut lines: Vec<String> = self.font_info.split_lines(line, Some(self.width));
        let add_len = std::cmp::min(self.history_length as usize, lines.len()) as i32;
        lines.drain(0..(std::cmp::max(0, lines.len() as i32 - add_len)) as usize);
        let history_remove = 
            std::cmp::max(0, self.history.len() as i32 - (self.history_length - add_len)) as usize;
        self.history.drain(0..history_remove);
        self.history.append(&mut lines);
        self.fade_timer = 0.0;
    }

    fn get_visible_history_empty_lines(&self) -> i32 {
        std::cmp::max(0, self.visible_lines - self.history.len() as i32)
    }

    pub fn get_visible_history(&self) -> Vec<&str> {
        let mut vec = Vec::new();
        for i in (std::cmp::max(0, self.history.len() as i32 - self.visible_lines) as usize)..self.history.len() {
            vec.push(self.history[i].as_str());
        }
        vec
    }

    pub fn get_typing(&self) -> &String {
        &self.typing
    }

    pub fn add_typing(&mut self, c: char) {
        self.typing.push(c);
    }

    pub fn remove_typing(&mut self, count: i32) {
        assert!(count >= 0);
        self.typing.truncate(std::cmp::max(0, self.typing.len() as i32 - count) as usize);
    }

    pub fn erase_typing(&mut self) {
        self.typing.clear();
    }

    pub fn set_typing_flicker(&mut self, typing_flicker: bool) {
        self.typing_flicker = typing_flicker;
        self.flicker_timer = 0.0;
        self.fade_timer = 0.0;
    }

    pub fn update(&mut self, delta_time: f32) {
        self.fade_timer += delta_time;
        if self.typing_flicker {
            self.flicker_timer += delta_time;
            while self.flicker_timer > BAR_FLICKER_TIME {
                self.flicker_timer -= BAR_FLICKER_TIME;
            }
        }
    }

    pub fn render(&self) -> (textured::Instance, Vec<(String, Vector2<f32>, Vector4<f32>)>) {
        let is_fade = self.fade_timer > FADE_START_TIME && !self.typing_flicker;
        let mut fade = 1.0;
        if is_fade {
            fade = 1.0 - f32::max(0.0, (self.fade_timer - FADE_START_TIME) / FADE_TIME);
        }

        let color = Vector4::new(1.0, 1.0, 1.0, 1.0) * fade;
        let background_color = Vector4::new(0.0, 0.0, 0.0, 0.6) * fade;
        let position = Vector2::new(0.0, 0.0);

        let background_instance = textured::Instance {
            color: background_color,
            position: Vector2::new(position.x + self.width / 2.0, position.y + self.height / 2.0),
            scale: Vector2::new(self.width, self.height),
        };
        
        let start = Vector2::new(
            position.x,
            position.y + self.line_height * (self.get_visible_history_empty_lines() + 1) as f32
        );
        let (pos, mut instances) = self.get_visible_history().iter().fold((start, vec![]), |(mut pos, mut instances), line| {
            instances.push((line.to_string(), pos, color));
            pos += Vector2::new(0.0, self.line_height);
            (pos, instances)
        });

        let typing_line = if self.flicker_timer > BAR_FLICKER_TIME / 2.0 && self.typing_flicker {
            self.typing.to_owned() + "|"
        } else {
            self.typing.to_owned()
        };
        instances.push((typing_line, pos, color));
        (background_instance, instances)
    }
}
