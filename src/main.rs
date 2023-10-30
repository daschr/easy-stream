use axum::{routing::get, Router, Server};
use cam_worker::FrameUpdater;
use clap::Parser;

use log::{self, debug};

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::task;
use webstream::StreamConfig;

use std::net::SocketAddr;

mod cam_worker;
mod camstream;
mod webstream;

const DEFAULT_FPS: f64 = 10f64;
const DEFAULT_RES: (u32, u32) = (1920u32, 1080u32);

/// Spawns a webserver which streams a webcam video
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None
)]

struct Args {
    /// index of the v4l device to grab from
    #[arg(long, default_value_t = 0)]
    device: usize,

    /// fps to grab from the camera
    #[arg(long, default_value_t = DEFAULT_FPS)]
    fps: f64,

    /// turn on debug messages
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// resolution [default: 1920x1080]
    #[arg(long)]
    resolution: Option<String>,

    /// listen [default: 0.0.0.0:8000]
    #[arg(long)]
    listen: Option<SocketAddr>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    simple_logger::init_with_level(if args.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    })
    .expect("Could not initialize logger");

    let resolution = if let Some(res) = args.resolution {
        let mut sp = res.split("x");
        (
            sp.next().unwrap().parse::<u32>().unwrap(),
            sp.next().unwrap().parse::<u32>().unwrap(),
        )
    } else {
        DEFAULT_RES
    };

    let buffer = Arc::new(RwLock::new(Vec::new()));
    let interval = Duration::from_millis((1000.0f64 / args.fps) as u64);

    debug!("interval: {:?}", &interval);
    let updater = FrameUpdater::new(args.device, interval, resolution, buffer.clone());

    task::spawn(async move {
        updater
            .update_framebuf()
            .await
            .expect("failure in FrameUpdater");
    });

    let stream_config = Arc::new(StreamConfig {
        interval,
        buf: buffer.clone(),
    });

    let router = Router::new()
        .route("/", get(webstream::stream))
        .fallback(webstream::fallback)
        .with_state(stream_config);

    let addr: SocketAddr = args
        .listen
        .unwrap_or("0.0.0.0:8000".parse::<SocketAddr>().unwrap());

    let builder = Server::bind(&addr);
    builder.serve(router.into_make_service()).await.unwrap();
}
