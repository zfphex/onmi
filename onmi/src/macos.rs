#![allow(unused)]
#[derive(Debug, Clone)]
pub struct Device {}

pub struct OutputDevices {}

impl OutputDevices {
    pub fn new() -> Self {
        todo!()
    }

    pub fn find(&self, device: &str) -> Option<Device> {
        todo!()
    }
}

pub struct Output {
    device: Device,
}

pub fn run_output(output: Output) {
    todo!()
}

pub fn new_output(device: Device, sample_rate: Option<u32>) -> Output {
    todo!()
}
