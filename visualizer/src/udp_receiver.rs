use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver},
    thread,
};

/// Maximum UDP payload size the receiver will accept.
const MAX_PACKET_SIZE: usize = 65_507;

/// A raw UDP packet together with the sender's address.
pub struct Packet {
    pub data: Vec<u8>,
    pub source: SocketAddr,
}

/// Binds a UDP socket to `listen_addr`, spawns a background thread that reads
/// incoming packets, and returns the receiving end of the channel.
///
/// The thread runs until the channel's [`Receiver`] is dropped, at which point
/// the next send will fail and the thread will exit gracefully.
///
/// # Panics
/// Panics if the socket cannot be bound to `listen_addr`.
pub fn spawn(listen_addr: SocketAddr) -> Receiver<Packet> {
    let socket = UdpSocket::bind(listen_addr)
        .unwrap_or_else(|err| panic!("failed to bind UDP socket on {listen_addr}: {err}"));

    log::info!("UDP receiver listening on {listen_addr}");

    let (tx, rx) = mpsc::channel::<Packet>();

    thread::Builder::new()
        .name("udp-receiver".into())
        .spawn(move || {
            let mut buf = vec![0u8; MAX_PACKET_SIZE];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((len, source)) => {
                        let packet = Packet {
                            data: buf[..len].to_vec(),
                            source,
                        };
                        if tx.send(packet).is_err() {
                            // Receiver dropped.
                            log::debug!("UDP receiver: channel closed, exiting thread");
                            break;
                        }
                    }
                    Err(err) => {
                        log::error!("UDP recv error: {err}");
                    }
                }
            }
        })
        .expect("failed to spawn udp-receiver thread");

    rx
}
