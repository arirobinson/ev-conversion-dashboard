# EV Conversion Dashboard
Modular microservices and web front end of electric vehicle conversions.

## Services
The EV conversion dashboard includes a set of microservices written in Rust to ingest data from the following devices and publish over MQTT for live viewing as well as storing in InfluxDB using Telegraf:
| Type | Device | Comms |
|------|--------|-------|
| BMS | [Thunderstruck MCU](https://www.thunderstruck-ev.com/mcu.html) | CAN |
| Motor Controller | [Alltrax SR](https://alltraxinc.com/product-category/sr-controllers/) | USB |
| GPS | [Generic NMEA0183](https://www.amazon.ca/dp/B078Y52FGQ) | USB |
| Energy Monitor | [PZEM-003](https://www.aliexpress.com/item/1005004321343868.html) | USB-RS485 |
| Screen | [HDMI Touchscreen](https://www.waveshare.com/10.4hp-capqled.htm) | USB ([DDCUTIL](https://www.ddcutil.com/)) |

## Software Layout Diagram
![Diagram](images/ev-dashboard-software-stack.svg)
