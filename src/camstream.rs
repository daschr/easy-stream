use tokio::{task::JoinHandle, time::Instant};

use core::pin::Pin;
use futures::{
    stream::Stream,
    task::{Context, Poll},
};
use std::sync::{Arc, RwLock};
use tokio::task;
use tokio::time::{sleep, Duration};

pub struct CamStream {
    interval: Duration,
    buf: Arc<RwLock<Vec<u8>>>,
    last_update: Option<Instant>,
    waker: Option<JoinHandle<()>>,
}

impl CamStream {
    pub fn new(interval: Duration, buf: Arc<RwLock<Vec<u8>>>) -> Self {
        CamStream {
            interval,
            buf,
            last_update: None,
            waker: None,
        }
    }
}

impl Drop for CamStream {
    fn drop(&mut self) {
        if let Some(waker) = self.waker.as_mut() {
            waker.abort();
        }
    }
}

impl Stream for CamStream {
    type Item = Result<Vec<u8>, &'static str>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(last_update) = self.last_update {
            let diff = Instant::now().duration_since(last_update);
            if diff < self.interval {
                if let Some(waker) = self.waker.as_mut() {
                    waker.abort();
                }

                let waker = ctx.waker().clone();
                self.waker = Some(task::spawn(async move {
                    sleep(diff).await;
                    waker.wake();
                }));

                return Poll::Pending;
            }
        }

        self.last_update = Some(Instant::now());

        let mut local_buf: Vec<u8> = Vec::new();
        {
            local_buf.clear();
            let rem_buf = self.buf.read().unwrap();
            local_buf.extend_from_slice(
                format!(
                    "--123456789000000000000987654321\r\n\
Content-Type: image/jpeg\r\n\
Content-Length: {}\r\n\r\n",
                    rem_buf.len()
                )
                .as_bytes(),
            );
            local_buf.extend_from_slice(&rem_buf);
        }

        Poll::Ready(Some(Ok(local_buf)))
    }
}
