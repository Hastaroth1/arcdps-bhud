use super::device::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use named_pipe::PipeClient;
use std::io::Write;
use crate::worker::log;

const PIPE_PREFIX: &str = "\\\\.\\pipe\\";
static NAME: &'static str = "arcdpspipe";

pub fn new() {
    let action = |active: Arc<AtomicBool>, rx: Receiver<ChannelType>| {

        let mut client = PipeClient::connect(PIPE_PREFIX.to_string() + NAME);
        match client.as_mut(){
            Ok(pipe_client) => {

                log::send("Client Connected\n".into());
                let duration = Duration::new(1, 0);
                //active.store(pipeClient), Release);

                loop {
                    let data_to_send: Vec<u8> = if active.load(Acquire) {
                        log::send("data_to_Send\n".into());
                        rx.recv().unwrap()
                    } else {
                        log::send("nothing to send\n".into());
                        std::thread::sleep(duration);
//                        client = PipeClient::connect(PIPE_PREFIX.to_string() + NAME);
//                        if client.is_ok() {
//                            active.store(true, Release);
//                        }
                        active.store(true, Release);
                        continue;
                    };

                    log::send("sending data\n".into());
                    pipe_client.write(data_to_send.as_ref()).expect("could not send data");
                    pipe_client.flush().expect("could not flush");
                }
            },
            Err(e) => {
                log::send("failed to open pipe\n".into());
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
