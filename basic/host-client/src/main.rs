use rand::Rng;
use rumqttc::{Client, MqttOptions, Packet, QoS};
use std::{error::Error, thread, time::Duration};

#[derive(Debug)]
#[toml_cfg::toml_config]
pub struct Config {
    #[default("localhost")]
    mqtt_host: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let hello_topic = "hello-topic";
    let test_topic = "test-topic";
    let rand_topic = "rand-topic";
    let mut mqttoptions = MqttOptions::new("2332", CONFIG.mqtt_host, 1883);
    mqttoptions.set_credentials(CONFIG.mqtt_user, CONFIG.mqtt_pass);

    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    client.subscribe(test_topic, QoS::AtMostOnce)?;
    client.subscribe(hello_topic, QoS::AtMostOnce)?;
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        loop {
            let r: i32 = rng.gen();
            println!("Generating rand {:?}", r);
            client
                .publish(rand_topic, QoS::AtLeastOnce, false, r.to_be_bytes())
                .unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Iterate to poll the eventloop for connection progress
    for (_, notification) in connection.iter().enumerate() {
        // if you want to see *everything*, uncomment:
        // println!("Notification = {:#?}", notification);

        if let Ok(rumqttc::Event::Incoming(Packet::Publish(publish_data))) = notification {
            if publish_data.topic == hello_topic {
                println!("Board says hi!");
            }

            if publish_data.topic == test_topic {
                let data: &[u8] = &publish_data.payload;
                let data: Result<[u8; 4], _> = data.try_into();

                if let Ok(data) = data {
                    let temp: f32 = f32::from_be_bytes(data);
                    println!("Board temperature: {:.2}°C", temp)
                }
            }
        }
    }
    Ok(())
}
