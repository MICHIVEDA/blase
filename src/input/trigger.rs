use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use evdev::{EventSummary, KeyCode};

const TRIGGER_KEY: KeyCode = KeyCode::KEY_LEFTMETA;

// trigger key listener
pub fn watch_trigger_key(held: Arc<AtomicBool>) {
    let mut keyboard = super::devices::find_keyboard(TRIGGER_KEY).expect("no device found");

    loop {
        let events = keyboard.fetch_events().expect("failed to read events");

        for event in events {
            match event.destructure() {
                EventSummary::Key(_, TRIGGER_KEY, 1) => {
                    held.store(true, Ordering::Relaxed);
                    println!("held");
                }
                EventSummary::Key(_, TRIGGER_KEY, 0) => {
                    held.store(false, Ordering::Relaxed);
                    println!("released");
                }
                _ => {}
            }
        }
    }
}
