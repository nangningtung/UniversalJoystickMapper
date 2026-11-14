use gilrs::{Gilrs, Event};
use std::time::Duration;

fn main() {
    let mut gilrs = Gilrs::new().unwrap();
    println!("Listening for 5 seconds... PRESS BUTTONS OR MOVE STICKS NOW!");
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        while let Some(Event { id, event, time, .. }) = gilrs.next_event() {
            println!("At {:?}: Pad {:?} -> {:?}", time, id, event);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
