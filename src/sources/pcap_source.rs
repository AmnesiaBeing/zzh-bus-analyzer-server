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

use super::pnet_packet_someip::{SomeipIterable, SomeipPacket};
use super::{DataSource, DataSourceConfig, DataSourceReceiver};

/// 用户不关心用TCP、UDP传输，还是用SomeIP-TP传输
/// 用户只关心，是不是正常的包，是不是SD包（有没有订阅过程等等）
/// 其中，ClientID、SessionID、Length等字段，用户是不会关心的
/// 甚至Return Code也不关心——对于RR类型的操作，只关心什么时候发了Request，什么时候Response（Field的Getter、Setter同理）
/// 但是，在没有提供原始矩阵表的情况下，只能按照MessageType字段区分上述类型了

struct PcapFileReceiverImpl {
    capture: Box<dyn DataLinkReceiver>,
    read_buffer: Vec<u8>,
}

impl DataSource {
    pub fn init_from_path<'a>(path: PathBuf) -> Result<Self, MyError> {
        let mut config = pnet::datalink::pcap::Config::default();
        match datalink::pcap::from_file(path, config) {
            Ok(Ethernet(_, rx)) => Ok(DataSource {
                config: DataSourceConfig::PcapFile { file_path: path },
                rx: Box::new(PcapFileReceiverImpl {
                    capture: rx,
                    read_buffer: Vec::new(),
                }),
            }),
            Ok(_) => Err("packetdump: unhandled channel type"),
            Err(e) => Err(&format!("packetdump: unable to create channel: {}", e)),
        }
    }
}

impl DataSourceReceiver for PcapFileReceiverImpl {
    fn next(&mut self) -> std::io::Result<&[u8]> {
        let mut cap = self.capture.lock().unwrap();
        match cap.next_packet() {
            Ok(pkt) => {
                self.read_buffer.truncate(0);
                self.read_buffer.extend(pkt.data);
            }
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };
        Ok(&self.read_buffer)
    }
}

pub fn handle_packet_loop(pp: &PacketParser) {
    while let Ok((ts, pkt)) = (*pp).rx.borrow_mut().next() {
        raw_packet_parser(pp.borrow(), ts, pkt);
    }
}

fn check_is_sd(pkt: &SomeipPacket) -> bool {
    (pkt.get_service_id() == 0xFFFF) && (pkt.get_method_id() == 0x8100)
}

fn handle_someip_sd_packet<'a>(pp: &'a PacketParser, ts: &'a Duration, pkt: &'a SomeipPacket) {}

// 确保只有1个原始的someip包的时候才到这里，pkt是一个原始的someip包——可以不完整，可以是一个TP包
fn handle_raw_someip_packet<'a>(pp: &'a PacketParser, ts: &'a Duration, pkt: &'a SomeipPacket) {
    if check_is_sd(pkt) {
        handle_someip_sd_packet(pp, ts, pkt);
    } else {
        if pkt.get_message_type().check_is_tp() {
            // TODO:
            error!("Unhandled TP Message.");
        } else {
            // handle_someip_packet(pp, ts, pkt);
        }
    }
}

fn handle_tcp_packet<'a>(pp: &'a PacketParser, ts: &'a Duration, pkt: &'a [u8]) {
    // 这里确定收到了一个TCP包，TCP包大概率不是SomeIP包，但是需要对TCP数据流进行判断，试图找出里面的TCP-SOMEIP包
    // 将所有TCP连接按照IP-PORT组合进行划分，不同类型的包设置缓冲区？
    // TODO: 还没想好怎么写tcp中筛选someip包
}

fn handle_udp_packet<'a>(pp: &'a PacketParser, ts: &'a Duration, pkt: &'a [u8]) {
    let pkt = UdpPacket::new(pkt).unwrap();
    let mut iter = SomeipIterable::new(pkt.payload());
    // 这里确定收到了一个UDP包，UDP包可能不是SomeIP包，需要先判断合法性
    // 而且，规范中还认为，通过PDU的方式，一条UDP包中可以有多条Someip包，也需要对当前收到的包进行判断，是否可能是一个子包
    // 尽可能从里面筛选出单独的SomeIP包出来，包括SD包
    while let Some(pkt) = iter.next() {
        handle_raw_someip_packet(pp, ts, &pkt);
    }
}

fn raw_packet_parser<'a>(pp: &'a PacketParser, ts: &'a Duration, pkt: &'a [u8]) {
    let handle_ipv4_packet = |packet: &[u8]| {
        let pkt = pnet::packet::ipv4::Ipv4Packet::new(packet).unwrap();
        match pkt.get_next_level_protocol() {
            IpNextHeaderProtocols::Udp => handle_udp_packet(pp, ts, pkt.payload()),
            IpNextHeaderProtocols::Tcp => handle_tcp_packet(pp, ts, pkt.payload()),
            _ => {
                error!("ts:{:?}, unknown layer3 packet", ts);
            }
        }
    };

    let handle_ipv6_packet = |packet: &[u8]| {
        let pkt = pnet::packet::ipv6::Ipv6Packet::new(packet).unwrap();
        match pkt.get_next_header() {
            IpNextHeaderProtocols::Udp => handle_udp_packet(pp, ts, pkt.payload()),
            IpNextHeaderProtocols::Tcp => handle_tcp_packet(pp, ts, pkt.payload()),
            _ => {
                error!("ts:{:?}, unknown layer3 packet", ts);
            }
        }
    };

    let handle_layer2_packet = |packet: &[u8]| {
        let pkt = EthernetPacket::new(&packet).unwrap();
        match pkt.get_ethertype() {
            EtherTypes::Ipv4 => handle_ipv4_packet(pkt.payload()),
            EtherTypes::Ipv6 => handle_ipv6_packet(pkt.payload()),
            _ => {
                error!("ts:{:?}, unknown layer2 packet", ts);
            }
        }
    };

    let handle_sll_packet = |packet: &[u8]| {
        let pkt = SLLPacket::new(&packet).unwrap();
        match pkt.get_protocol() {
            EtherTypes::Ipv4 => handle_ipv4_packet(pkt.payload()),
            EtherTypes::Ipv6 => handle_ipv6_packet(pkt.payload()),
            _ => {
                error!("ts:{:?}, unknown sll packet", ts);
            }
        }
    };

    match pp.pnet_config.channel_type {
        datalink::ChannelType::Layer2 => handle_layer2_packet(pkt),
        datalink::ChannelType::Layer3(_) => handle_sll_packet(pkt),
        _ => {
            error!("ts:{:?}, unknown channel type", ts);
        }
    }
}
