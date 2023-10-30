use anyhow::Result;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use v4l::prelude::*;
use v4l::FourCC;

pub struct FrameUpdater {
    device: usize,
    update_interval: Duration,
    resolution: (u32, u32),
    frame: Arc<RwLock<Vec<u8>>>,
}

impl FrameUpdater {
    pub fn new(
        device: usize,
        update_interval: Duration,
        resolution: (u32, u32),
        frame: Arc<RwLock<Vec<u8>>>,
    ) -> Self {
        FrameUpdater {
            device,
            update_interval,
            resolution,
            frame,
        }
    }

    pub async fn update_framebuf(&self) -> Result<()> {
        let mut dev = CaptureDevice::new(self.device)?;

        let mut fmt = dev.format().expect("failed reading format!");
        fmt.width = self.resolution.0;
        fmt.height = self.resolution.1;
        fmt.fourcc = FourCC::new(b"MJPG");
        dev.set_format(&fmt)?;

        let mut stream = MmapStream::with_buffers(&mut dev, 4)?;

        loop {
            {
                let mut buf = self.frame.write().unwrap();
                let next_frame = stream.next().unwrap();
                buf.resize(next_frame.len(), 0u8);
                buf.copy_from_slice(next_frame.data());
            }

            sleep(self.update_interval).await;
        }
    }
}
