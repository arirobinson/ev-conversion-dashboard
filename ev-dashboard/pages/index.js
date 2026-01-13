import Head from 'next/head';
import React, { useState, useEffect } from "react";

import '../libraries/mqtt.min.js'
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

import { ThemeProvider, createTheme, styled } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Grid from '@mui/material/Unstable_Grid2';
import CircularProgress from '@mui/material/CircularProgress';
import LinearProgress from '@mui/material/LinearProgress';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import IconButton from '@mui/material/IconButton';
import MenuIcon from '@mui/icons-material/Menu';

import {
  Gauge,
  gaugeClasses,
  GaugeContainer,
  GaugeValueArc,
  GaugeReferenceArc,
  useGaugeState
} from '@mui/x-charts/Gauge';
import { LineChart } from '@mui/x-charts/LineChart';
import { SparkLineChart } from '@mui/x-charts/SparkLineChart';

import { Map, Marker, NavigationControl } from 'react-map-gl';
import "mapbox-gl/dist/mapbox-gl.css";

const defaultLat = 48.4;
const defaultLon = -123.3;
const defaultZoom = 16;

const maxSpeed = 80;
const maxPower = 20;

const mqttUrl = "mqtt://127.0.0.1:9001";

const darkTheme = createTheme({
  palette: {
    mode: 'dark',
  },
});

const lightTheme = createTheme({
  palette: {
    mode: 'light',
  },
});

const Item = styled(Paper)(({ theme }) => ({
  backgroundColor: theme.palette.mode === 'dark' ? '#1A2027' : '#fff',
  ...theme.typography.body2,
  padding: theme.spacing(1),
  textAlign: 'left',
  color: theme.palette.text.secondary,
}));

const mqtt = require("mqtt");
const options = {
  protocol: "ws",
  keepalive: 20,
  reconnectPeriod: 1,
  clientId: "mqttjs_" + Math.random().toString(16).substr(2, 8),
};

let darkMode = true;
let autoCenter = true;
let northUp = false;

const mapStyleDay = "mapbox://styles/mapbox/navigation-day-v1";
const mapStyleNight = "mapbox://styles/mapbox/navigation-night-v1";

