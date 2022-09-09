use librumqttd::{async_locallink::construct_broker, Config};
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

    let (mut router, console, servers, builder) = construct_broker(config);

    thread::spawn(move || {
        router.start().unwrap();
    });

    let mut rt = tokio::runtime::Builder::new_multi_thread();
    rt.enable_all();
    rt.build().unwrap().block_on(async {
        let (mut tx, mut rx) = builder.connect("localclient", 200).await.unwrap();
        tx.subscribe(std::iter::once("nth_odd/+")).await.unwrap();

        let console_task = tokio::spawn(console);

        let sub_task = tokio::spawn(async move {
            loop {
                let message = rx.recv().await.unwrap();
                let mut num = [0u8; 4];
                num.clone_from_slice(&message.payload[0][..4]);
                let num = u32::from_be_bytes(num);
                let nth_odd = find_nth_odd(num);
                let topic = format!("{}/response", message.topic);
                tx.publish(topic, true, nth_odd.to_be_bytes())
                    .await
                    .unwrap();
            }
        });

        servers.await;
        sub_task.await.unwrap();
        console_task.await.unwrap();
    });
}
