use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use evdev::{
    uinput::VirtualDevice, AttributeSet, Device, EventSummary, EventType, InputEvent, KeyCode,
    RelativeAxisCode,
};

const SCROLL_DEVICE_NAME: &str = "blase scroll";
const UNITS_PER_CLICK: i32 = 10;

pub fn create_scroll_device() -> io::Result<VirtualDevice> {
    let buttons = AttributeSet::from_iter([KeyCode::BTN_LEFT]);
    let axes = AttributeSet::from_iter([
        RelativeAxisCode::REL_X,
        RelativeAxisCode::REL_Y,
        RelativeAxisCode::REL_WHEEL,
        RelativeAxisCode::REL_HWHEEL,
    ]);

    VirtualDevice::builder()?
        .name(SCROLL_DEVICE_NAME)
        .with_keys(&buttons)?
        .with_relative_axes(&axes)?
        .build()
}

fn take_clicks(remainder: &mut i32, amount: i32) -> i32 {
    *remainder += amount;
    let clicks = *remainder / UNITS_PER_CLICK;
    *remainder -= clicks * UNITS_PER_CLICK;
    clicks
}

fn scroll_event(axis: RelativeAxisCode, clicks: i32) -> Option<InputEvent> {
    if clicks == 0 {
        return None;
    };

    Some(InputEvent::new_now(EventType::RELATIVE.0, axis.0, clicks))
}

fn emit_scroll(scroll_device: &mut VirtualDevice, h_clicks: i32, v_clicks: i32) -> io::Result<()> {
    let mut events = Vec::new();

    if let Some(event) = scroll_event(RelativeAxisCode::REL_WHEEL, -v_clicks) {
        events.push(event);
    }

    if let Some(event) = scroll_event(RelativeAxisCode::REL_HWHEEL, h_clicks) {
        events.push(event);
    }

    if events.is_empty() {
        return Ok(());
    }

    scroll_device.emit(&events)
}

pub fn run_scroll(
    mut touchpad: Device,
    mut scroll_device: VirtualDevice,
    held: Arc<AtomicBool>,
) -> io::Result<()> {
    let mut grabbed = false;
    let (mut dx, mut dy) = (0, 0);
    let (mut h_rem, mut v_rem) = (0, 0);

    loop {
        let events: Vec<InputEvent> = touchpad.fetch_events()?.collect();

        for event in events {
            match event.destructure() {
                EventSummary::RelativeAxis(_, RelativeAxisCode::REL_X, value) => dx += value,
                EventSummary::RelativeAxis(_, RelativeAxisCode::REL_Y, value) => dy += value,
                EventSummary::Synchronization(..) => {
                    let held_bool = held.load(Ordering::Relaxed);

                    if held_bool && !grabbed {
                        touchpad.grab()?;
                        grabbed = true;
                    } else if !held_bool && grabbed {
                        touchpad.ungrab()?;
                        grabbed = false;
                        (h_rem, v_rem) = (0, 0);
                    }

                    if grabbed {
                        let h_clicks = take_clicks(&mut h_rem, dx);
                        let v_clicks = take_clicks(&mut v_rem, dy);
                        emit_scroll(&mut scroll_device, h_clicks, v_clicks)?;
                    }

                    (dx, dy) = (0, 0);
                }

                _ => {}
            }
        }
    }
}
