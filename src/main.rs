extern crate openzwave;
use openzwave::manager;

fn main() {
    manager::options::create("./config/", "", "--SaveConfiguration=true --DumpTriggerLevel=0");
    println!("Hello, world!");
}
