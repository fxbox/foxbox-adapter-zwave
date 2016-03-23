extern crate openzwave_stateful as openzwave;
use openzwave::InitOptions;
use openzwave::{ ValueGenre, ValueID, ZWaveNotification };

use std::{ io, thread };
use std::io::Write;
use std::sync::mpsc;

fn display_prompt() {
    print!("> ");
    io::stdout().flush().unwrap(); // https://github.com/rust-lang/rust/issues/23818
}

fn spawn_notification_thread(rx: mpsc::Receiver<ZWaveNotification>) {
    thread::spawn(move || {
        for notification in rx {
            println!("{}", notification);
        }
    });
}

fn main() {

    let options = InitOptions {
        device: std::env::args().skip(1).last() // last but not first
    };

    let (ozw, rx) = openzwave::init(&options).unwrap();
    spawn_notification_thread(rx);

    println!("Enter `exit` (or Control-D) to exit.");
    let mut input = String::new();
    loop {
        input.clear();
        display_prompt();
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
            "add-node" => {
                if tokens.len() < 2 || tokens.len() > 3 {
                    println!("Syntax: add-node <home_id> [secure]");
                    continue;
                }
                if let Ok(home_id) = u32::from_str_radix(tokens[1], 16) {
                    let secure = tokens.get(2).unwrap_or(&"") == &"secure";
                    if ozw.add_node(home_id, secure).is_err() {
                        println!("Error adding node");
                    } else {
                        println!("\nPress Include button on node you wish to add\n");
                    }
                } else {
                    println!("Must specify a numeric home_id (in base 16)");
                }
            }
            "controllers"           => {
                let state = ozw.get_state();
                let controllers = state.get_controllers();
                for controller in controllers {
                    println!("{}", controller);
                }
            },
            "controllers_dbg"       => println!("{:?}\n", ozw.get_state().get_controllers()),
            "nodes" => {
                let state = ozw.get_state();
                let nodes_map = state.get_nodes_map();
                for (controller, node_set) in nodes_map {
                    println!("{}", controller);
                    for node in node_set.iter() {
                        println!("  Node: {}", node);
                    }
                }
            },
            "nodes_dbg"             => println!("{:?}\n", ozw.get_state().get_nodes()),
            "remove-node" => {
                if tokens.len() != 2 {
                    println!("Syntax: remove-node <home_id>");
                    continue;
                }
                if let Ok(home_id) = u32::from_str_radix(tokens[1], 16) {
                    if ozw.remove_node(home_id).is_err() {
                        println!("Error removing node");
                    } else {
                        println!("\nPress Exclude button on node you wish to remove\n");
                    }
                } else {
                    println!("Must specify a numeric home_id (in base 16)");
                }
            }
            "set"                   => {
                if tokens.len() != 4 {
                    println!("Syntax: set <home_id> <value_id> <value>");
                    continue;
                }
                match (u32::from_str_radix(tokens[1], 16), u64::from_str_radix(tokens[2], 16)) {
                    (Ok(home_id), Ok(id)) => {
                        let vid = ValueID::from_packed_id(home_id, id);
                        let state = ozw.get_state();
                        let value_ids = state.get_values();
                        if value_ids.contains(&vid) {
                            if vid.set_string(tokens[3]).is_err() {
                                println!("Error setting value");
                            }
                        } else {
                            println!("Unknown ValueID: {:08x} {:016x}", home_id, id);
                        }
                    }
                    _ => {
                        println!("Must specify a numeric home_id and value_id (in base 16)");
                    }
                }
            }
            "values"                => {
                let state = ozw.get_state();
                let value_ids = state.get_values();
                let display_all_values = tokens.get(1).unwrap_or(&"") == &"all";
                for value_id in value_ids {
                    if !display_all_values && value_id.get_genre() != ValueGenre::ValueGenre_User {
                        continue;
                    }
                    println!("{}", value_id);
                }
            },
            "values_dbg"            => println!("{:?}", ozw.get_state().get_values()),
            _                       => println!("Unrecognized command: '{}'", tokens[0]),
        }
    }
    println!("Exiting...");
}
