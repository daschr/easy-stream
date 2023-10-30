use axum::body::StreamBody;
use axum::extract::State;
use axum::http::Uri;
use axum::response::IntoResponse;
use log::debug;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::camstream::CamStream;

#[derive(Debug)]
pub struct StreamConfig {
    pub interval: Duration,
    pub buf: Arc<RwLock<Vec<u8>>>,
}

pub async fn stream(State(state): State<Arc<StreamConfig>>) -> impl IntoResponse {
    let stream = CamStream::new(state.interval, state.buf.clone());

    (
        [
            ("Connection", "Keep-Alive"),
            ("Keep-Alive", "timeout=15"),
            (
                "Content-Type",
                "multipart/x-mixed-replace;boundary=123456789000000000000987654321",
            ),
        ],
        StreamBody::new(stream),
    )
}

pub async fn fallback(uri: Uri) -> &'static str {
    debug!("[fallback] uri: {:?}", uri);
    "not found"
}
