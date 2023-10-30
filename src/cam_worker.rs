use anyhow::Result;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;
use v4l::Format;
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
        let mut dev = Device::new(self.device)?;

        dev.set_format(&Format::new(
            self.resolution.0,
            self.resolution.1,
            FourCC::new(b"MJPG"),
        ))?;

        let mut stream = MmapStream::with_buffers(&mut dev, v4l::buffer::Type::VideoCapture, 4)?;

        loop {
            {
                let mut buf = self.frame.write().unwrap();
                let next_frame = stream.next().unwrap();
                buf.resize(next_frame.0.len(), 0u8);
                buf.copy_from_slice(next_frame.0);
            }

            sleep(self.update_interval).await;
        }
    }
}
