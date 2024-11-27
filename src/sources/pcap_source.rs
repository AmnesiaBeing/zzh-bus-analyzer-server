use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::time::Duration;

use log::{error, info};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, Config, DataLinkReceiver};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols::{self};
use pnet::packet::sll::SLLPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

use crate::errors::MyError;
use crate::types::SomeipMessage;

pub struct PcapFileSource {
    file_path: PathBuf,
    pcap_capture: Box<dyn DataLinkReceiver>,
}

impl PcapFileSource {
    fn new(path: &PathBuf) -> Result<Self, MyError> {
        let config = pnet::datalink::pcap::Config::default();

        Ok(PcapFileSource {
            file_path: path.clone(),
            pcap_capture: match datalink::pcap::from_file(path, config) {
                Ok(Ethernet(_, rx)) => Ok(rx),
                Ok(_) => Err(MyError::Custom(
                    "packetdump: unhandled channel type".to_string(),
                )),
                Err(e) => Err(MyError::Custom(format!(
                    "packetdump: unable to create channel: {}",
                    e
                ))),
            }?,
        })
    }

    fn start(
        mut self,
        send_data: crossbeam_channel::Sender<Vec<u8>>,
    ) -> std::io::Result<std::thread::JoinHandle<()>> {
        std::thread::Builder::new()
            .name("pcap-file-reader".to_string())
            .spawn(move || {
                while let Ok((ts,pkt))= self.pcap_capture.next() {

                }
            })
    }
}
