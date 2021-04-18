use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use v4l::prelude::*;
use v4l::FourCC;

pub fn update_framebuf(
    device: usize,
    update_interval: u64,
    resolution: (u32, u32),
    frame: Arc<RwLock<Vec<u8>>>,
) {
    let mut dev = CaptureDevice::new(device).expect("Failed to open device");

    let mut fmt = dev.format().expect("failed reading format!");
    fmt.width = resolution.0;
    fmt.height = resolution.1;
    fmt.fourcc = FourCC::new(b"MJPG");
    dev.set_format(&fmt)
        .expect("failed settting image properties!");

    let mut stream =
        MmapStream::with_buffers(&mut dev, 4).expect("failed to create buffered stream");

    loop {
        {
            let mut buf = frame.write().unwrap();
            let next_frame = stream.next().unwrap();
            buf.resize(next_frame.len(), 0u8);
            buf.copy_from_slice(next_frame.data());
        }

        thread::sleep(Duration::from_millis(update_interval));
    }
}
