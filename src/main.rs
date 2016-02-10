extern crate openzwave;
use openzwave::{options, manager};
use std::time::Duration;
use std::thread;
use std::fs;

#[cfg(windows)]
fn get_default_device() {
    "\\\\.\\COM6"
}

#[cfg(unix)]
fn get_default_device() -> Option<&'static str> {
    let default_devices = [
        "/dev/cu.usbserial", // MacOS X
        "/dev/ttyUSB0" // Linux
    ];

    default_devices
        .iter()
        .find(|device_name| fs::metadata(device_name).is_ok())
        .map(|&str| str)
}

fn is_device_usb(device_name: &str) -> bool {
    device_name.to_lowercase().contains("usb")
}

fn main() {
    options::create("./config/", "", "--SaveConfiguration=true --DumpTriggerLevel=0").unwrap();
    options::get().unwrap().lock().unwrap();
    let mut manager = manager::create().unwrap();
    let mut watcher = manager::Watcher::new(
        |notification: manager::Notification| println!("{}", notification.a)
    );

    manager.add_watcher(&mut watcher).unwrap();

    {
        let arg_device = std::env::args()
            .skip(1).last(); // last but not first

        let device = arg_device.as_ref().map(String::as_ref).or(get_default_device()).unwrap();

        println!("found device {}", device);

        match device {
            "usb" => manager.add_usb_driver(),
            _ => manager.add_driver(&device)
        }.unwrap()
    }

    thread::sleep(Duration::from_secs(10));

    manager.remove_watcher(&mut watcher).unwrap();
    manager::destroy();
    options::destroy().unwrap();
    println!("Hello, world!");
}
