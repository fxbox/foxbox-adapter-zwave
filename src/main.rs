extern crate openzwave;
use openzwave::{options, manager, controller};
use openzwave::notification::*;
use openzwave::node::*;
use openzwave::value_classes::value_id::{ ValueGenre, ValueID };
use std::{fs, io};
use std::sync::{ Arc, Mutex };
use std::collections::{ BTreeSet, HashMap, HashSet };
use std::io::Write;

#[cfg(windows)]
fn get_default_device() {
    "\\\\.\\COM6"
}

#[cfg(unix)]
fn get_default_device() -> Option<&'static str> {
    let default_devices = [
        "/dev/cu.usbserial", // MacOS X
        "/dev/cu.SLAB_USBtoUART", // MacOS X
        "/dev/ttyUSB0", // Linux
        "/dev/ttyACM0"  // Linux (Aeotech Z-Stick Gen-5)
    ];

    default_devices
        .iter()
        .find(|device_name| fs::metadata(device_name).is_ok())
        .map(|&str| str)
}

#[derive(Debug, Clone)]
struct ProgramState {
    controllers: HashSet<controller::Controller>,
    nodes: BTreeSet<Node>,
    nodes_map: HashMap<controller::Controller, BTreeSet<Node>>,
    value_ids: BTreeSet<ValueID>,
}

impl ProgramState {
    fn new() -> ProgramState {
        ProgramState {
            controllers: HashSet::new(),
            nodes: BTreeSet::new(),
            nodes_map: HashMap::new(),
            value_ids: BTreeSet::new()
        }
    }

    pub fn add_node(&mut self, node: Node) {
        let node_set = self.nodes_map.entry(node.get_controller()).or_insert(BTreeSet::new());
        node_set.insert(node);
        self.nodes.insert(node);
    }

    pub fn remove_node(&mut self, node: Node) {
        if let Some(node_set) = self.nodes_map.get_mut(&node.get_controller()) {
            node_set.remove(&node);
        }
        self.nodes.remove(&node);
    }

    pub fn add_value_id(&mut self, value_id: ValueID) {
        self.value_ids.insert(value_id.clone());
        println!("Added value_id: {:?}", value_id);
    }

    pub fn remove_value_id(&mut self, value_id: ValueID) {
        self.value_ids.remove(&value_id);
    }
}

#[derive(Debug, Clone)]
struct Program {
    state: Arc<Mutex<ProgramState>>
}

impl Program {
    pub fn new() -> Program {
        Program {
            state: Arc::new(Mutex::new(ProgramState::new()))
        }
    }
}

impl manager::NotificationWatcher for Program {
    fn on_notification(&self, notification: Notification) {
        //println!("Received notification: {:?}", notification);

        match notification.get_type() {
            NotificationType::Type_DriverReady => {
                let controller = notification.get_controller();
                let mut state = self.state.lock().unwrap();
                if !state.controllers.contains(&controller) {
                    println!("Found new controller: {:?}", controller);
                    state.controllers.insert(controller);
                }
            },
            NotificationType::Type_NodeAdded => {
                let mut state = self.state.lock().unwrap();
                let node = notification.get_node();
                println!("NodeAdded: {:?}", node);
                state.add_node(node);
            },
            NotificationType::Type_NodeRemoved => {
                let mut state = self.state.lock().unwrap();
                let node = notification.get_node();
                println!("NodeRemoved: {:?}", node);
                state.remove_node(node);
            },
            NotificationType::Type_NodeEvent => {
                println!("NodeEvent");
            },
            NotificationType::Type_ValueAdded => {
                let mut state = self.state.lock().unwrap();
                let value_id = notification.get_value_id();
                println!("ValueAdded: {:?}", value_id);
                state.add_value_id(value_id);
            },
            NotificationType::Type_ValueChanged => {
                let mut state = self.state.lock().unwrap();
                let value_id = notification.get_value_id();
                println!("ValueChanged: {:?}", value_id);
                state.add_value_id(value_id);
                // TODO: Tell somebody that the value changed
            },
            NotificationType::Type_ValueRemoved => {
                let mut state = self.state.lock().unwrap();
                let value_id = notification.get_value_id();
                println!("ValueRemoved: {:?}", value_id);
                state.remove_value_id(value_id);
            },
            _ => {
                //println!("Unknown notification: {:?}", notification);
            }
        }
    }
}

fn main() {
    let mut options = options::Options::create("./config/", "", "--SaveConfiguration true --DumpTriggerLevel 0 --ConsoleOutput false").unwrap();

    // TODO: The NetworkKey should really be derived from something unique
    //       about the foxbox that we're running on. This particular set of
    //       values happens to be the default that domoticz uses.
    options::Options::add_option_string(&mut options, "NetworkKey", "0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10", false).unwrap();

    let mut manager = manager::Manager::create(options).unwrap();
    let program = Program::new();

    manager.add_watcher(program.clone()).unwrap();

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


    println!("Enter `exit` (or Control-D) to exit.");
    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush().unwrap(); // https://github.com/rust-lang/rust/issues/23818
        // Note: read_line includes the newline character.
        if let Ok(n) = io::stdin().read_line(&mut input) {
            if n == 0 {
                // End-of-file (either Control-D or we were redirected).
                break;
            }
        } else {
            println!("Error reading stdin");
            break;
        }

        let tokens: Vec<_> = input.split_whitespace().collect();
        if tokens.len() == 0 {
            continue;
        }

        match tokens[0] {
            "args"                  => println!("args = {:?}", tokens),
            "exit" | "q" | "quit"   => break,
            "controllers"           => {
                let ref controllers = program.state.lock().unwrap().controllers;
                for controller in controllers {
                    println!("{}", controller);
                }
            },
            "controllers_dbg"       => println!("{:?}\n", program.state.lock().unwrap().controllers),
            "nodes" => {
                let mut program_state = program.state.lock().unwrap();
                let ref mut nodes_map = program_state.nodes_map;
                for (ref controller, ref mut node_set) in nodes_map {
                    println!("{}", controller);
                    for node in node_set.iter() {
                        println!("  Node: {}", node);
                    }
                }
            },
            "nodes_dbg"             => println!("{:?}\n", program.state.lock().unwrap().nodes),
            "values"                => {
                let ref value_ids = program.state.lock().unwrap().value_ids;
                for value_id in value_ids {
                    if value_id.get_genre() != ValueGenre::ValueGenre_User {
                        continue;
                    }
                    println!("{}", value_id);
                }
            },
            "values_dbg"            => println!("{:?}", program.state.lock().unwrap().value_ids),
            _                       => println!("Unrecognized command: '{}'", tokens[0]),
        }
    }
    println!("Exiting...");
}
