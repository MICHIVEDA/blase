#![allow(unused)]

mod a11y;
mod input;
mod overlay;

use std::{
    error::Error,
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

fn main() {
    /*
    let touchpad = input::devices::find_tracking_devices(Some("CHERRY"))
        .into_iter()
        .next()
        .expect("no matching touchpad");

    let scroll_device = input::scroll::create_scroll_device().expect("failed");

    let held = Arc::new(AtomicBool::new(false));

    let held_for_keyboard = Arc::clone(&held);
    thread::spawn(move || input::trigger::watch_trigger_key(held_for_keyboard));

    input::scroll::run_scroll(touchpad, scroll_device, held).expect("scroll loop failed");

     */

    // thread::sleep(Duration::from_secs(4));

    let overlay_handle = thread::spawn(|| {
        if let Err(e) = overlay::run::run() {
            eprintln!("overlay error {e}");
        }
    });

    let a11y_handle = thread::spawn(|| {
        if let Err(e) = a11y::run() {
            eprintln!("a11y error {e}");
        }
    });

    a11y_handle.join().expect("a11y");
    overlay_handle.join().expect("overlay");
}
