use sdl2::event::Event;
use sdl2::sys;
use std::mem;

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
