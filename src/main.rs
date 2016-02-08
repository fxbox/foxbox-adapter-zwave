extern crate openzwave;
use openzwave::{options, manager};

fn main() {
    options::create("./config/", "", "--SaveConfiguration=true --DumpTriggerLevel=0").unwrap();
    options::get().unwrap().lock().unwrap();
    manager::create().unwrap();
    manager::get().unwrap();
    manager::destroy();
    options::destroy().unwrap();
    println!("Hello, world!");
}
