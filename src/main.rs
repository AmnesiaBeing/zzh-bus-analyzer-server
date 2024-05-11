use std::path::PathBuf;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::sll::SLLPacket;
use pnet::packet::someip::{SomeipIterable, SomeipMessageTypes, SomeipPacket, SomeipSdPacket};
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

extern crate clap;
use clap::Parser;

type SomeipServiceId = u16;
type SomeipMethodId = u16;
type SomeipClientId = u16;
type SomeipSessionId = u16;

/// 用户不关心用TCP、UDP传输，还是用SomeIP-TP传输
/// 用户只关心，是不是正常的包，是不是SD包（有没有订阅过程等等）
/// 其中，ClientID、SessionID、Length等字段，用户是不会关心的
/// 甚至Return Code也不关心——对于RR类型的操作，只关心什么时候发了Request，什么时候Response（Field的Getter、Setter同理）
/// 但是，在没有提供原始矩阵表的情况下，只能按照MessageType字段区分上述类型了
#[derive(Debug)]
enum SomeipTransportMessageType {
    Request,
    RequestWithoutResponse,
    Response,
    ResponseWithError,
    Notification,
    SdOffer,
    SdSubscribe,
    SdSubscribeAck,
}

struct SomeipTransportMessage<'a> {
    timestamp: Duration,
    message_type: SomeipTransportMessageType,
    service_id: SomeipServiceId,
    method_id: SomeipMethodId,
    client_id: SomeipClientId,
    session_id: SomeipSessionId,
    raw_packet: Box<&'a [u8]>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 后台运行，持续读取指定网卡报文
    #[arg(short, long)]
    daemon: bool,

    /// 矩阵文件
    #[arg(short, long)]
    matrix: Option<PathBuf>,

    /// 需要筛选的serivceid和methodid，用:间隔，支持十六进制和十进制，如果加载了矩阵文件，可以支持信号名
    /// methodid支持不使用:间隔，表示筛选所有信号
    #[arg(short, long)]
    signals: String,

    /// 回放报文文件
    #[arg(short, long)]
    file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    // TODO: 针对signals进行处理，筛选出要匹配的服务

    let mut config = datalink::Config::default();
    let mut rx = match datalink::pcap::from_file(args.file.unwrap(), &mut config) {
        Ok(Ethernet(rx)) => rx,
        Ok(_) => panic!("packetdump: unhandled channel type"),
        Err(e) => panic!("packetdump: unable to create channel: {}", e),
    };

    let mut index = 0;

    let mut ret: Vec<SomeipTransportMessage> = vec![];

    // 最大是0xFFFF，用u16即可
    let filter_sid = u16::from_str_radix(&args.signals[2..], 16).unwrap();

    while let Ok((ts, pkt)) = rx.next() {
        let handle_someip_packet = |pkt: SomeipPacket| {
            // 这里需要区分是否是SD包，如果是，那就需要解包看看是什么Entry的SD包
            if !((pkt.get_service_id() == 0xFFFF) && (pkt.get_method_id() == 0x8100)) {
                let mut tmp = SomeipTransportMessage {
                    timestamp: *ts,
                    service_id: pkt.get_service_id(),
                    method_id: pkt.get_method_id(),
                    client_id: pkt.get_client_id(),
                    session_id: pkt.get_session_id(),
                    message_type: SomeipTransportMessageType::Notification,
                    raw_packet: Box::new(pkt.packet()),
                };
                // Normal Someip Message
                tmp.message_type = match pkt.get_message_type() {
                    SomeipMessageTypes::Request => SomeipTransportMessageType::Request,
                    SomeipMessageTypes::Response => SomeipTransportMessageType::Response,
                    SomeipMessageTypes::RequestNoReturn => {
                        SomeipTransportMessageType::RequestWithoutResponse
                    }
                    SomeipMessageTypes::Notification => SomeipTransportMessageType::Notification,
                    // TODO: Someip-TP
                    _ => SomeipTransportMessageType::ResponseWithError,
                };
                if (tmp.service_id == filter_sid) {
                    println!(
                        "{} 0x{:X}:0x{:X} type:{:?} payload:{:?}",
                        Utc.timestamp_opt(ts.as_secs() as i64, 0)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S"),
                        tmp.service_id,
                        tmp.method_id,
                        tmp.message_type,
                        tmp.raw_packet
                    );
                }
            } else {
                // Someip SD Message
                let sdpkt = SomeipSdPacket::new(pkt.payload()).unwrap();
                let mut iter = sdpkt.get_entries_iter();
                while let Some(entry) = iter.next() {
                    let mut tmp = SomeipTransportMessage {
                        timestamp: *ts,
                        service_id: pkt.get_service_id(),
                        method_id: pkt.get_method_id(),
                        client_id: pkt.get_client_id(),
                        session_id: pkt.get_session_id(),
                        message_type: SomeipTransportMessageType::Notification,
                        raw_packet: Box::new(pkt.packet()),
                    };
                    tmp.message_type = SomeipTransportMessageType::SdOffer;
                    tmp.service_id = entry.get_service_id();
                    if (tmp.service_id == filter_sid) {
                        println!(
                            "{} 0x{:X}:0x{:X} type:{:?}",
                            Utc.timestamp_opt(ts.as_secs() as i64, 0)
                                .unwrap()
                                .format("%Y-%m-%d %H:%M:%S"),
                            tmp.service_id,
                            tmp.method_id,
                            tmp.message_type,
                        );
                    }
                }
            }
            // ret.push(tmp);
        };

        let handle_udp_packet = |packet: &[u8]| {
            let pkt = UdpPacket::new(packet).unwrap();
            let mut iter = SomeipIterable::new(pkt.payload());
            // 这里确定收到了一个UDP包，UDP包可能不是SomeIP包，需要先判断合法性
            // 同时，假设这里是一个SomeIP包，也可能是一个SomeIP-SD或者Someip-TP的包，需要进一步分析
            // 而且，规范中还认为，通过PDU的方式，一条UDP包中可以有多条Someip包，也需要
            while let Some(pkt) = iter.next() {
                handle_someip_packet(pkt);
            }
        };

        // 当要传输的Someip包>1400字节且对传输延迟没有要求时，会对包进行分段
        let handle_tcp_packet = |_packet: &[u8]| {};

        let handle_ipv4_packet = |packet: &[u8]| {
            let pkt = pnet::packet::ipv4::Ipv4Packet::new(packet).unwrap();
            match pkt.get_next_level_protocol() {
                IpNextHeaderProtocols::Udp => handle_udp_packet(pkt.payload()),
                IpNextHeaderProtocols::Tcp => handle_tcp_packet(pkt.payload()),
                _ => {}
            }
        };

        let handle_ipv6_packet = |packet: &[u8]| {
            let pkt = Ipv6Packet::new(packet).unwrap();
            match pkt.get_next_header() {
                IpNextHeaderProtocols::Udp => handle_udp_packet(pkt.payload()),
                IpNextHeaderProtocols::Tcp => handle_tcp_packet(pkt.payload()),
                _ => {}
            }
        };

        let handle_layer2_packet = |packet: &[u8]| {
            let pkt = EthernetPacket::new(&packet).unwrap();
            match pkt.get_ethertype() {
                EtherTypes::Ipv4 => handle_ipv4_packet(pkt.payload()),
                EtherTypes::Ipv6 => handle_ipv6_packet(pkt.payload()),
                _ => {
                    println!("index:{}, ts:{:?}, unknown layer2 packet", index, ts);
                }
            }
        };

        let handle_sll_packet = |packet: &[u8]| {
            let pkt = SLLPacket::new(&packet).unwrap();
            match pkt.get_protocol() {
                EtherTypes::Ipv4 => handle_ipv4_packet(pkt.payload()),
                EtherTypes::Ipv6 => handle_ipv6_packet(pkt.payload()),
                _ => {
                    // println!("index:{}, ts:{:?}, unknown sll packet", index, ts);
                }
            }
        };

        match config.channel_type {
            datalink::ChannelType::Layer2 => handle_layer2_packet(pkt),
            datalink::ChannelType::Layer3(_) => handle_sll_packet(pkt),
        }

        index += 1;
    }
}
