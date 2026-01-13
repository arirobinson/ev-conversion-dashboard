use mqtt::Client;
use std::process;
use std::process::Command;
use std::thread;
use std::time::Duration;

extern crate paho_mqtt as mqtt;

const MQTT_IP: &str = "tcp://127.0.0.1:1883";
const MQTT_CLIENT_ID: &str = "display";
const TOPICS: &[&str] = &["display/#"];
const QOS: &[i32] = &[0];

fn main() {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_IP)
        .client_id(MQTT_CLIENT_ID.to_string())
        .finalize();
    // Create a client.
    let mqtt_client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let rx = mqtt_client.start_consuming();

    let mqtt_client = open_mqtt_connection(&mqtt_client);

    subscribe_topics(&mqtt_client);

    println!("Processing requests...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            let topic = msg.topic();
            let payload = msg.payload_str();

            //println!("{} - {}", topic, payload);

            if topic == "display/control/power" {
                println!("Setting display power to: {}", payload);
                
                let value = if payload.to_string() == "On" {"1"} else if payload.to_string() == "Off" {"5"} else {"5"};

                Command::new("ddcutil")
                    .arg("setvcp")
                    .arg("d6")
                    .arg(value)
                    .spawn()
                    .expect("ddcutil failed");
            }
            if topic == "display/control/brightness" {
                println!("Setting display brightness to: {}", payload);
                
                Command::new("ddcutil")
                    .arg("setvcp")
                    .arg("10")
                    .arg(payload.to_string())
                    .spawn()
                    .expect("ddcutil failed");
            }
        } else if !mqtt_client.is_connected() {
            if try_reconnect(&mqtt_client) {
                println!("Resubscribe topics...");
                subscribe_topics(&mqtt_client);
            } else {
                break;
            }
        }
    }
}

// Reconnect to the broker when connection is lost.
fn try_reconnect(mqtt_client: &mqtt::Client) -> bool {
    println!("Connection lost. Waiting to retry connection");
    for _ in 0..12 {
        thread::sleep(Duration::from_millis(5000));
        if mqtt_client.reconnect().is_ok() {
            println!("Successfully reconnected");
            return true;
        }
    }
    println!("Unable to reconnect after several attempts.");
    false
}

fn subscribe_topics(mqtt_client: &mqtt::Client) {
    if let Err(e) = mqtt_client.subscribe_many(TOPICS, QOS) {
        println!("Error subscribes topics: {:?}", e);
        process::exit(1);
    }
}

/*fn write_mqtt_message(mqtt_client: &Client, topic: &str, payload: &str){
    if !mqtt_client.is_connected(){
        println!("Lost connection to mqtt broker");
        match mqtt_client.reconnect(){
            Ok(_) => println!("Reconnected to mqtt broker"),
            Err(e) => println!("{:?}", e),
        }
    }

    let msg = mqtt::Message::new(topic, payload, 1);
    let tok = mqtt_client.publish(msg);
    //println!("{payload}");

    if let Err(e) = tok {
        println!("Error sending message: {:?}", e);
    }
}*/

fn open_mqtt_connection(mqtt_client: &Client) -> Client {
    /*let create_opts = mqtt::CreateOptionsBuilder::new()
    .server_uri(MQTT_IP)
    .client_id(MQTT_CLIENT_ID.to_string())
    .finalize(); */

    /*// Create a client.
    let mqtt_client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });*/

    // Define the set of options for the connection.
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail.
    if let Err(e) = mqtt_client.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    return mqtt_client.clone();
}
