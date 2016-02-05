extern crate openzwave;
use openzwave::manager;

fn main() {
    manager::options::create("./config/", "", "--SaveConfiguration=true --DumpTriggerLevel=0");
    manager::options::get().lock().unwrap();
    manager::options::destroy().unwrap();
    println!("Hello, world!");
}
