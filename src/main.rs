extern crate openzwave;
use openzwave::{options, manager};

fn main() {
    options::create("./config/", "", "--SaveConfiguration=true --DumpTriggerLevel=0").unwrap();
    options::get().unwrap().lock().unwrap();
    manager::create().unwrap();
    let mut watcher = manager::Watcher::new(
        |notification: manager::Notification| println!("{}", notification.a)
    );

    manager::get().unwrap().add_watcher(&mut watcher);
    manager::get().unwrap().remove_watcher(&mut watcher);
    manager::destroy();
    options::destroy().unwrap();
    println!("Hello, world!");
}
