use hidapi::{HidApi, HidDevice, HidResult};

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

    let mute_status = run_cmd("pamixer --default-source --get-mute");
    let mute = mute_status.trim() == "true";
    let mut data = vec![0x00; REPORT_LENGTH];
    data[0] = 2;
    data[1] = if mute { 1 } else { 0 };
    send_raw_report(&interface, &data).unwrap();

    let volume_status = run_cmd("pamixer --get-volume");
    let mut data = vec![0x00; REPORT_LENGTH];
    data[0] = 3;
    data[1] = volume_status.trim().parse().unwrap();
    send_raw_report(&interface, &data).unwrap();

    loop {
        let mut response = [0u8; REPORT_LENGTH];
        interface.read(&mut response).unwrap();
        println!("{} {}", response[0], (response[1] + 61) as char);
        let layer = response[0];
        let key = (response[1] + 61) as char;

        match layer {
            0 => match key {
                'M' => {
                    run_cmd("ydotool key 97:1 231:1 231:0 97:0");
                }
                'N' => {
                    run_cmd("ydotool key 97:1 232:1 232:0 97:0");
                }
                'T' => {
                    run_cmd("pamixer -d 1");
                    let volume_status = run_cmd("pamixer --get-volume");
                    let mut data = vec![0x00; REPORT_LENGTH];
                    data[0] = 3;
                    data[1] = volume_status.trim().parse().unwrap();
                    send_raw_report(&interface, &data).unwrap();
                }
                'U' => {
                    run_cmd("pamixer -t");
                    let volume_status = run_cmd("pamixer --get-volume");
                    let mut data = vec![0x00; REPORT_LENGTH];
                    data[0] = 3;
                    data[1] = volume_status.trim().parse().unwrap();
                    send_raw_report(&interface, &data).unwrap();
                }
                'V' => {
                    run_cmd("pamixer -i 1");
                    let volume_status = run_cmd("pamixer --get-volume");
                    let mut data = vec![0x00; REPORT_LENGTH];
                    data[0] = 3;
                    data[1] = volume_status.trim().parse().unwrap();
                    send_raw_report(&interface, &data).unwrap();
                }
                'P' => {
                    run_cmd("pamixer --default-source -t");
                    let mute_status = run_cmd("pamixer --default-source --get-mute");
                    let mute = mute_status.trim() == "true";
                    let mut data = vec![0x00; REPORT_LENGTH];
                    data[0] = 2;
                    data[1] = if mute { 1 } else { 0 };
                    send_raw_report(&interface, &data).unwrap();
                }
                'Q' => {
                    run_cmd("$HOME/.local/bin/spotifyvolume down");
                }
                'R' => {
                    run_cmd("$HOME/.local/bin/spotifyvolume up");
                }
                'W' | 'B' => {
                    run_cmd("playerctl --ignore-player=firefox play-pause");
                }
                'Y' | 'C' => {
                    run_cmd("playerctl --ignore-player=firefox next");
                }
                'X' | 'A' => {
                    run_cmd("playerctl --ignore-player=firefox previous");
                }
                'D' => {
                    run_cmd("hyprctl clients | grep Spotify > /dev/null && hyprctl dispatch togglespecialworkspace music || hyprctl dispatch exec spotify --disable-smooth-scrolling");
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn run_cmd(command: &str) -> String {
    use std::process::Command;
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    String::from_utf8_lossy(&output.stdout).to_string()
}
