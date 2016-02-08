extern crate openzwave;
use openzwave::options;

fn main() {
    options::create("./config/", "", "--SaveConfiguration=true --DumpTriggerLevel=0");
    options::get().lock().unwrap();
    options::destroy().unwrap();
    println!("Hello, world!");
}
