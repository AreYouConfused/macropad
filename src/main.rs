use std::{process::{Output, Command}, vec};

use hidapi::{HidApi, HidDevice, HidResult};

use mouse_keyboard_input::VirtualDevice;
use mouse_keyboard_input::key_codes::{KEY_F23, KEY_F24};

const VENDOR_ID: u16 = 0xd010;
const PRODUCT_ID: u16 = 0x1601;
const USAGE_PAGE: u16 = 0xFF60;
const USAGE: u16 = 0x61;
const REPORT_LENGTH: usize = 32;

fn get_raw_hid_interface() -> Option<HidDevice> {
    let api = HidApi::new().unwrap();
    let device_interfaces = api
        .device_list()
        .filter(|d| {
            d.vendor_id() == VENDOR_ID && d.product_id() == PRODUCT_ID && d.usage_page() == USAGE_PAGE && d.usage() == USAGE
        })
        .collect::<Vec<_>>();
    
    if device_interfaces.is_empty() {
        return None;
    }

    let interface = device_interfaces[0].open_device(&api).unwrap();
    println!("Manufacturer: {:?}", interface.get_manufacturer_string());
    println!("Product: {:?}", interface.get_product_string());

    Some(interface)
}

fn send_raw_report(interface: &HidDevice, data: &[u8]) -> HidResult<()> {
    let mut request_data = vec![0x00; REPORT_LENGTH + 1];
    request_data[1..data.len() + 1].copy_from_slice(data);
    let request_report = request_data.into_boxed_slice();

    println!("Request:");
    println!("{:?}", request_report);

    interface.write(&request_report)?;

    let mut response_report = [0u8; REPORT_LENGTH];
    interface.read_timeout(&mut response_report, 1000)?;

    println!("Response:");
    println!("{:?}", response_report);

    Ok(())
}

fn main() {
    let interface = match get_raw_hid_interface() {
        Some(interface) => interface,
        None => {
            println!("No device found");
            std::process::exit(1);
        }
    };

    let mut vdev = VirtualDevice::default().unwrap();

    send_raw_report(&interface, &vec![
        2, 
        if run_cmd_block("pamixer --default-source --get-mute").trim() == "true"
            { 1 } else { 0 },
    ]).unwrap();

    send_raw_report(&interface, &vec![
        3, 
        run_cmd_block("pamixer --get-volume").trim().parse().unwrap(),
    ]).unwrap();

    loop {
        let mut response = [0u8; REPORT_LENGTH];
        interface.read(&mut response).unwrap();
        println!("{} {}", response[0], (response[1] + 61) as char);
        let layer = response[0];
        let key = (response[1] + 61) as char;

        match layer {
            0 => match key {
                'M' => {
                    vdev.click(KEY_F23).unwrap();
                }
                'N' => {
                    vdev.click(KEY_F24).unwrap();
                }
                'T' => {
                    run_cmd("pamixer -d 1");
                    send_raw_report(&interface, &vec![
                        3, 
                        run_cmd_block("pamixer --get-volume").trim().parse().unwrap(),
                    ]).unwrap();
                }
                'U' => {
                    run_cmd("pamixer -t");
                    send_raw_report(&interface, &vec![
                        3, 
                        run_cmd_block("pamixer --get-volume").trim().parse().unwrap(),
                    ]).unwrap();
                }
                'V' => {
                    run_cmd("pamixer -i 1");
                    send_raw_report(&interface, &vec![
                        3, 
                        run_cmd_block("pamixer --get-volume").trim().parse().unwrap(),
                    ]).unwrap();
                }
                'P' => {
                    run_cmd("pamixer --default-source -t");
                    send_raw_report(&interface, &vec![
                        2, 
                        if run_cmd_block("pamixer --default-source --get-mute").trim() == "true"
                            { 1 } else { 0 },
                    ]).unwrap();
                }
                'Q' => {
                    run_cmd("$HOME/.local/bin/spotifyvolume down");
                }
                'R' => {
                    run_cmd("$HOME/.local/bin/spotifyvolume up");
                }
                'B' => {
                    run_cmd("playerctl --ignore-player=firefox play-pause");
                }
                'C' => {
                    run_cmd("playerctl --ignore-player=firefox next");
                }
                'A' => {
                    run_cmd("playerctl --ignore-player=firefox previous");
                }
                'D' => {
                    run_cmd("hyprctl clients | grep Spotify && hyprctl dispatch togglespecialworkspace music || hyprctl dispatch exec -- spotify --disable-smooth-scrolling");
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn run_cmd(command: &str) -> Output {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process")
    //String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_cmd_block(command: &str) -> String {
    let output = run_cmd(command);
    String::from_utf8_lossy(&output.stdout).to_string()
}
