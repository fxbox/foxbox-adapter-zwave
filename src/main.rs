extern crate openzwave;
use openzwave::{options, manager, notification, controller};
use std::time::Duration;
use std::{fs, thread, io};
use std::sync::Mutex;

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

struct Program {
    controller: Mutex<Option<controller::Controller>>
}

impl Program {
    pub fn new() -> Program {
        Program {
            controller: Mutex::new(None)
        }
    }
}

impl manager::NotificationWatcher for Program {
    fn on_notification(&self, notification: notification::Notification) {
        println!("{:?}", notification);

        {
            let mut controller = self.controller.lock().unwrap();
            if controller.is_none() {
                *controller = controller::Controller::new(notification.get_home_id());
                println!("Found controller: {:?}", *controller);
            }
        }
    }
}

fn main() {
    let mut options = options::Options::create("./config/", "", "--SaveConfiguration true --DumpTriggerLevel 0 --ConsoleOutput false").unwrap();
    let mut manager = manager::Manager::create(options).unwrap();
    let mut program = Program::new();

    manager.add_watcher(program).unwrap();

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


    println!("Press ENTER to exit.");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    println!("Exiting...");
}
