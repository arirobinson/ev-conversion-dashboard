use mqtt::{message, Client};
use std::time::{Duration, Instant};
use std::{process, thread};

use embedded_can::{Frame as EmbeddedFrame, StandardId};
use socketcan::{CanFrame, CanSocket, ExtendedId, Frame, NonBlockingCan, Socket};
use std::env;

extern crate paho_mqtt as mqtt;

const MSG_LEN: usize = 13; // message size
const MQTT_IP: &str = "tcp://127.0.0.1:1883";
const MQTT_CLIENT_ID: &str = "can-mcu";
const CAN_INTERFACE: &str = "can0";
const CELLS_PER_GROUP: u8 = 10;

const EID_REQUEST_READ: u32 = 0x14ebd0d8; // message id for read requests

// PGNs to request at a slow frequency
const REQUEST_RATE_SLOW: Duration = Duration::from_millis(1000); // rate to make requests
const PGN_SLOW: [[u8; 4]; 6] = 
[
    [0x20, 0xFF, 0x00, 0x00], // PGN_MCUSUM
    [0x23, 0xFF, 0x00, 0x00], // PGN_THSUM
    [0x24, 0xFF, 0x00, 0x00], // PGN_SOCSUM
    [0xA0, 0xFF, 0x00, 0x00], // PGN_CELLG1_CV
    [0xA1, 0xFF, 0x00, 0x00], // PGN_CELLG2_CV
    [0xC0, 0xFF, 0x00, 0x00]  // PGN_CELLG1_TH
    ];
    
// PGNs to request at a high frequency
const REQUEST_RATE_FAST: Duration = Duration::from_millis(100); // rate to make requests
const PGN_FAST: [[u8; 4]; 2] = 
    [
        [0x21, 0xFF, 0x00, 0x00], // PGN_PACKSUM
        [0x22, 0xFF, 0x00, 0x00], // PGN_CVSUM
    ];

fn main() {
    let mqtt_client = open_mqtt_connection();

    let iface = env::args().nth(1).unwrap_or_else(|| CAN_INTERFACE.into());

    let mut sock: CanSocket = CanSocket::open(&iface).expect("Failed to open socket");

    // create new thread for sending requests
    thread::spawn(|| {
        let mut last: Instant = Instant::now() - REQUEST_RATE_SLOW;
        let mut now: Instant;

        let iface = env::args().nth(1).unwrap_or_else(|| CAN_INTERFACE.into());
        let mut sock: CanSocket = CanSocket::open(&iface).expect("Failed to open socket");

        loop {
            for pgn in PGN_FAST {
                let frame = CanFrame::new(ExtendedId::new(EID_REQUEST_READ).unwrap(), &pgn).expect("Failed to create frame");
                sock.transmit(&frame).expect("Failed to transmit frame");
                
                thread::sleep(Duration::from_millis(5));
            }
            
            now = Instant::now();

            if now.duration_since(last) > REQUEST_RATE_SLOW {
                for pgn in PGN_SLOW {
                    let frame = CanFrame::new(ExtendedId::new(EID_REQUEST_READ).unwrap(), &pgn).expect("Failed to create frame");
                    sock.transmit(&frame).expect("Failed to transmit frame");
                    
                    thread::sleep(Duration::from_millis(5));
                }

                last = now;
            }

            thread::sleep(REQUEST_RATE_FAST);
        }
    });

    loop {
        match sock.receive() {
            Ok(f) => {
                decode_message(&mqtt_client, f)
            }
            Err(e) => {
                //eprintln!("Receive Error: {:?}", e);
            }
        }
    }

    //close_mqtt_connection(mqtt_client);
}

