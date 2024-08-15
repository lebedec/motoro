use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::sys;
use std::mem;

#[derive(Debug, Default, Clone)]
pub struct UserInput {
    pub mouse: MouseInput,
    pub events: Vec<Event>,
}

impl UserInput {
    pub(crate) fn clear(&mut self) {
        self.mouse.left.click = false;
        self.mouse.right.click = false;
        self.mouse.wheel = [0.0; 2];
        self.events.clear();
    }

    pub(crate) fn handle(&mut self, event: Event) {
        match &event {
            Event::MouseMotion { x, y, .. } => {
                self.mouse.position = [*x as f32, *y as f32];
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
pub struct MouseInput {
    pub position: [f32; 2],
    pub wheel: [f32; 2],
    pub left: MouseButtonInput,
    pub right: MouseButtonInput,
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
