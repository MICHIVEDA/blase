use std::error::Error;

use x11rb::{protocol::xproto::{ConnectionExt, QueryPointerReply, Window}, rust_connection::RustConnection};


pub fn pointer_pos(conn: &RustConnection, window: Window) -> Result<(i16, i16), Box<dyn Error>> {

    let reply: QueryPointerReply = conn.query_pointer(window)?
	.reply()?;

    Ok((reply.win_x, reply.win_y))
}
