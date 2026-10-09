#![allow(unused)]

mod input;
mod overlay;

use std::{
    error::Error,
    sync::{atomic::AtomicBool, Arc},
    thread,
};

fn main() -> Result<(), Box<dyn Error>> {
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

    let (conn, screen_num) = overlay::window::connect()?;
    let window = overlay::window::create_window(&conn, screen_num)?;
    println!("overlay window: {:#x}", window);

    let gc_id = overlay::draw::create_graphic_context(&conn, window)?;
    overlay::draw::draw_circle(&conn, window, gc_id, 1920 / 2 as i16, 1080 / 2 as i16, 40)?;
    loop {}
}
