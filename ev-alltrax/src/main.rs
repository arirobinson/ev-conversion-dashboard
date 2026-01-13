use mqtt::Client;
use std::time::Duration;
use std::{process, thread};
use std::str;

extern crate paho_mqtt as mqtt;
extern crate hidapi;

const MQTT_IP: &str = "tcp://127.0.0.1:1883";
const MQTT_CLIENT_ID: &str = "alltrax";
const MSG_REQUEST: [u8; 64] = [0x01, 0x2E, 0xA0, 0x00, 0x20, 0x00, 0x41, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const VID: u16 = 0x23D4;
const PID: u16 = 0x0001;

fn main() {
    let mqtt_client = open_mqtt_connection();

    let api = hidapi::HidApi::new().unwrap();

    // Connect to device using its VID and PID
    let device = api.open(VID, PID).unwrap();
    let mut buf = [0u8; 64];

    loop{
        let res = device.write(&MSG_REQUEST).unwrap(); //write request message
        thread::sleep(Duration::from_millis(50)); //wait 50ms

        let res = device.read(&mut buf[..]).unwrap(); //read from device
        //println!("Read: {:?}", &buf[..res]);

        let message_id = buf[1];
        let checksum = bytes_to_word_unsigned(buf[3], buf[2]);

        if message_id == 0x2E{
            let battery_voltage = (bytes_to_word_unsigned(buf[8], buf[9]) as f32) / 10.0;
            let throttle_pointer = bytes_to_word_unsigned(buf[14], buf[15]);
            let throttle_position = bytes_to_word_unsigned(buf[16], buf[17]);
            let motor_current = (bytes_to_word_signed(buf[18], buf[19]) as f32) / 10.0;
            let overtemp_cap = bytes_to_word_signed(buf[46], buf[47]);

            let uk_6_7 = bytes_to_word_signed(buf[6], buf[7]);
            let uk_10_11 = (bytes_to_word_signed(buf[10], buf[11]) as f32) / 10.0;
            let uk_12_13 = bytes_to_word_signed(buf[12], buf[13]);
            let uk_20_21 = bytes_to_word_signed(buf[20], buf[21]);
            let uk_22_23 = bytes_to_word_signed(buf[22], buf[23]);
            let uk_24_25 = bytes_to_word_signed(buf[24], buf[25]);
            let uk_26_27 = bytes_to_word_signed(buf[26], buf[27]);
            let uk_28_29 = bytes_to_word_signed(buf[28], buf[29]);
            let uk_30_31 = bytes_to_word_signed(buf[30], buf[31]);
            let uk_32_33 = bytes_to_word_signed(buf[32], buf[33]);
            let uk_34_35 = bytes_to_word_signed(buf[34], buf[35]);
            let uk_36_37 = bytes_to_word_signed(buf[36], buf[37]);
            let uk_38_39 = bytes_to_word_signed(buf[38], buf[39]);
            let uk_40_41 = bytes_to_word_signed(buf[40], buf[41]);
            let uk_42_43 = bytes_to_word_signed(buf[42], buf[43]);
            let uk_44_45 = bytes_to_word_signed(buf[44], buf[45]);
            
            let uk_48_49 = bytes_to_word_signed(buf[48], buf[49]);
            let uk_50_51 = bytes_to_word_signed(buf[50], buf[51]);
            let uk_52_53 = bytes_to_word_signed(buf[52], buf[53]);
            let uk_54_55 = bytes_to_word_signed(buf[54], buf[55]);
            let uk_56_57 = bytes_to_word_signed(buf[56], buf[57]);
            let uk_58_59 = bytes_to_word_signed(buf[58], buf[59]);
            let uk_60_61 = bytes_to_word_signed(buf[60], buf[61]);
            let uk_62_63 = bytes_to_word_signed(buf[62], buf[63]);

            /*println!("Battery Voltage:\t{battery_voltage}");
            println!("Motor Current:\t\t{motor_current}");
            println!("Throttle Pointer:\t{throttle_pointer}");
            println!("Throttle Position:\t{throttle_position}");*/

            let payload = format!("motor_controller,device=alltrax \
                battery_voltage={battery_voltage:.1},\
                motor_current={motor_current:.1},\
                throttle_pointer={throttle_pointer},\
                throttle_position={throttle_position},\
                overtemp_cap={overtemp_cap},\
                uk_06_07={uk_6_7},\
                uk_10_11={uk_10_11},\
                uk_12_13={uk_12_13},\
                uk_20_21={uk_20_21},\
                uk_22_23={uk_22_23},\
                uk_24_25={uk_24_25},\
                uk_26_27={uk_26_27},\
                uk_28_29={uk_28_29},\
                uk_30_31={uk_30_31},\
                uk_32_33={uk_32_33},\
                uk_34_35={uk_34_35},\
                uk_36_37={uk_36_37},\
                uk_38_39={uk_38_39},\
                uk_40_41={uk_40_41},\
                uk_42_43={uk_42_43},\
                uk_44_45={uk_44_45},\
                uk_48_49={uk_48_49},\
                uk_50_51={uk_50_51},\
                uk_52_53={uk_52_53},\
                uk_54_55={uk_54_55},\
                uk_56_57={uk_56_57},\
                uk_58_59={uk_58_59},\
                uk_60_61={uk_60_61},\
                uk_62_63={uk_62_63}\
                ");

            //let payload = format!("motor_controller,device=alltrax battery_voltage={battery_voltage:.1},motor_current={motor_current:.1},throttle_pointer={throttle_pointer},throttle_position={throttle_position}");
            println!("{payload}");
            write_mqtt_message(&mqtt_client, "motor_controller", payload.as_str());
        }

        thread::sleep(Duration::from_millis(450));
    }
}

fn write_mqtt_message(mqtt_client: &Client, topic: &str, payload: &str){
    let msg = mqtt::Message::new(topic, payload, 1); //build message
    if !mqtt_client.is_connected(){ //check if connected to broker
        println!("Lost connection to mqtt broker");
        match mqtt_client.reconnect(){ //reconnext to broker
            Ok(_) => println!("Reconnected to mqtt broker"),
            Err(e) => println!("{:?}", e),
        }
    }
    let tok = mqtt_client.publish(msg); // publish message over mqtt
    println!("{payload}");

    if let Err(e) = tok {
        println!("Error sending message: {:?}", e);
    }
}

fn open_mqtt_connection() -> Client {
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

fn close_mqtt_connection(mqtt_client: Client){
    let tok = mqtt_client.disconnect(None);
    println!("Disconnect from the broker");
    tok.unwrap();
}

// combine two bytes (u8) into a word (i16)
fn bytes_to_word_signed(a: u8, b: u8) -> i16 {
    let a: i16 = a.into(); // cast byte into 16 bit signed integer
    let b: i16 = b.into(); // cast byte into 16 bit signed integer

    let c: i16 = (b << 8) | a; // combine bytes into word

    return c; // return word
}

fn bytes_to_word_unsigned(a: u8, b: u8) -> u16 {
    let a: u16 = a.into(); // cast byte into 16 bit signed integer
    let b: u16 = b.into(); // cast byte into 16 bit signed integer

    let c: u16 = (b << 8) | a; // combine bytes into word

    return c; // return word
}