export default function Home() {
  const [altitude, setAltitude] = useState(0);
  const [speed, setSpeed] = useState(0);
  const [bearing, setBearing] = useState(0);
  const [position, setPosition] = useState({
    latitude: defaultLat,
    longitude: defaultLon
  })
  const [packVoltage, setPackVoltage] = useState(0);
  //const [packVoltageHistory, setPackVoltageHistory] = useState([]);
  const [packCurrent, setPackCurrent] = useState(0);
  const [packCurrentHistory, setPackCurrentHistory] = useState([]);
  const [lowCellVoltage, setLowCellVoltage] = useState(0);
  const [meanCellVoltage, setMeanCellVoltage] = useState(0);
  const [highCellVoltage, setHighCellVoltage] = useState(0);
  const [soc, setSoc] = useState(0);
  const [packKwhCurrent, setPackKwhCurrent] = useState(0);
  const [packKwhMax, setPackKwhMax] = useState(0);
  const [chargeState, setChargeState] = useState("");
  const [chargePlugState, setChargePlugState] = useState("");
  const [chargeKwh, setChargeKwh] = useState(0);
  const [packLowTemp, setPackLowTemp] = useState(0);
  const [packHighTemp, setPackHighTemp] = useState(0);
  const [throttlePointer, setThrottlePointer] = useState(0);
  const [throttlePosition, setThrottlePosition] = useState(0);
  const [throttleMax, setThrottleMax] = useState(0);

  const [solarPower, setSolarPower] = useState(0);

  const [muiTheme, setMuiTheme] = useState(darkTheme);

  const [mapStyle, setMapStyle] = useState(mapStyleNight);
  const [viewState, setViewState] = useState({
    latitude: defaultLat,
    longitude: defaultLon,
    zoom: defaultZoom
  });

  // https://codesandbox.io/s/horloge-enfants-j92il?file=/src/App.js:2122-2155
  const [date, setDate] = useState(new Date()); //used for clock
  useEffect(() => {
    setInterval(() => setDate(new Date()), 1000);
  }, []);

  const onMove = React.useCallback(({ viewState }) => {
    const newCenter = [viewState.longitude, viewState.latitude];
    setViewState(newCenter);
  }, [])

  function parseMessage(topic, value) {
    switch (topic) {
      case 'live/gps/altitude':
        setAltitude(value);
        break;
      case 'live/gps/speed':
        setSpeed(value);
        break;
      case 'live/gps/bearing':
        setBearing(x => (x*0.5) + (value * 0.5));  
      //setBearing(value);
        break;
      case 'live/gps/position':
        const positionArray = value.split(",");
        setPosition({
          latitude: positionArray[0],
          longitude: positionArray[1]
        });
        if (autoCenter) {
          setViewState({
            latitude: positionArray[0],
            longitude: positionArray[1],
            zoom: defaultZoom
          });
        }
        break;
      case 'live/mcu/charge_kwh':
        setChargeKwh(value);
        break;
      case 'live/mcu/charge_state':
        setChargeState(value);
        break;
      case 'live/mcu/charge_plug_state':
        setChargePlugState(value);
        break;
      case 'live/mcu/pack_current':
        setPackCurrent(value);
        setPackCurrentHistory(packCurrentHistory => [
          ...(packCurrentHistory.slice(Math.max(packCurrentHistory.length - 19, 0))),
          value
        ]);
        break;
      case 'live/mcu/cell_voltage_low':
        setLowCellVoltage(value);
        break;
      case 'live/mcu/cell_voltage_mean':
        setMeanCellVoltage(value);
        setPackVoltage(value * 20);
        break;
      case 'live/mcu/cell_voltage_high':
        setHighCellVoltage(value);
        break;
      case 'live/mcu/soc':
        setSoc(value);
        break;
      case 'live/mcu/pack_kwh_current':
        setPackKwhCurrent(value);
        break;
      case 'live/mcu/pack_kwh_max':
        setPackKwhMax(value);
        break;
      case 'live/mcu/pack_temp_low':
        setPackLowTemp(value);
        break;
      case 'live/mcu/pack_temp_high':
        setPackHighTemp(value);
        break;
      case 'live/solar/power':
        setSolarPower(value);
        break;
      case 'live/motor_controller/throttle_pointer':
        setThrottlePointer(value);
        break;
      case 'live/motor_controller/throttle_position':
        setThrottlePosition(value);
        break;
      case 'live/motor_controller/overtemp_cap':
        setThrottleMax(value);
        break;
    }
  }

  function sendMessage(topic, value) {
    client.publish(topic, value);
    return console.log(topic + " " + value);
  }
  const [client, setClient] = useState(null);
  const [isConnected, setIsConnected] = useState(false);
  const [payload, setPayload] = useState({});

  const getClientId = () => {
    console.log('Set MQTT Broker...');
    return `mqttjs_ + ${Math.random().toString(16).substr(2, 8)}`;
  };

  const mqttConnect = async () => {
    const clientId = getClientId();
    const clientMqtt = await mqtt.connect(mqttUrl, options);
    setClient(clientMqtt);
  };

  const mqttDisconnect = () => {
    if (client) {
      client.end(() => {
        console.log('MQTT Disconnected');
        setIsConnected(false);
      });
    }
  };

  const mqttSubscribe = async (topic) => {
    if (client) {
      console.log('MQTT subscribe ', topic);
      const clientMqtt = await client.subscribe(topic, {
        qos: 0,
        rap: false,
        rh: 0,
      }, (error) => {
        if (error) {
          console.log('MQTT Subscribe to topics error', error);
          return;
        }
      });
      setClient(clientMqtt);
    }
  };

  const mqttUnSubscribe = async (topic) => {
    if (client) {
      const clientMqtt = await client.unsubscribe(topic, (error) => {
        if (error) {
          console.log('MQTT Unsubscribe error', error);
          return;
        }
      });
      setClient(clientMqtt);
    }
  };

  useEffect(() => {
    mqttConnect();
    return () => {
      mqttDisconnect();
    };
  }, []);

  useEffect(() => {
    if (client) {
      client.on('connect', () => {
        setIsConnected(true);
        console.log('MQTT Connected');
      });
      client.on('error', (err) => {
        console.error('MQTT Connection error: ', err);
        client.end();
      });
      client.on('reconnect', () => {
        setIsConnected(true);
      });
      client.on('message', (_topic, message) => {
        parseMessage(_topic, message.toString());
      });
    }
  }, [client]);

  useEffect(() => {
    if (isConnected) {
      mqttSubscribe('live/#');
    }
  }, [isConnected]);

  return (
    <ThemeProvider theme={muiTheme}>
      <CssBaseline />
      <Head>
        <title>EV Dashboard</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <Box sx={{ flexGrow: 1 }}>
        <AppBar position="static" color='transparent' enableColorOnDark>
          <Toolbar variant="regular" position="static">
            <Typography variant="h6">
              {("0" + date.getHours()).slice(-2)}:{("0" + date.getMinutes()).slice(-2)}
            </Typography>

            <Typography variant="h6" sx={{ flexGrow: 1 }} align='center'>
              Sambar
            </Typography>
            <CircularProgressWithLabel value={soc * 1} size={50} />
          </Toolbar>
        </AppBar>
      </Box>

      <Grid container spacing={1} disableEqualOverflow>
        <Grid xs={12} lg={8} paddingLeft={2}>
          <Item>
            <ComponentMap setViewState={setViewState} viewState={viewState} mapStyle={mapStyle} lat={position.latitude} lon={position.longitude} bearing={bearing}></ComponentMap>
            <Button
              variant="outlined"
              onClick={function () {
                autoCenter = !autoCenter;
                console.log("Auto Center: " + autoCenter);
              }}
            >{autoCenter ? "Manual Center" : "Auto Center"}</Button>
            <Button
              variant="outlined"
              onClick={function () {
                darkMode = !darkMode;
                if (darkMode) {
                  setMapStyle(mapStyleNight);
                  setMuiTheme(darkTheme);
                }
                else {
                  setMapStyle(mapStyleDay);
                  setMuiTheme(lightTheme);
                }
                console.log("Dark Mode: " + darkMode);
              }}
            >{darkMode ? "Light Mode" : "Dark Mode"}</Button>
            <Button
              variant="outlined"
              onClick={function () {
                northUp = !northUp;
                console.log("North Up: " + northUp);
              }}
            >{northUp ? "Vehicle Up" : "North Up"}</Button>
            <Button
              variant="outlined"
              onClick={function () {
                sendMessage("display/control/power", "On");
              }}
            >{"Screen On"}</Button>
            <Button
              variant="outlined"
              onClick={function () {
                sendMessage("display/control/power", "Off");
              }}
            >{"Screen Off"}</Button>
            <Button
              variant="outlined"
              onClick={function () {
                sendMessage("display/control/brightness", "1");
              }}
            >{"-"}</Button>
            <Button
              variant="outlined"
              onClick={function () {
                sendMessage("display/control/brightness", "100");
              }}
            >{"+"}</Button>
          </Item>
        </Grid>
        <Grid xs={12} lg={4}>
          <Grid xs={12} paddingRight={1} paddingBottom={0}>
            <Item>
              {/*<LinearProgressWithLabel variant="determinate" size={75} valuelabel={-packVoltage * packCurrent / 1000} valuelabelplaces={1} value={Interpolate(-packVoltage * packCurrent / 1000, 0, 33, 0, 100)} />*/}
              {/*<Typography variant='h5' component="div">Speed: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{speed} km/h</Typography></Typography>*/}
              {/*<Typography variant='h5' component="div">Altitude: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{altitude} m</Typography></Typography>*/}
              <Typography variant='h5' component="div">Voltage: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{Round(packVoltage, 2)} V</Typography></Typography>
              <Typography variant='h5' component="div">Current: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{Round(packCurrent, 1)} A</Typography></Typography>
              <Typography variant='h5' component="div">Throttle: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{throttlePointer} / {throttleMax}</Typography></Typography>
              <br />
              <Typography variant='h5' component="div">Battery Temp: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{packLowTemp}° / {packHighTemp}°</Typography></Typography>
              <br />
              <Typography variant='h5' component="div">Solar Power: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{Round(solarPower, 0)} W</Typography></Typography>
            </Item>
          </Grid>
          <GaugePanel packVoltage={packVoltage} packCurrent={packCurrent} speed={speed}></GaugePanel>
          <ChargingPanel chargeState={chargeState} chargePlugState={chargePlugState} chargeKwh={chargeKwh}></ChargingPanel>
          {/*<CurrentGraphPanel packCurrentHistory={packCurrentHistory}></CurrentGraphPanel>*/}
        </Grid>
      </Grid>
    </ThemeProvider>
  )
}

