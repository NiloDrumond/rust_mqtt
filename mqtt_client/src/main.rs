use rumqttc::{self, Client, Event::Incoming, LastWill, MqttOptions, Packet::Publish, QoS};
use std::time::{Duration, Instant};
use std::{env, process};

fn main() {
    pretty_env_logger::init();

    let mut args = env::args();
    args.next();

    let index: usize = match args.next() {
        Some(arg) => match arg.parse() {
            Ok(num) => num,
            Err(e) => {
                eprintln!("Index inválido: {}", e);
                process::exit(1);
            }
        },
        None => {
            eprintln!("Index não recebido");
            process::exit(1);
        }
    };

    let mut mqttoptions = MqttOptions::new("rust_mqtt", "localhost", 1883);
    let will = LastWill::new("hello/world", "good bye", QoS::AtMostOnce, false);
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_last_will(will);

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    let sub_topic = format!("nth_odd/{}/response", index);
    client.subscribe(sub_topic, QoS::AtMostOnce).unwrap();

    let topic = format!("nth_odd/{}", index);
    // thread::spawn(move || run_test(client, topic));
    let num: u32 = 10_000_000;
    let mut durations = [0; 10];

    let mut i = 0;

    while i < durations.len() {
        let now = Instant::now();
        client
            .publish(&topic, QoS::AtLeastOnce, true, num.to_be_bytes())
            .unwrap();
        loop {
            let payload = connection.iter().next();
            if let Some(Ok(Incoming(Publish(event)))) = payload {
                let mut nth_odd = [0u8; 4];
                nth_odd.clone_from_slice(&event.payload[..4]);
                let _nth_odd = u32::from_be_bytes(nth_odd);
                break;
            }
        }
        durations[i] = now.elapsed().as_millis() as i32;
        i += 1;
    }

    let mean_duration: i32 = durations.iter().sum::<i32>() / durations.len() as i32;
    let variance: f32 = durations
        .iter()
        .map(|value| {
            let diff: i32 = mean_duration - value;
            diff * diff
        })
        .sum::<i32>() as f32
        / durations.len() as f32;
    let std_deviation = variance.sqrt();

    println!("Average RTT: {}", mean_duration);
    println!("Standard Deviation: {}", std_deviation);

    client.disconnect().unwrap();
}
