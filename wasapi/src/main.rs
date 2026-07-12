use wasapi::*;

fn main() {
    unsafe {
        CoInitializeEx(ConcurrencyModel::MultiThreaded).unwrap();

        let enumerator = IMMDeviceEnumerator::new().unwrap();
        let default = enumerator
            .GetDefaultAudioEndpoint(DataFlow::Render, Role::Console)
            .unwrap();

        let collection = enumerator
            .EnumAudioEndpoints(DataFlow::All, DeviceState::Disabled)
            .unwrap();

        let devices: Vec<IMMDevice> = (0..collection.GetCount().unwrap())
            .map(|i| collection.Item(i).unwrap())
            .collect();

        println!("Default: {}", default.name());

        for device in devices {
            println!("{}", device.name());
        }

        let client: IAudioClient = default.Activate(ExecutionContext::All).unwrap();

        // let fmt = *fmt_ptr;
        // let mut format = if fmt.cbSize == 22 && fmt.wFormatTag as u32 == WAVE_FORMAT_EXTENSIBLE {
        //     (fmt_ptr as *const _ as *const WAVEFORMATEXTENSIBLE).read()
        // } else {
        //     todo!("Unsupported format?");
        // };

        // if format.Format.nChannels < 2 {
        //     todo!("Support mono devices.");
        // }

        // //Update format to desired sample rate.
        // if let Some(sample_rate) = sample_rate {
        //     assert!(COMMON_SAMPLE_RATES.contains(&sample_rate));
        //     format.Format.nSamplesPerSec = sample_rate;
        //     format.Format.nAvgBytesPerSec = sample_rate * format.Format.nBlockAlign as u32;
        // }

        // let mut default_period = 0;
        // audio_client
        //     .GetDevicePeriod(Some(&mut default_period), None)
        //     .unwrap();

        let (default, _) = client.GetDevicePeriod().unwrap();
        let format =
            (client.GetMixFormat().unwrap() as *const _ as *const WAVEFORMATEXTENSIBLE).read();

        let block_align = format.Format.nBlockAlign;

        client
            .Initialize(
                ShareMode::Shared,
                AUDCLNT_STREAMFLAGS_EVENTCALLBACK
                    | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                    | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                default,
                default,
                &format as *const _ as *const WAVEFORMATEX,
                None,
            )
            .unwrap();
        let event = CreateEventA(core::mem::zeroed(), 0, 0, core::mem::zeroed());
        assert!(!event.is_null());
        client.SetEventHandle(event as isize).unwrap();

        let render: IAudioRenderClient = client.GetService().unwrap();

        client.Start().unwrap();

        let padding = client.GetCurrentPadding().unwrap();
        let size = client.GetBufferSize().unwrap();
        let nframes = size - padding - 1;
        let buffer = render.GetBuffer(nframes).unwrap();

        let size = nframes as usize * block_align as usize;

        let slice = core::slice::from_raw_parts(buffer, size);
        dbg!(slice.len());
    }
}