fn decode_message(mqtt_client: &Client, frame: CanFrame) {
    let message_id = frame.raw_id(); // combine bytes of message id into a word
    let message = frame.data();

    match message_id {
        // switch case for message id's
        0x14ffa0d0 | 0x14ffa1d0 => {
            // PGN_CELLG1_CV
            let group_num = match message_id {
                0x14ffa0d0 => 0,
                0x14ffa1d0 => 1,
                _ => 0,
            };
            let group_index = message[0];
            let base_cell_num = (group_num * CELLS_PER_GROUP) + (group_index * 3) + 1;

            let mut i = 2;
            let mut values: String = format!("power,system=pack "); //influxdb line protocol
            for cell_number in base_cell_num..(base_cell_num + 3) {
                if (cell_number - (group_num * CELLS_PER_GROUP)) > CELLS_PER_GROUP {
                    break;
                };

                let w_cv: f32 = (bytes_to_word_unsigned(message[i], message[i + 1]) as f32) / 10000.0;

                values = format!("{values}cv_{cell_number:02}={w_cv},");

                i = i + 2;
            }
            if i > 2 {
                values.pop();
                //println!("{values}");
                write_mqtt_message(mqtt_client, "mcu", values.as_str());
            }
        }
        0x14ff20d0 => {
            // PGN_MCUSUM
            let charge_kwh: f32 = (bytes_to_word_unsigned(message[2], message[3]) as f32) / 100.0;
            let charge_state = match message[4] {
                0 => "Standby",
                1 => "Startup",
                2 => "Warmdown",
                10 => "Bulk",
                11 => "Finish",
                12 => "Float",
                13 => "Top Balance",
                _ => "N/A"
            };
            let charge_plug_state = match message[5]{
                0 => "Unknown",
                1 => "Disconnected",
                2 => "Connected",
                3 => "Locked",
                4 => "Waiting For Disc",
                5 => "Active",
                _ => "N/A"
            };
            let bms_alerts: u16 = bytes_to_word_unsigned(message[6], message[7]);
            let w_alerts: [BitField; 9] = [
                BitField { name: "BMS_FAULT_ILLEGAL_CONF".to_string(), mask: 0x0040 },
                BitField { name: "BMS_FAULT_NOT_LOCKED".to_string(), mask: 0x0080 },
                BitField { name: "BMS_FAULT_TH_UNDERTEMP".to_string(), mask: 0x0100 },
                BitField { name: "BMS_FAULT_TH_OVERTEMP".to_string(), mask: 0x0200 },
                BitField { name: "BMS_FAULT_CELL_LVC".to_string(), mask: 0x0400 },
                BitField { name: "BMS_FAULT_CELL_HVC".to_string(), mask: 0x0800 },
                BitField { name: "BMS_FAULT_THERM_CENSUS".to_string(), mask: 0x1000 },
                BitField { name: "BMS_FAULT_CELL_CENSUS".to_string(), mask: 0x2000 },
                BitField { name: "BMS_FAULT_HARDWARE".to_string(), mask: 0x4000 },
            ];

            let mut bms_alerts_message = "power,system=bms ".to_string();
            for alert in w_alerts { //loop through all alerts in list
                let value = (bms_alerts & alert.mask == alert.mask) as u8; //check if alert bit is 1
                bms_alerts_message = String::new() + &bms_alerts_message.to_string() + &alert.name.to_string() + "=" + &value.to_string() + ","; //add value to message string
            }
            bms_alerts_message.pop(); //remove trailing comma
            //println!("{bms_alerts_message}");
            write_mqtt_message(mqtt_client, "mcu", bms_alerts_message.as_str());

            //println!("Charge kWh: {charge_kwh} kWh");
            //println!("Charge State: {charge_state}");
            //println!("Charge Plug State: {charge_plug_state}");
            //println!("BMS Alerts: {bms_alerts:x}");

            let payload = format!("power,system=mcu charge_kwh={charge_kwh},charge_state=\"{charge_state}\",charge_plug_state=\"{charge_plug_state}\"");
            write_mqtt_message(mqtt_client, "mcu", payload.as_str());
            write_mqtt_message(mqtt_client, "live/mcu/charge_kwh", format!("{charge_kwh}").as_str()); //live data for dashboard
            write_mqtt_message(mqtt_client, "live/mcu/charge_state", format!("{charge_state}").as_str()); //live data for dashboard
            write_mqtt_message(mqtt_client, "live/mcu/charge_plug_state", format!("{charge_plug_state}").as_str()); //live data for dashboard
        
        }
        0x14ff21d0 => {
            // PGN_PACKSUM
            let pack_voltage: f32 = (bytes_to_word_unsigned(message[2], message[3]) as f32) / 10.0; // extract and convert pack voltage
            let pack_current: f32 = (bytes_to_word_signed(message[4], message[5]) as f32) / 10.0; // extract and convert pack current
            //println!("Pack Voltage: {pack_voltage} V");
            //println!("Pack Current: {pack_current} A");

            let payload = format!("power,system=pack pack_voltage={pack_voltage},pack_current={pack_current}");
            write_mqtt_message(mqtt_client, "mcu", payload.as_str());
            write_mqtt_message(mqtt_client, "live/mcu/pack_current", format!("{pack_current}").as_str()); //live data for dashboard
        }
        0x14ff22d0 => {
            // PGN_CVSUM
            let cell_voltage_low: f32 = (bytes_to_word_unsigned(message[2], message[3]) as f32) / 10000.0; // extract and convert lowest cell voltage
            let cell_voltage_mean: f32 = (bytes_to_word_unsigned(message[4], message[5]) as f32) / 10000.0; // extract and convert mean cell voltage
            let cell_voltage_high: f32 = (bytes_to_word_unsigned(message[6], message[7]) as f32) / 10000.0; // extract and convert highest cell voltage
            //println!("Cell Voltage Low: {cell_voltage_low} V");
            //println!("Cell Voltage Mean: {cell_voltage_mean} V");
            //println!("Cell Voltage High: {cell_voltage_high} V");

            let payload = format!("power,system=cells cell_voltage_low={cell_voltage_low},cell_voltage_mean={cell_voltage_mean},cell_voltage_high={cell_voltage_high}");
            write_mqtt_message(mqtt_client, "mcu", payload.as_str().clone());
            write_mqtt_message(mqtt_client, "live/mcu/cell_voltage_mean", format!("{cell_voltage_mean}").as_str()); //live data for dashboard        }
        }
        0x14ff23d0 => {
            //PGN_THSUM
            let thermistor_count: u8 = message[1]; // extract thermistor count from message
            let thermistor_temp_low: i8 = message[2] as i8; // extract lowest thermistor temperature from message
            let thermistor_temp_high: i8 = message[3] as i8; // extract highest thermistor temperature from message
            let thermistor_temp_low_alarm: i8 = message[6] as i8; // extract thermistor low temperature alarm (configured) from message
            let thermistor_temp_high_alarm: i8 = message[7] as i8; // extract thermistor high temperature alarm (configured) from message
            //println!("Thermistor Count: {thermistor_count}");
            //println!("Thermistor Temp Low: {thermistor_temp_low}°C");
            //println!("Thermistor Temp High: {thermistor_temp_high}°C");
            //println!("Thermistor Temp Low Alarm (configured): {thermistor_temp_low_alarm}°C");
            //println!("Thermistor Temp High Alarm (configured): {thermistor_temp_high_alarm}°C");

            let payload = format!("power,system=pack thermistor_count={thermistor_count},thermistor_temp_low={thermistor_temp_low},thermistor_temp_high={thermistor_temp_high},thermistor_temp_low_alarm={thermistor_temp_low_alarm},thermistor_temp_high_alarm={thermistor_temp_high_alarm}");
            write_mqtt_message(mqtt_client, "mcu", payload.as_str().clone());
            write_mqtt_message(mqtt_client, "live/mcu/pack_temp_low", format!("{thermistor_temp_low}").as_str()); //live data for dashboard
            write_mqtt_message(mqtt_client, "live/mcu/pack_temp_high", format!("{thermistor_temp_high}").as_str()); //live data for dashboard
        }
        0x14ff24d0 => {
            // PGN_SOCSUM
            let soc: u8 = message[1]; // extract soc from message
            let pack_kwh_current: f32 = (bytes_to_word_unsigned(message[2], message[3]) as f32) / 10.0; // extract and convert remaining pack capacity
            let pack_kwh_max: f32 = (bytes_to_word_unsigned(message[4], message[5]) as f32) / 10.0; // extract and convert total pack capacity
            //println!("SOC: {soc}%");
            //println!("Pack Capacity: {pack_kwh_current} kWh / {pack_kwh_max} kWh");

            let payload = format!("power,system=pack soc={soc},pack_kwh_current={pack_kwh_current},pack_kwh_max={pack_kwh_max}");
            write_mqtt_message(mqtt_client, "mcu", payload.as_str().clone());
            write_mqtt_message(mqtt_client, "live/mcu/soc", format!("{soc}").as_str()); //live data for dashboard
            write_mqtt_message(mqtt_client, "live/mcu/pack_kwh_current", format!("{pack_kwh_current}").as_str()); //live data for dashboard
            write_mqtt_message(mqtt_client, "live/mcu/pack_kwh_max", format!("{pack_kwh_max}").as_str()); //live data for dashboard
        }
        0x14ffc0d0 => {
            if message[0] == 0x00 {
                let thermistor_04: i8 = message[6] as i8; // extract thermistor 04 temperature from message
                let thermistor_05: i8 = message[7] as i8; // extract thermistor 05 temperature from message

                //println!("Thermistor 04: {thermistor_04}°C");
                //println!("Thermistor 05: {thermistor_05}°C");

                let payload = format!("power,system=pack th_04={thermistor_04},th_05={thermistor_05}");
                write_mqtt_message(mqtt_client, "mcu", payload.as_str());
                write_mqtt_message(mqtt_client, "live/mcu/pack_th_04", format!("{thermistor_04}").as_str()); //live data for dashboard
                write_mqtt_message(mqtt_client, "live/mcu/pack_th_05", format!("{thermistor_05}").as_str()); //live data for dashboard
            }
        }
        _ => {
            //println!("Unmatched: {message_id:x} {message:x?}");
        }
    }
}

fn write_mqtt_message(mqtt_client: &Client, topic: &str, payload: &str) {
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

/*fn close_mqtt_connection(mqtt_client: Client){
    let tok = mqtt_client.disconnect(None);
    println!("Disconnect from the broker");
    tok.unwrap();
}*/

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

struct BitField {
    name: String,
    mask: u16,
}