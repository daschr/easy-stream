use std::env;
use std::sync::{Arc, RwLock};
use std::thread;
use tiny_http::{Response, Server};
use rust_embed::RustEmbed;

mod cam_worker;

#[derive(RustEmbed)]
#[folder = "html/"]
struct HtmlAssets;

fn main() {
    if env::args().len() != 3 {
        eprintln!(
            "Usage: {} [device num] [resolution]",
            env::args().nth(0).unwrap()
        );
        return;
    }

    let device_num = env::args()
        .nth(1)
        .unwrap()
        .parse::<usize>()
        .expect("this needs to be a number!");
    let resolution = {
        let s = env::args().nth(2).unwrap();
        let mut sp = s.split("x");
        (
            sp.next().unwrap().parse::<u32>().unwrap(),
            sp.next().unwrap().parse::<u32>().unwrap(),
        )
    };

    let buffer = Arc::new(RwLock::new(Vec::new()));
    let c_buffer = Arc::clone(&buffer);

    thread::spawn(move || {
        cam_worker::update_framebuf(device_num, 250, resolution, c_buffer);
    });

    let server = Server::http("0.0.0.0:8000").expect("could not start server!");

    for request in server.incoming_requests() {
        let response = match request.url() {
            "/" | "/index.html"  => Response::from_data(HtmlAssets::get("index.html").unwrap()),
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
