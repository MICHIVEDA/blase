use evdev::{
    AttributeSet, Device, EventSummary, EventType, InputEvent, KeyCode, RelativeAxisCode,
    uinput::VirtualDevice,
};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

const TRIGGER_KEY: KeyCode = KeyCode::KEY_LEFTMETA;
const OWN_PREFIX: &str = "blase";
const SCROLL_DEVICE_NAME: &str = "blase scroll";

const UNITS_PER_CLICK: i32 = 10;
// finding device

fn name_matches(device: &Device, wanted: Option<&str>) -> bool {
    wanted.is_none_or(|w| device.name().is_some_and(|n| n.contains(w)))
}

fn is_relative_pointer(device: &Device) -> bool {
    device.supported_relative_axes().is_some_and(|axes| {
        axes.contains(RelativeAxisCode::REL_X) && axes.contains(RelativeAxisCode::REL_Y)
    })
}

fn is_own_device(device: &Device) -> bool {
    device.name().is_some_and(|n| n.starts_with(OWN_PREFIX))
}

fn has_key(device: &Device, key: KeyCode) -> bool {
    device
        .supported_keys()
        .is_some_and(|keys| keys.contains(key))
}

pub fn find_tracking_devices(name: Option<&str>) -> Vec<Device> {
    evdev::enumerate()
        .filter(|(_path, device)| {
            !is_own_device(device) && name_matches(device, name) && is_relative_pointer(device)
        })
        .map(|(_path, device)| device)
        .collect()
}

pub fn find_keyboard(key: KeyCode) -> Option<Device> {
    evdev::enumerate()
        .map(|(_path, device)| device)
        .find(|device| has_key(device, key))
}

// trigger key listener
fn watch_trigger_key(held: Arc<AtomicBool>) {
    let mut keyboard = find_keyboard(TRIGGER_KEY).expect("no device found");

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

// Scrolling
fn create_scroll_device() -> io::Result<VirtualDevice> {
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

fn run_scroll(
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

fn main() {
    let touchpad = find_tracking_devices(Some("CHERRY"))
        .into_iter()
        .next()
        .expect("no matching touchpad");

    let scroll_device = create_scroll_device().expect("failed");

    let held = Arc::new(AtomicBool::new(false));

    let held_for_keyboard = Arc::clone(&held);
    thread::spawn(move || watch_trigger_key(held_for_keyboard));

    run_scroll(touchpad, scroll_device, held).expect("scroll loop failed");
}
