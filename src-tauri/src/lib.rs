mod core;
/// Auther: WannaR
/// Recive and Process DAS data
///
mod error;

use std::io::Read;
// use std::sync::mpsc;
use std::time::{self, Duration};

#[allow(unused)]
use core::deconvolve1d;
use error::{Error, Result};
use lazy_static::lazy_static;
use rusb::{self, Device, GlobalContext};
use serde::{Deserialize, Serialize};
use serialport;
use tauri::async_runtime::JoinHandle;
use tauri::utils::debug_eprintln;
use tauri::{ipc::Channel, AppHandle, Manager, ResourceId, Runtime};
use tokio::sync::mpsc;
use tokio::sync::Mutex;

type DataReciver = Mutex<mpsc::Receiver<(RawDASFrame, u64)>>;
type DataSender = Mutex<mpsc::Sender<(RawDASFrame, u64)>>;

/// speed of light
const C: f64 = 299792458.458;
/// mode
// const DEV: bool = true;
/// Lower limit for pulse judgment
const PULSE_THRESHOLD: u64 = 1;
/// kernel len
// const KERNEL_LEN: usize = 18;
/// kernel, shape of pulse
// const KERNEL: [f64; KERNEL_LEN] = [1.0; KERNEL_LEN];
/// packet length
const PACKET_LENGTH: usize = 3779;

lazy_static! {
    /// channel from port to compute unit
    static ref DATA_CHANNEL: (DataSender, DataReciver) = {
        let (s,r) = mpsc::channel(1);
        (Mutex::new(s), Mutex::new(r))
    };
}

struct ListenerJoinHandler<T> {
    join_handler: JoinHandle<T>,
}

impl<T: Sync + Send + 'static> tauri::Resource for ListenerJoinHandler<T> {}

#[derive(Debug, Serialize, Deserialize)]
struct PlotlyData {
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
    #[serde(rename = "type")]
    plot_type: String,
    showscale: bool,
    mode: String,
}

impl From<DASFrame> for PlotlyData {
    fn from(value: DASFrame) -> Self {
        PlotlyData {
            x: value.data.iter().map(|(distance, _)| *distance).collect(),
            y: vec![value.timestamp.as_secs_f64(); value.data.len()],
            z: value
                .data
                .iter()
                .map(|(_, strain)| *strain as f64)
                .collect(),
            plot_type: String::from("scatter3d"),
            mode: String::from("lines"),
            showscale: false,
        }
    }
}

// #[derive(Debug)]
// struct DASData {
//     sequence: Vec<DASFrame>,
//     info: DASInfo,
// }

#[derive(Debug, Serialize, Default)]
struct DASFrame {
    /// [distance/m, strain/]
    data: Vec<(f64, f64)>,
    /// ms
    timestamp: Duration,
}

#[derive(Debug)]
struct RawDASFrame {
    /// voltage
    data: Vec<u16>,
    /// start time (from UNIX_EPOCH)
    timestamp: Duration,
}

// struct DasPackge {}

#[derive(Debug, Clone, Deserialize)]
struct DASInfo {
    /// length of the FUT (m)
    length: f64,
    /// s^-1
    sample_rate: u64,
    /// μs
    pulse_interval: u128,
}

impl RawDASFrame {
    /// ## core function
    /// Demodulate the original information
    fn to_das_frame(&self, info: &DASInfo, start: Duration) -> Result<DASFrame> {
        if let Some(pulse_head) = self.find_pulse_start_index() {
            let sample_count = (info.length / C * info.sample_rate as f64) as u64 * 2;
            let pulse_tail = pulse_head + sample_count;
            // 检查pulse是否超出了采样范围
            if pulse_tail > self.data.len() as u64 {
                println!("pulse out of range: Pulse tail {}", pulse_tail);
                return Err(Error::PulseOutOfRange);
            }
            let pulse_raw = self.data[pulse_head as usize..pulse_tail as usize]
                .to_vec()
                .iter()
                .map(|v| *v as f64)
                .collect::<Vec<f64>>();
            // let pulse = deconvolve1d(pulse_raw.as_slice(), &KERNEL);
            let pulse = pulse_raw;
            // let pulse = core::fft(pulse.as_slice());
            let mut index = 0.0;
            let mut data = pulse
                .iter()
                .map(|&v| {
                    let distance = (index / pulse.len() as f64) * info.length;
                    index += 1.0;
                    (distance, v)
                })
                .collect::<Vec<_>>();
            // let data = pulse_raw
            //     .iter()
            //     .map(|&v| {
            //         let distance = (index / pulse.len() as f64) * info.length;
            //         index += 1.0;
            //         (distance, v)
            //     })
            //     .collect();

            // debug_eprintln!("Data: {:?}", data);
            data.remove(0); // 无效数据
            Ok(DASFrame {
                data,
                timestamp: self.timestamp - start,
            })
        } else {
            Err(Error::NoPulseFound)
        }
    }

