use evdev::{Device, KeyCode, RelativeAxisCode};

const OWN_PREFIX: &str = "blase";

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
