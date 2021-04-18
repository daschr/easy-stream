use rust_embed::RustEmbed;
use std::env;
use std::sync::{Arc, RwLock};
use std::thread;
use tiny_http::{Response, Server};

mod cam_worker;

const UPDATE_INTERVAL: u64 = 100u64;
const DEFAULT_RES: (u32, u32) = (1920u32, 1080u32);

#[derive(RustEmbed)]
#[folder = "html/"]
struct HtmlAssets;

fn main() {
    let argc = env::args().len();

    if argc == 2 && env::args().nth(1).unwrap() == "-h" {
        eprintln!(
            "Usage: {} [?device num] [?resolution]",
            env::args().nth(0).unwrap()
        );
        return;
    }

    let device_num = match argc {
        1 => 0,
        _ => env::args()
            .nth(1)
            .unwrap()
            .parse::<usize>()
            .expect("this needs to be a number!"),
    };

    let resolution = if argc >= 3 {
        let s = env::args().nth(2).unwrap();
        let mut sp = s.split("x");
        (
            sp.next().unwrap().parse::<u32>().unwrap(),
            sp.next().unwrap().parse::<u32>().unwrap(),
        )
    } else {
        DEFAULT_RES
    };

    let update_interval = if argc >= 4 {
        env::args().nth(3).unwrap().parse::<u64>().unwrap()
    } else {
        UPDATE_INTERVAL
    };

    let buffer = Arc::new(RwLock::new(Vec::new()));
    let c_buffer = Arc::clone(&buffer);

    thread::spawn(move || {
        cam_worker::update_framebuf(device_num, update_interval, resolution, c_buffer);
    });

    let server = Server::http("0.0.0.0:8000").expect("could not start server!");

    for request in server.incoming_requests() {
        let response = match request.url() {
            "/" | "/index.html" => Response::from_data(HtmlAssets::get("index.html").unwrap()),
            _ => {
                let data = buffer.read().unwrap().clone();
                Response::from_data(data)
            }
        };
        match request.respond(response) {
            _ => {}
        };
    }
}
