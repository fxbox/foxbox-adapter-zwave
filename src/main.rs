extern crate openzwave;
use openzwave::{options, manager, notification};
use std::time::Duration;
use std::{fs, thread, io};

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

fn main() {
    let mut options = options::Options::create("./config/", "", "--SaveConfiguration=true --DumpTriggerLevel=0").unwrap();

    // TODO: The NetworkKey should really be derived from something unique
    //       about the foxbox that we're running on. This particular set of
    //       values happens to be the default that domoticz uses.
    options::Options::add_option_string(&mut options, "NetworkKey", "0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10", false).unwrap();

    let mut manager = manager::Manager::create(options).unwrap();
    let mut watcher = manager::Watcher::new(
        |notification: notification::Notification| println!("{:?}", notification)
    );

    manager.add_watcher(&mut watcher).unwrap();

    {
        let arg_device: Option<String> = std::env::args()
            .skip(1).last(); // last but not first

        let device = match arg_device {
            Some(ref x) => x as &str,
            None => get_default_device().expect("No device found.")
        };

        println!("found device {}", device);

        match device {
            "usb" => manager.add_usb_driver(),
            _ => manager.add_driver(&device)
        }.unwrap()
    }


    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    manager.remove_watcher(&mut watcher).unwrap();
    println!("Hello, world!");
}
