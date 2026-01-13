use std::{thread, time::Duration, process};
use tokio_modbus::prelude::*;

extern crate paho_mqtt as mqtt;

const TTY_PATH: &str = "/dev/ttySOLAR";
const BAUD_RATE: u32 = 9600;

const MQTT_IP: &str = "tcp://127.0.0.1:1883";
const MQTT_CLIENT_ID: &str = "energy_monitor";

fn main() {
    let mqtt_client = open_mqtt_connection();

    loop {
        read_sensor(&mqtt_client);
        thread::sleep(Duration::from_millis(1000));
    }
}

fn read_sensor(mqtt_client: &mqtt::Client) -> Result<(), Box<dyn std::error::Error>> {
    let slave = Slave(0x01);

    let builder = tokio_serial::new(TTY_PATH, BAUD_RATE);

    let mut ctx = sync::rtu::connect_slave(&builder, slave)?;
    
    let rsp = ctx.read_input_registers(0x00, 6)?;
    match rsp {
        Ok(data) => {
            let voltage = (data[0] as f32) / 100.0;

            let mut current = (data[1] as f32) / 100.0;
            if current <= 0.01 {
                current = 0 as f32
            }

            let mut power = (bytes_to_word_unsigned(
                data[2] as u8,
                data[3] as u8
            ) as f32) / 10.0;
            if power < 1.0 {
                power = 0 as f32;
            }

            let energy = bytes_to_word_unsigned(
                data[4] as u8,
                data[5] as u8
            );

            println!("{voltage}V, {current}A, {power}W, {energy}Wh");
            let payload = format!("solar,panel=0 voltage={voltage},charge_current={current},charge_power={power},charge_energy={energy}");
            //println!("{payload}");
            write_mqtt_message(mqtt_client, "mcu", payload.as_str());
            write_mqtt_message(mqtt_client, "live/solar/power", format!("{power}").as_str()); //live data for dashboard

        },
        Err(e) => {
            println!("{e:?}");
        },
    }

    Ok(())
}

fn write_mqtt_message(mqtt_client: &mqtt::Client, topic: &str, payload: &str) {
    if !mqtt_client.is_connected() {
        println!("Lost connection to mqtt broker");
        match mqtt_client.reconnect() {
            Ok(_) => println!("Reconnected to mqtt broker"),
            Err(e) => println!("{:?}", e),
        }
    }

    let msg = mqtt::Message::new(topic, payload.clone(), 1);
    let tok = mqtt_client.publish(msg);
    //println!("{payload}");

    if let Err(e) = tok {
        println!("Error sending message: {:?}", e);
    }
}

fn open_mqtt_connection() -> mqtt::Client {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_IP)
        .client_id(MQTT_CLIENT_ID.to_string())
        .finalize();

    // Create a client.
    let mqtt_client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

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

    return mqtt_client;
}


fn bytes_to_word_unsigned(a: u8, b: u8) -> u16 {
    let a: u16 = a.into(); // cast byte into 16 bit signed integer
    let b: u16 = b.into(); // cast byte into 16 bit signed integer

    let c: u16 = (b << 8) | a; // combine bytes into word

    return c; // return word
}