    fn find_pulse_start_index(&self) -> Option<u64> {
        let mut index = 0;
        for i in self.data.chunks_exact(2) {
            // Prevent overflow
            if i.iter().map(|x| *x as u64).sum::<u64>() >= PULSE_THRESHOLD {
                return Some(index);
            }
            index += 2;
        }
        None
    }
}

// fn write_das_data(data: DASData, path: &str) -> Result<()> {
//     !todo!()
// }

#[allow(dead_code)]
async fn start_dev_service(tx: DataSender) {
    let mut metadata: (u8, u64) = (1, 0);
    let tx = DATA_CHANNEL.0.lock().await;
    loop {
        let mut buffer = tokio::fs::read(format!(
            "D:/code/DAS/_mock/data/packet_{}_for_100.0ms.bin",
            metadata.0
        ))
        .await
        .expect("read error");
        buffer.remove(0);
        let raw_frame = parse_to_rawframe(buffer);
        tx.send((raw_frame, metadata.1))
            .await
            .expect("tx send error");
        // debug_eprintln!(
        //     "[Send] sending D:/code/DAS/_mock/data/packet_{}_for_100.0ms.bin",
        //     file_index
        // );
        metadata = ((metadata.0) % 9 + 1, metadata.1 + 1);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[allow(dead_code)]
#[allow(arithmetic_overflow)]
async fn start_listen_usb(usb: Device<GlobalContext>, tx: DataSender) {
    let mut metadata: (u8, u64) = (0, 0);
    let usb_port = usb.open().expect("can not open device");
    let tx = DATA_CHANNEL.0.lock().await;
    loop {
        let mut buffer = vec![0u8; PACKET_LENGTH];
        let _len = usb_port
            .read_bulk(3, &mut buffer, Duration::from_millis(500))
            .expect("read error");
        let index = buffer.remove(0);
        if metadata.0 != index {
            eprintln!(
                "expect {}, but found {} when processing {:?}",
                metadata.0, index, metadata
            );
            metadata.0 = index;
        } else {
            tx.send((parse_to_rawframe(buffer), metadata.1))
                .await
                .expect("tx send error");
        }
        // overflow, and then loop...
        metadata = ((metadata.0 + 1), metadata.1 + 1);
    }
}

async fn start_listen_serial(mut port: serialport::COMPort) {
    let mut metadata: (u64, u64) = (1, 1);
    let tx = DATA_CHANNEL.0.lock().await;
    loop {
        let mut buffer = vec![0u8; PACKET_LENGTH];
        let _len = port.read(&mut buffer);
        let index = buffer.remove(0) as u64;
        if metadata.0 != index {
            eprintln!(
                "expect {}, but found {} when processing {:?}",
                metadata.0, index, metadata
            );
            metadata.0 = index;
        } else {
            println!("[send] {:?}", metadata);
            tx.send((parse_to_rawframe(buffer), metadata.1))
                .await
                .expect("tx send error");
        }
        // overflow, and then loop...
        metadata = ((metadata.0 + 1) % 256, metadata.1 + 1);
    }
}

fn parse_to_rawframe(mut buffer: Vec<u8>) -> RawDASFrame {
    let sample_per_pulse = buffer.remove(0) as u64; // 填满一次20us所需要的采样数 17
    let pulse_per_frame = buffer.remove(0) as u64; // 采样时间所占96Mhz的周期数，需要错出来的周期数 111
    let _a = buffer.remove(0); // 0 暂时无意义 占位
                               // println!(
                               //     "sample_per_pulse: {}, pulse_per_frame: {}, _a: {}",
                               //     sample_per_pulse, pulse_per_frame, _a
                               // );
    let timestamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("cannot get time stamp");

    let iter = buffer.chunks_exact((sample_per_pulse * 2) as usize); // size是u8， 数据是u16的
    let mut data = vec![0; (sample_per_pulse * pulse_per_frame + 1) as usize]; // 采样时前错一位，所以加一
    let mut count = 0;
    // 循环 pulse_per_frame 次
    for pulse in iter {
        let mut index = 0;
        // 循环 sample_per_pulse 次
        for sample in pulse.chunks(2).map(|v| (v[0] as u16) << 8 | v[1] as u16) {
            data[(index * pulse_per_frame + count) as usize] = sample;
            index += 1;
        }
        count += 1;
    }

    // let data = buffer
    //     .chunks_exact(2)
    //     .map(|v| (v[0] as u16) << 8 | v[1] as u16)
    //     .collect();

    RawDASFrame { data, timestamp }
}

#[tauri::command]
async fn start<R: Runtime>(
    app: AppHandle<R>,
    on_data: Channel,
    info: DASInfo,
) -> Result<ResourceId> {
    debug_eprintln!("starting das with info: {:?}", info);
    // ugly!!!!
    let recv: tauri::async_runtime::JoinHandle<()> = tauri::async_runtime::spawn(async move {
        // let mut rx = handle.state::<DataReciver>().inner().lock().await;
        // let mut table = handle.resources_table();
        // let binding = table.take::<DataReciverWrapper>(recv_rid).unwrap();
        // let rx_wrapper = std::ops::Deref::deref(&binding);
        // let mut rx = rx_wrapper.reciver.lock().await;
        // let rx = std::ops::DerefMut::deref_mut(&mut rx);
        let mut rx = DATA_CHANNEL.1.lock().await;

        let r = rx.recv().await;
        let start = r.unwrap().0.timestamp;
        loop {
            if let Some((raw_frame, index)) = rx.recv().await {
                //         let raw_frames = parse_to_rawframe(bytes);
                match raw_frame.to_das_frame(&info, start) {
                    Ok(frame) => {
                        // debug_eprintln!("[Recv] data: {:?}", index);
                        let _ = on_data.send::<PlotlyData>(frame.into());
                    }
                    Err(e) => {
                        eprintln!("calculate error: {:#?} when processing {:?}", e, index);
                    }
                }
            } else {
                break;
            }
        }
    });
    let mut table = app.resources_table();
    let receiver_handler = ListenerJoinHandler { join_handler: recv };
    Ok(table.add(receiver_handler))
}

#[tauri::command]
fn stop(app: AppHandle, rid: ResourceId) -> Result<u32> {
    let mut table = app.resources_table();
    // actually there is <ReceiverHandler<!>>, but I don't want to use the unstable feature :)
    let recv_handle = table.take::<ListenerJoinHandler<()>>(rid)?;
    recv_handle.join_handler.abort();
    Ok(0)
}

#[tauri::command]
async fn connect_das(app: AppHandle, usb_id: u16) -> Result<ResourceId> {
    let mut maybe_das: Option<Device<GlobalContext>> = None;
    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();
        // device.open().unwrap().read_bulk(endpoint, buf, timeout)
        println!(
            "Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id()
        );
        println!("{:?}", device.speed());
        if device_desc.product_id() == usb_id {
            maybe_das = Some(device);
        }
    }

    // let (tx, rx) = mpsc::channel(1);
    if let Some(das) = maybe_das.take() {
        println!("device: {:?}", das.device_descriptor().unwrap());
    }

    // if let Some(das) = maybe_das.take() {
    // let desc = das.device_descriptor().unwrap();
    // if desc.class_code() == rusb::constants::LIBUSB_CLASS_COMM {
    // let mut port: std::prelude::v1::Result<serialport::COMPort, serialport::Error>;
    loop {
        let port = serialport::new("COM4", 115200).open_native();
        match port {
            Ok(_) => break,
            Err(_) => (),
        }
    }
    println!("Connected to DAS");
    let port = serialport::new("COM4", 115200).open_native();
    let handler =
        tauri::async_runtime::spawn(async move { start_listen_serial(port.unwrap()).await });
    // } else {
    //     tauri::async_runtime::spawn(async move { start_listen_usb(das, tx).await });
    // }
    // } else if DEV {
    //     eprintln!("device not found! using _mock");
    //     tauri::async_runtime::spawn(async move { start_dev_service(tx).await });
    // } else {
    //     return Err(Error::DeviceNotFound);
    // }
    // manage rx: DataReciver
    let mut table = app.resources_table();
    // let rx: DataReciver = Mutex::new(rx);
    // app.manage(rx);
    Ok(table.add(ListenerJoinHandler {
        join_handler: handler, // das_join_handle
    }))
}

#[tauri::command]
fn disconnect_das<R: Runtime>(app: tauri::AppHandle<R>, das_rid: ResourceId) -> Result<u32> {
    let mut table = app.resources_table();
    let das_join_handle = table.take::<ListenerJoinHandler<()>>(das_rid)?;
    das_join_handle.join_handler.abort();
    println!("disconnected DAS");
    Ok(0)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    tauri::Builder::default()
        .setup(move |app| {
            lazy_static::initialize(&DATA_CHANNEL);
            tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .inner_size(840.0, 920.0)
            .title("DAS")
            .initialization_script(
                "setTimeout(()=>{window.document.getElementsByClassName('plotlyjsicon')[0].remove();},500)",
            )
            .build()?;
            Ok(())
        })
        // .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![start,stop,connect_das,disconnect_das])
        .run(tauri::generate_context!())
        .expect("error while running DAS");
    Ok(())
}
