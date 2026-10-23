use std::error::Error;

use x11rb::{connection::Connection, protocol::xproto::ConnectionExt as _};
use super::{draw, pointer_pos, window};

pub fn run() -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = window::connect()?;
    let window = window::create_window(&conn, screen_num)?;
    println!("overlay window: {:#x}", window);

    let gc_id = draw::create_graphic_context(&conn, window)?;

    let mut last_pos = None;

    loop {
        let pos = pointer_pos::pointer_pos(&conn, window)?;

        if last_pos != Some(pos) {
            let (x, y) = pos;

            conn.clear_area(false, window, 0, 0, 0, 0)?;
            draw::draw_circle(&conn, window, gc_id, x, y, 40)?;

            conn.flush()?;
            last_pos = Some(pos);
        }
    }
}
