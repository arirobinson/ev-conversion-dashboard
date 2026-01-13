use mqtt::Client;
use serialport::SerialPort;
use std::time::Duration;
use std::process;
use nmea_parser::*;
use std::str;

extern crate paho_mqtt as mqtt;

const MSG_LEN: usize = 82; // message size
const GPS_PATH: &str = "/dev/ttyGPS"; // path to usb gps
const GPS_BAUD_RATE: u32 = 9600; // usb gps baud rate
const MQTT_IP: &str = "tcp://127.0.0.1:1883";
const MQTT_CLIENT_ID: &str = "gps";

fn main() {
    let mut reconnect = false;
    let mut gps_port = connect_gps();

    let mut serial_buffer = [0;1]; // buffer to store each byte as it's received
    let mut message_buffer = [0;MSG_LEN]; // buffer to store a full message of 13 bytes

    let mut parser = NmeaParser::new(); // nmea 0183 parser
    let mqtt_client = open_mqtt_connection();

    let mut i = 0; // buffer index
    loop{
        if reconnect {
            gps_port = connect_gps();
            reconnect = false;
        }
        match gps_port.read(&mut serial_buffer){ // read new byte from serial port
                Ok(_) => {
                    let mut increment = false; // reset increment boolean to false

                    // check if buffer is full
                    if i == MSG_LEN + 1{
                        i = 0; // reset back to first index
                        message_buffer = [0;MSG_LEN]; // reset buffer to zeros
                    }
                    
                    // if first byte in message
                    if i == 0 {
                        // check if byte is $
                        if serial_buffer[0] == 0x24 {
                            message_buffer[i] = serial_buffer[0]; // add current byte to buffer
                            increment = true; // allow loop to incrememnt index
                        }
                    }
                    else {
                        message_buffer[i] = serial_buffer[0]; // add current byte to buffer
                        increment = true; // allow loop to incrememnt index
                    }

                    if i > 0 && message_buffer[i-1] == 0x0d && message_buffer[i] == 0x0a {
                        let sentence = str::from_utf8(&message_buffer).unwrap();
                        //println!("{sentence}");

                        if let Ok(sentence) = parser.parse_sentence(sentence) {
                            match sentence {
                                ///// RMC /////
                                ParsedMessage::Rmc(rmc) => {
                                    if (rmc.latitude.is_some() && rmc.longitude.is_some()) || rmc.sog_knots.is_some() || rmc.bearing.is_some() {
                                        let mut payload: String = "gps,device=gps ".to_string();

                                        if rmc.latitude.is_some() && rmc.longitude.is_some(){
                                            let lat = rmc.latitude.unwrap();
                                            let lon = rmc.longitude.unwrap();

                                            payload.push_str(format!("latitude={lat:.8},longitude={lon:.8}").as_str());
                                            write_mqtt_message(&mqtt_client, "live/gps/position", format!("{lat:.8},{lon:.8}").as_str()); //live data for dashboard
                                        }

                                        if rmc.sog_knots.is_some() {
                                            let speed = rmc.sog_knots.unwrap() * 1.852;

                                            payload.push_str(format!(",speed={speed:.1}").as_str());
                                            write_mqtt_message(&mqtt_client, "live/gps/speed", format!("{speed:.1}").as_str()); //live data for dashboard
                                        }

                                        if rmc.bearing.is_some() {
                                            let bearing = rmc.bearing.unwrap();

                                            payload.push_str(format!(",bearing={bearing:.1}").as_str());
                                            write_mqtt_message(&mqtt_client, "live/gps/bearing", format!("{bearing:.1}").as_str()); //live data for dashboard
                                        }

                                        write_mqtt_message(&mqtt_client, "gps", payload.as_str().clone()); //influxdb protocol
                                    }
                                },

                                ///// GGA /////
                                ParsedMessage::Gga(gga) => {
                                    if gga.satellite_count.is_some() || gga.hdop.is_some() || gga.altitude.is_some() {
                                        let mut payload: String = "gps,device=gps ".to_string();

                                        if gga.altitude.is_some() {
                                            let altitude = gga.altitude.unwrap();

                                            payload.push_str(format!("altitude={altitude:.1}").as_str());
                                            write_mqtt_message(&mqtt_client, "live/gps/altitude", format!("{altitude:.1}").as_str()); //live data for dashboard
                                        }

                                        if gga.satellite_count.is_some() {
                                            let satellite_count = gga.satellite_count.unwrap();

                                            payload.push_str(format!(",satellite_count={satellite_count}").as_str());
                                        }

                                        /*if gga.hdop.is_some() {
                                            let hdop = gga.hdop.unwrap();

                                            payload.push_str(format!(",hdop={hdop:.1}").as_str());
                                        }*/

                                        write_mqtt_message(&mqtt_client, "gps", payload.as_str().clone()); //influxdb protocol
                                    }
                                },

                                ///// GSA /////
                                ParsedMessage::Gsa(gsa) => {
                                    if (gsa.mode1_automatic.is_some()) || gsa.mode2_3d.is_some() || gsa.pdop.is_some() || gsa.hdop.is_some() || gsa.vdop.is_some() {
                                        let mut payload: String = "gps,device=gps ".to_string();

                                        if gsa.mode1_automatic.is_some() {
                                            let mode1_automatic = gsa.mode1_automatic.unwrap();

                                            payload.push_str(format!("mode1_automatic={}", mode1_automatic as i32).as_str());
                                        }

                                        if gsa.mode2_3d.is_some() {
                                            let mode2_3d = gsa.mode2_3d.unwrap();

                                            payload.push_str(format!(",mode2_3d=\"{}\"", mode2_3d.to_string().replace(" ", "_")).as_str());
                                        }

                                        if gsa.pdop.is_some() {
                                            let pdop = gsa.pdop.unwrap();

                                            payload.push_str(format!(",pdop={pdop:.1}").as_str());
                                        }

                                        if gsa.hdop.is_some() {
                                            let hdop = gsa.hdop.unwrap();

                                            payload.push_str(format!(",hdop={hdop:.1}").as_str());
                                        }

                                        if gsa.vdop.is_some() {
                                            let vdop = gsa.vdop.unwrap();

                                            payload.push_str(format!(",vdop={vdop:.1}").as_str());
                                        }

                                        write_mqtt_message(&mqtt_client, "gps", payload.as_str().clone()); //influxdb protocol
                                    }
                                },
                                
                                ///// GLL /////
                                ParsedMessage::Gll(gll) => {
                                    if gll.data_valid.is_some() {
                                        let mut payload: String = "gps,device=gps ".to_string();

                                        if gll.data_valid.is_some() {
                                            let data_valid = gll.data_valid.unwrap();

                                            payload.push_str(format!("data_valid={}", data_valid as i32).as_str());
                                        }

                                        write_mqtt_message(&mqtt_client, "gps", payload.as_str().clone()); //influxdb protocol
                                    }
                                },
                                _ => {
                                }
                            }
                        }
                        
                        message_buffer = [0;MSG_LEN];
                        i = 0;

                        increment = false;
                    }

                    if increment {
                        i += 1;
                    }
                },
                Err(e) => {
                    eprintln!("Read Error:{:?}", e);
                    reconnect = true;
                }
        }
    }
}

fn write_mqtt_message(mqtt_client: &Client, topic: &str, payload: &str){
    let msg = mqtt::Message::new(topic, payload.clone(), 1); //build message
    if !mqtt_client.is_connected(){ //check if connected to broker
        println!("Lost connection to mqtt broker");
        match mqtt_client.reconnect(){ //reconnext to broker
            Ok(_) => println!("Reconnected to mqtt broker"),
            Err(e) => println!("{:?}", e),
        }
    }
    let tok = mqtt_client.publish(msg); // publish message over mqtt
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

fn connect_gps() -> Box<dyn SerialPort>{
    let gps_port = serialport::new(GPS_PATH, GPS_BAUD_RATE)
        .timeout(Duration::from_millis(10000))
        .open()
        .expect("Failed to open serial port {GPS_PATH}");
    return gps_port;
}