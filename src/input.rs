use crate::math::{VecArith, VecCast, VecComponents, VecMagnitude};
use crate::Camera;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::sys;
use std::collections::HashSet;
use std::mem;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct UserInput {
    pub mouse: MouseInput,
    pub keys: KeysInput,
    pub events: Vec<Event>,
    pub time: Duration,
    timestamp: Instant,
}

impl Default for UserInput {
    fn default() -> Self {
        Self {
            mouse: MouseInput::default(),
            keys: KeysInput::default(),
            events: vec![],
            time: Duration::default(),
            timestamp: Instant::now(),
        }
    }
}

impl UserInput {
    pub(crate) fn clear(&mut self) {
        self.time = self.timestamp.elapsed();
        self.timestamp = Instant::now();
        self.mouse.left.click = false;
        self.mouse.right.click = false;
        self.mouse.wheel = [0.0; 2];
        self.keys.pressed.clear();
        self.events.clear();
    }

    pub(crate) fn handle(&mut self, event: Event) {
        match &event {
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                self.keys.down.insert(*keycode);
            }
            Event::KeyUp {
                keycode: Some(keycode),
                ..
            } => {
                self.keys.down.remove(keycode);
                self.keys.pressed.insert(*keycode);
            }
            Event::MouseMotion { x, y, .. } => {
                self.mouse.raw = [*x, *y];
            }
            Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                MouseButton::Left => {
                    self.mouse.left.down = true;
                }
                MouseButton::Right => {
                    self.mouse.right.down = true;
                }
                _ => {}
            },
            Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                MouseButton::Left => {
                    self.mouse.left.down = false;
                    self.mouse.left.click = true;
                }
                MouseButton::Right => {
                    self.mouse.right.down = false;
                    self.mouse.right.click = true;
                }
                _ => {}
            },
            Event::MouseWheel { x, y, .. } => {
                self.mouse.wheel = [*x as f32, *y as f32];
            }
            _ => {}
        }
        self.events.push(event);
    }
}

#[derive(Debug, Default, Clone)]
pub struct KeysInput {
    pub down: HashSet<Keycode>,
    pub pressed: HashSet<Keycode>,
}

impl KeysInput {
    pub fn wasd_xy_direction(&self) -> [f32; 2] {
        let mut delta = [0.0, 0.0];
        if self.down.contains(&Keycode::W) {
            delta[1] -= 1.0;
        }
        if self.down.contains(&Keycode::A) {
            delta[0] -= 1.0;
        }
        if self.down.contains(&Keycode::S) {
            delta[1] += 1.0;
        }
        if self.down.contains(&Keycode::D) {
            delta[0] += 1.0;
        };
        delta.normal()
    }
}

#[derive(Debug, Default, Clone)]
pub struct MouseInput {
    pub raw: [i32; 2],
    pub wheel: [f32; 2],
    pub left: MouseButtonInput,
    pub right: MouseButtonInput,
}

impl MouseInput {
    pub fn position(&self, camera: &Camera) -> [f32; 2] {
        self.raw
            .cast()
            .div(camera.resolution_scale)
            .div(camera.zoom)
            .add(camera.eye.xy())
    }
}

#[derive(Debug, Default, Clone)]
pub struct MouseButtonInput {
    pub click: bool,
    pub down: bool,
}

pub(crate) fn poll_event() -> Option<Event> {
    unsafe {
        let mut raw = mem::MaybeUninit::uninit();
        let has_pending = sys::SDL_PollEvent(raw.as_mut_ptr()) == 1;

        if has_pending {
            Some(Event::from_ll(raw.assume_init()))
        } else {
            None
        }
    }
}
