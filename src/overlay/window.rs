use crate::Error;
use x11rb::protocol::{shape::{SK, SO}};
use x11rb::protocol::{shape::ConnectionExt as _};
use x11rb::protocol::xproto::ClipOrdering;
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ColormapAlloc, ConnectionExt as _, CreateWindowAux, Screen, VisualClass,
        Visualid, Window, WindowClass,
    },
    rust_connection::RustConnection,
};

/* Finds a visual overlay our window can use to be transperent.
A visual is a pixel format supported by the screen. We need support for the Alpha channel and
therefore need 32 Bit depth, 8 bit pro channel Alpha, Red, Green, Blue.
 */
fn find_argb_visual(screen: &Screen) -> Option<Visualid> {
    screen
        .allowed_depths
        .iter()
        .find(|depth| depth.depth == 32)?
        .visuals
        .iter()
        .find(|visual| visual.class == VisualClass::TRUE_COLOR)
        .map(|visual| visual.visual_id)
}

pub fn connect() -> Result<(RustConnection, usize), Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    Ok((conn, screen_num))
}

/* Creates an empty list of input rectangles which means the window does not take input and instead
it's passed to what's underneath.
 */
fn make_click_through(conn: &RustConnection, window: Window) -> Result<(), Box<dyn Error>> {
    conn.shape_rectangles(
        SO::SET, // new region is the rectangles
        SK::INPUT, // rectangle is where input is accepted           
        ClipOrdering::UNSORTED, 
        window,
        0,
        0,
        &[], // list is empty -> no rectangles all input falls through
    )?
    .check()?;

    Ok(())
}

pub fn create_window(
    conn: &RustConnection,
    screen_num: usize,
) -> Result<Window, Box<dyn std::error::Error>> {
    let screen = &conn.setup().roots[screen_num];

    println!(
        "screen {}: {}x{}, root depth: {}, root window {:x}",
        screen_num, screen.width_in_pixels, screen.height_in_pixels, screen.root_depth, screen.root
    );

    let visual_id: Visualid =
        find_argb_visual(screen).ok_or("no 32-bit TrueColor visual on this screen")?;
    println!("{:#x}", visual_id);

    let colormap_id = conn.generate_id()?;
    let win_id = conn.generate_id()?;

    conn.create_colormap(ColormapAlloc::NONE, colormap_id, screen.root, visual_id)?
        .check()?;

    conn.create_window(
        32,
        win_id,
        screen.root,
        0,
        0,
        screen.width_in_pixels,
        screen.height_in_pixels,
        0,
        WindowClass::INPUT_OUTPUT,
        visual_id,
        &CreateWindowAux::new()
            .colormap(colormap_id)
            .border_pixel(0)
            .override_redirect(1)
            .background_pixel(0x00FF0AFA),
    )?
    .check()?;

    make_click_through(conn, win_id)?;

    conn.map_window(win_id)?;
    conn.flush()?;

    Ok(win_id)
}
