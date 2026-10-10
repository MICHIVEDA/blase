#![allow(unused)]

mod input;
mod overlay;

use std::{
    error::Error,
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

use x11rb::{
    connection::Connection,
    protocol::xproto::{clear_area, ConnectionExt},
};

fn main() -> Result<(), Box<dyn Error>> {
    
    let touchpad = input::devices::find_tracking_devices(Some("CHERRY"))
        .into_iter()
        .next()
        .expect("no matching touchpad");

    let scroll_device = input::scroll::create_scroll_device().expect("failed");

    let held = Arc::new(AtomicBool::new(false));

    let held_for_keyboard = Arc::clone(&held);
    thread::spawn(move || input::trigger::watch_trigger_key(held_for_keyboard));

    input::scroll::run_scroll(touchpad, scroll_device, held).expect("scroll loop failed");
    

    let (conn, screen_num) = overlay::window::connect()?;
    let window = overlay::window::create_window(&conn, screen_num)?;
    println!("overlay window: {:#x}", window);

    let gc_id = overlay::draw::create_graphic_context(&conn, window)?;

    let mut last_pos = None;

    loop {
        let pos = overlay::pointer_pos::pointer_pos(&conn, window)?;

        if last_pos != Some(pos) {
            let (x, y) = pos;

            conn.clear_area(false, window, 0, 0, 0, 0)?;
            overlay::draw::draw_circle(&conn, window, gc_id, x, y, 40)?;

            conn.flush()?;
            last_pos = Some(pos);
        }
    }
}
