use super::device::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use named_pipe::PipeClient;
use std::io::Write;
use crate::worker::log;
use std::str;
const PIPE_PREFIX: &str = "\\\\.\\pipe\\";
static NAME: &'static str = "arcdpspipe";

pub fn new() {
    let action: fn(Arc<AtomicBool>, Receiver<Vec<u8>>) = |active: Arc<AtomicBool>, rx: Receiver<ChannelType>| {
        log::send("Opening pipe".into());
        let mut client = PipeClient::connect(PIPE_PREFIX.to_string() + NAME);
        let duration = Duration::new(1, 0);
        active.store(client.is_ok(), Release);

        loop {
            let data_to_send = if active.load(Acquire) {
                log::send("data_to_Send\n".into());
                rx.recv().unwrap()
            } else {
                log::send("nothing to send\n".into());
                std::thread::sleep(duration);
                client = PipeClient::connect(PIPE_PREFIX.to_string() + NAME);
                if client.is_ok() {
                    active.store(true, Release);
                }
                continue;
            };

            if client.is_err() {
                client = PipeClient::connect(PIPE_PREFIX.to_string() + NAME);
            }

            if let Ok(pipe_client) = &mut client {
                active.store(true, Release);
                let bytes = build_array(data_to_send.as_ref());

                pipe_client.write(bytes.as_ref());
                pipe_client.flush();
            } else {
                active.store(false, Release);
            }
        }
    };

    gen_device(NAME, action);
}

pub fn send(content: ChannelType) {
    send_to_device(NAME, content);
}

fn build_array(bytes: &[u8]) -> Vec<u8> {
    [&bytes.len().to_le_bytes(), bytes].concat()
}
