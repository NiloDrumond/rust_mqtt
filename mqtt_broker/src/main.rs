use librumqttd::{Broker, Config};
use std::thread;

fn find_nth_odd(n: u32) -> u32 {
    let mut i = 0;
    let mut odd_count = 0;

    while odd_count != n {
        i += 1;
        if i % 2 == 1 {
            odd_count += 1;
        }
    }

    i
}

fn main() {
    pretty_env_logger::init();
    let config: Config = confy::load_path("config/rumqttd.conf").unwrap();
    let mut broker = Broker::new(config);

    let mut tx = broker.link("localclient").unwrap();

    thread::spawn(move || {
        broker.start().unwrap();
    });

    let mut rx = tx.connect(200).unwrap();
    tx.subscribe("nth_odd/+").unwrap();

    loop {
        if let Some(message) = rx.recv().unwrap() {
            let mut num = [0u8; 4];
            num.clone_from_slice(&message.payload[0][..4]);
            let num = u32::from_be_bytes(num);
            let nth_odd = find_nth_odd(num);
            let topic = format!("{}/response", message.topic);
            tx.publish(topic, true, nth_odd.to_be_bytes()).unwrap();
            println!("num {}", num);
        }
    }
}
