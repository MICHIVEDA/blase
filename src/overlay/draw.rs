use std::error::Error;

use x11rb::{
    connection::Connection, protocol::xproto::{
        Arc, ConnectionExt as _, CoordMode, CreateGCAux, Gcontext, Point, Rectangle, Screen, Window, poly_rectangle,
    }, rust_connection::RustConnection,
};

pub fn create_graphic_context(
    conn: &RustConnection,
    window: Window,
) -> Result<Gcontext, Box<dyn Error>> {
    let gc_id = conn.generate_id()?;
    conn.create_gc(
        gc_id,
        window,
        &CreateGCAux::new().foreground(0xAF0F0000).line_width(4),
    )?
    .check()?;

    Ok(gc_id)
}

pub fn draw_circle(
    conn: &RustConnection,
    window: Window,
    gc_id: Gcontext,
    center_x: i16,
    center_y: i16,
    radius: u16,
) -> Result<(), Box<dyn Error>> {
    let circle = Arc {
        x: center_x - radius as i16,
        y: center_y - radius as i16,
        width: radius * 2,
        height: radius * 2,
        angle1: 0,
        angle2: 360 * 64,
    };

    conn.poly_fill_arc(window, gc_id, &[circle]);

    Ok(())
}
