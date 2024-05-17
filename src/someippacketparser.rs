pub mod someip_packet_parser {

    use self::someip_types::*;
    use crate::someiptypes::*;

    use std::borrow::{Borrow, BorrowMut};
    use std::cell::{Cell, RefCell};
    use std::path::PathBuf;
    use std::time::Duration;

    use pnet::datalink::Channel::{self, Ethernet};
    use pnet::datalink::{self, Config};
    use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
    use pnet::packet::ip::IpNextHeaderProtocols::{self};
    use pnet::packet::ipv6::Ipv6Packet;
    use pnet::packet::sll::SLLPacket;
    use pnet::packet::someip::{SomeipIterable, SomeipMessageTypes, SomeipPacket, SomeipSdPacket};
    use pnet::packet::udp::UdpPacket;
    use pnet::packet::Packet;

    /// 用户不关心用TCP、UDP传输，还是用SomeIP-TP传输
    /// 用户只关心，是不是正常的包，是不是SD包（有没有订阅过程等等）
    /// 其中，ClientID、SessionID、Length等字段，用户是不会关心的
    /// 甚至Return Code也不关心——对于RR类型的操作，只关心什么时候发了Request，什么时候Response（Field的Getter、Setter同理）
    /// 但是，在没有提供原始矩阵表的情况下，只能按照MessageType字段区分上述类型了
    ///


    struct SomeipSingleMessage<'a> {
        index: PacketIndex,
        timestamp: Duration,
        message_type: SomeipSingleMessageType,
        service_id: SomeipServiceId,
        method_id: SomeipMethodId,
        client_id: SomeipClientId,
        session_id: SomeipSessionId,
        transport_protocol: SomeipTransportPortocol,
        raw_packet: Box<&'a [u8]>,
    }

    pub struct PacketParser {
        path: PathBuf,
        pnet_config: Config,
        rx: RefCell<Box<dyn datalink::DataLinkReceiver>>,
    }

    pub fn init_from_path(path: PathBuf) -> PacketParser {
        let mut config = datalink::Config::default();
        PacketParser {
            path: path.clone(),
            pnet_config: config,
            rx: match datalink::pcap::from_file(path, &mut config) {
                Ok(Ethernet(rx)) => rx.into(),
                Ok(_) => panic!("packetdump: unhandled channel type"),
                Err(e) => panic!("packetdump: unable to create channel: {}", e),
            },
        }
    }

    pub fn handle_packet_loop(pp: &PacketParser) {
        let mut index = 0;

        while let Ok((ts, pkt)) = (*pp).rx.borrow_mut().next() {
            raw_packet_parser(pp.borrow(), ts, index, pkt);
            index += 1;
        }
    }

    fn check_if_sd(pkt: &SomeipPacket) -> bool {
        (pkt.get_service_id() == 0xFFFF) && (pkt.get_method_id() == 0x8100)
    }

    fn raw_packet_parser(pp: &PacketParser, ts: &Duration, index: PacketIndex, pkt: &[u8]) {
        let handle_someip_packet = |pkt: SomeipPacket| {
            // 这里需要区分是否是SD包，如果是，那就需要解包看看是什么Entry的SD包
            if !(check_if_sd(&pkt)) {
                let mut tmp = SomeipSingleMessage {
                    timestamp: *ts,
                    service_id: pkt.get_service_id(),
                    method_id: pkt.get_method_id(),
                    client_id: pkt.get_client_id(),
                    session_id: pkt.get_session_id(),
                    message_type: SomeipSingleMessageType::Notification,
                    raw_packet: Box::new(pkt.packet()),
                    index,
                    transport_protocol: todo!(),
                };
                // Normal Someip Message
                tmp.message_type = match pkt.get_message_type() {
                    SomeipMessageTypes::Request => SomeipSingleMessageType::Request,
                    SomeipMessageTypes::Response => SomeipSingleMessageType::Response,
                    SomeipMessageTypes::RequestNoReturn => {
                        SomeipSingleMessageType::RequestWithoutResponse
                    }
                    SomeipMessageTypes::Notification => SomeipSingleMessageType::Notification,
                    // TODO: Someip-TP
                    _ => SomeipSingleMessageType::ResponseWithError,
                };
                // TODO: deserialize
            } else {
                // Someip SD Message
                let sdpkt = SomeipSdPacket::new(pkt.payload()).unwrap();
                let mut iter = sdpkt.get_entries_iter();
                while let Some(entry) = iter.next() {
                    let mut tmp = SomeipSingleMessage {
                        timestamp: *ts,
                        service_id: pkt.get_service_id(),
                        method_id: pkt.get_method_id(),
                        client_id: pkt.get_client_id(),
                        session_id: pkt.get_session_id(),
                        message_type: SomeipSingleMessageType::Notification,
                        raw_packet: Box::new(pkt.packet()),
                        index,
                        transport_protocol: todo!(),
                    };
                    tmp.message_type = SomeipSingleMessageType::SdOffer;
                    tmp.service_id = entry.get_service_id();
                }
                // TODO: return
            }
        };

        let handle_udp_packet = |packet: &[u8]| {
            let pkt = UdpPacket::new(packet).unwrap();
            let mut iter = SomeipIterable::new(pkt.payload());
            // 这里确定收到了一个UDP包，UDP包可能不是SomeIP包，需要先判断合法性
            // 同时，假设这里是一个SomeIP包，也可能是一个SomeIP-SD或者Someip-TP的包，需要进一步分析
            // TODO: Someip-TP
            // 而且，规范中还认为，通过PDU的方式，一条UDP包中可以有多条Someip包，也需要对每个包进行分析
            while let Some(pkt) = iter.next() {
                handle_someip_packet(pkt);
            }
        };

        // 当要传输的Someip包>1400字节且对传输延迟没有要求时，会对包进行分段
        // TODO: Someip-TCP
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
                    println!("index:{}, ts:{:?}, unknown sll packet", index, ts);
                }
            }
        };

        match pp.pnet_config.channel_type {
            datalink::ChannelType::Layer2 => handle_layer2_packet(pkt),
            datalink::ChannelType::Layer3(_) => handle_sll_packet(pkt),
        }
    }
}
