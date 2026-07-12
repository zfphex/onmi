#![allow(unused)]

use crate::PlayerState;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Device {}

pub struct OutputDevices {}

impl OutputDevices {
    pub fn new() -> Self {
        todo!()
    }

    pub fn default_device(&self) -> Device {
        todo!()
    }

    pub fn find(&self, device: &str) -> Option<Device> {
        todo!()
    }
}

pub struct Output {
    device: Device,
}

unsafe impl Send for Output {}

pub fn run_output(_state: Arc<PlayerState>, _output: Output) {
    todo!()
}

pub fn new_output(device: Device, _sample_rate: Option<u32>) -> Output {
    Output { device }
}
