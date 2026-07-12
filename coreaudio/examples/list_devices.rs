use coreaudio::*;

fn main() {
    println!("Core Audio Devices:\n-------------------");

    let devices = AudioDevice::system_devices().expect("Failed to get system devices");

    for device in devices {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        let inputs = device.input_channel_count().unwrap_or(0);
        let outputs = device.output_channel_count().unwrap_or(0);

        println!("- ID {}: {}", device.id, name);
        println!("  Channels: {} in, {} out", inputs, outputs);
    }

    println!(
        "\nDefault Input: {:?}",
        AudioDevice::default_input().and_then(|d| d.name())
    );
    println!(
        "Default Output: {:?}",
        AudioDevice::default_output().and_then(|d| d.name())
    );
}