function Round(number, places) {
  //let roundedNumber = (Math.round(number * (10 * places)) / (10 * places)).toFixed(places);
  let roundedNumber = (number * 1).toFixed(places);
  return roundedNumber;
}

function ComponentMap(props) {
  return (
    <Map
      {...props.viewState}
      onMove={evt => props.setViewState(evt.viewState)}
      mapboxAccessToken=""
      style={{ height: '600px' }}
      mapStyle={props.mapStyle}
      bearing={northUp ? 0 : props.bearing} // if northUp is true, set bearing to 0, else set to props.bearing
      pitch={northUp ? 0 : 50}
    >
      <Marker
        latitude={props.lat}
        longitude={props.lon}
        color="center"
        rotation={northUp ? props.bearing : 0}
        pitchAlignment="map"
      >
        <img src="/vehicle.png" width={northUp ? 65 : 75} />
      </Marker>
      <NavigationControl />
    </Map>
  )
}

function Interpolate(input, inMin, inMax, outMin, outMax) {
  let output = outMin + (input - inMin) * ((outMax - outMin) / (inMax - inMin));
  if (output > outMax) {
    output = outMax;
  }
  return output;
}

function CircularProgressWithLabel(props) {
  let textSize = "caption";
  if (props.size > 25 && props.size <= 50) {
    textSize = "body";
  }
  else if (props.size > 50 && props.size <= 75) {
    textSize = "h6";
  }
  else if (props.size > 75 && props.size <= 100) {
    textSize = "h5";
  }
  else if (props.size > 100 && props.size <= 130) {
    textSize = "h4";
  }
  else if (props.size > 130 && props.size <= 160) {
    textSize = "h3";
  }
  else if (props.size > 160) {
    textSize = "h2";
  }

  return (
    <Box sx={{ position: 'relative', display: 'inline-flex' }}>
      <CircularProgress variant="determinate" {...props} thickness={6} />
      <Box
        sx={{
          top: 0,
          left: 0,
          bottom: 0,
          right: 0,
          position: 'absolute',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <Typography variant={textSize} component="div" color="text.secondary">
          {`${Math.round(props.value)}%`}
        </Typography>
      </Box>
    </Box>
  );
}

function LinearProgressWithLabel(props) {
  return (
    <Box sx={{ display: 'flex', alignItems: 'center' }}>
      <Box sx={{ width: '100%', ml: 1, mr: 2 }}>
        <LinearProgress variant="determinate" {...props} sx={{ height: 10, borderRadius: 5 }} />
      </Box>
      <Box sx={{ minWidth: 80 }}>
        <Typography variant="h6" color="text.secondary">{Round(props.valuelabel, props.valuelabelplaces)} kW</Typography>
      </Box>
    </Box>
  );
}

function ChargingPanel(props) {
  if (props.chargePlugState == "Disconnected") {
    return (
      null
    )
  }
  else {
    return (
      <Grid xs={12} paddingRight={1} paddingLeft={0} paddingBottom={0}>
        <Item>
          <Typography variant='h5' component="div">Charge State: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{props.chargeState}</Typography></Typography>
          <Typography variant='h5' component="div">Charge Plug State: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{props.chargePlugState}</Typography></Typography>
          <Typography variant='h5' component="div">Charged: <Typography variant='h5' display="inline" fontWeight="fontWeightBold">{props.chargeKwh} kWh</Typography></Typography>
        </Item>
      </Grid>
    )
  }
}

function CurrentGraphPanel(props) {
  /*if (props.chargePlugState == "Disconnected") {
    return (
      null
    )
  }*/
  //else {
    if (props.packCurrentHistory.length > 0){
    return (
      <Grid xs={12} paddingRight={1} paddingLeft={0} paddingBottom={0}>
        <Item>
          <Typography variant='h5' component="div">Current</Typography>
          <LineChart
            //xAxis={[{ data: [1, 2, 3, 5, 8, 10] }]}
            series={[
              {
                data: props.packCurrentHistory,
                area: true,
                showMark: false,
                curve: "catmullRom",
              },
            ]}
            width={500}
            height={150}
          />
        </Item>
      </Grid>
    )}
    else{
      return (null)
    }
  //}
}

function GaugePanel(props) {
    return (
      <Grid xs={12} paddingRight={1} paddingLeft={0} paddingBottom={0}>
        <Item>
            <Stack direction="row" spacing={1}>
              <Item>
                <Gauge
                  width={230}
                  height={150}
                  value={-Round(props.packVoltage * props.packCurrent / 1000, 0)}
                  valueMin={0}
                  valueMax={32}
                  startAngle={-120}
                  endAngle={120}
                  text={
                    ({ value }) => `${value}`
                  }
                  sx={{
                    [`& .${gaugeClasses.valueText}`]: {
                      fontSize: 75,
                      transform: 'translate(0px, 0px)',
                    },
                  }}
                />
                <Typography align='center' fontWeight="fontWeightBold">kW</Typography>
              </Item>
              <Item>
                <Gauge
                  width={230}
                  height={150}
                  value={Round(props.speed, 0)}
                  valueMin={0}
                  valueMax={100}
                  startAngle={-120}
                  endAngle={120}
                  text={
                    ({ value }) => `${value}`
                  }
                  sx={{
                    [`& .${gaugeClasses.valueText}`]: {
                      fontSize: 75,
                      transform: 'translate(0px, 0px)',
                    },
                  }}
                />
                <Typography align='center' fontWeight="fontWeightBold">km/h</Typography>
              </Item>
            </Stack>
        </Item>
      </Grid>
    )
  //}
}