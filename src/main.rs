#![allow(unused)]

mod input;
mod overlay;
mod a11y;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let conn = atspi::AccessibilityConnection::new().await?;

    let Some(frame) = a11y::get_active_window(&conn).await? else {
	println!("no active frame");
	return Ok(());
    };

    let targets = a11y::collect_targets(&conn, &frame).await?;

    println!("{} targets", targets.len());
    for target in &targets {
        println!(
            "{:?} at ({}, {}), size {}x{}",
            target.role, target.x, target.y, target.width, target.height
        );
    }

    Ok(())
}
