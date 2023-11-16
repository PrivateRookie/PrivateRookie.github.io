use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use clap::Parser;
use playground::init_log;
use time::OffsetDateTime;
use tracing::info;

#[derive(Parser)]
pub enum Command {
    L {
        #[arg(default_value = "127.0.0.1:9000")]
        addr: SocketAddr,
    },
    C {
        #[arg(default_value = "127.0.0.1:9000")]
        addr: SocketAddr,
        #[arg(short, long, default_value = "1000")]
        count: usize,
        #[arg(short, long, default_value = "1024")]
        size: u32,
    },
}

fn main() {
    init_log();
    let cmd = Command::parse();
    match cmd {
        Command::L { addr } => {
            let listener = TcpListener::bind(addr).unwrap();
            info!("listening at {addr}");
            for mut stream in listener.incoming().flatten() {
                let mut state = RecvState::default();
                let mut write_buf = vec![];
                'out: loop {
                    if state
                        .read_stream(&mut stream)
                        .map(|cnt| cnt == 0)
                        .unwrap_or(true)
                    {
                        break;
                    }
                    loop {
                        if let Ok(msg) = state.consume() {
                            write_buf.clear();
                            msg.encode(&mut write_buf);
                            if stream.write_all(&write_buf).is_err() {
                                break 'out;
                            }
                        } else {
                            break;
                        }
                    }
                }
                info!("peer go away");
            }
        }
        Command::C { addr, count, size } => {
            info!("sending {count} messages with {size} bytes as payload size");
            let data = vec![0; size as usize];
            let msg = Message { len: size, data };
            let mut buf = vec![];
            msg.encode(&mut buf);
            let mut stream = TcpStream::connect(addr).unwrap();
            let mut state = RecvState::default();
            let start = OffsetDateTime::now_utc();
            for _ in 0..count {
                if let Err(e) = stream.write_all(&buf) {
                    tracing::error!("{e}");
                    break;
                }
                if let Err(e) = state.read_stream(&mut stream) {
                    tracing::error!("{e}");
                    break;
                }
                assert!(state.consume().is_ok());
                assert!(state.consume().is_err());
            }
            let end = OffsetDateTime::now_utc();
            let delta = end - start;
            let mps = size as f64 / delta.whole_seconds() as f64;
            info!("time consumed: {} {mps:.2} message/sec", end - start);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub len: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum DecodeError {
    LenError,
    NotEnoughData,
}

impl Message {
    pub fn encode_len(&self) -> usize {
        self.data.len() + 4
    }

    pub fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.len.to_be_bytes());
        buf.extend_from_slice(&self.data);
    }

    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        if buf.len() < 4 {
            return Err(DecodeError::LenError);
        }
        let len = u32::from_be_bytes(buf[..4].try_into().unwrap());
        if len as usize + 4 > buf.len() {
            return Err(DecodeError::NotEnoughData);
        }
        let data = buf[4..(4 + len as usize)].to_vec();
        Ok(Self { len, data })
    }
}

const BUF_SIZE: usize = 8192;
#[derive(Debug)]
pub struct RecvState {
    buf: [u8; BUF_SIZE],
    remain: usize,
    consume_start: usize,
    consume_end: usize,
}

impl Default for RecvState {
    fn default() -> Self {
        Self {
            buf: [0; BUF_SIZE],
            remain: Default::default(),
            consume_start: Default::default(),
            consume_end: Default::default(),
        }
    }
}

impl RecvState {
    pub fn read_stream(&mut self, stream: &mut TcpStream) -> std::io::Result<usize> {
        let cnt = stream.read(&mut self.buf[self.remain..])?;
        self.consume_end = cnt + self.remain;
        Ok(cnt)
    }

    pub fn consume(&mut self) -> Result<Message, DecodeError> {
        match Message::decode(&self.buf[self.consume_start..self.consume_end]) {
            Ok(msg) => {
                let offset = msg.encode_len();
                self.consume_start += offset;
                Ok(msg)
            }
            Err(e) => {
                let remain_buf = self.buf[self.consume_start..self.consume_end].to_vec();
                self.remain = remain_buf.len();
                self.buf[..self.remain].copy_from_slice(&remain_buf);
                self.consume_start = 0;
                self.consume_end = self.remain;
                Err(e)
            }
        }
    }
}
