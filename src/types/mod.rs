use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::default;
use std::net::IpAddr;
use std::rc::{Rc, Weak};
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub type SomeipServiceId = u16;
pub type SomeipMethodId = u16;
pub type SomeipClientId = u16;
pub type SomeipSessionId = u16;
pub type SomeipInstanceId = u16;
pub type SomeipMajorVersion = u16;
pub type SomeipMinorVersion = u16;

pub type PacketIndex = isize;

pub type Port = u16;
pub type ServerPort = Port;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SomeipTransportPortocol {
    TCP,
    UDP,
}

pub type SomeipReturnCode = u8;

/// 基础SomeIP消息类型，这里涵盖了服务发现的报文类型
/// 对于SomeIP-TP类型，不包含在此处，自动解包至单个SomeIP包
/// 设计上不考虑显示最最原始的报文，只显示收到/发送的报文类型
pub enum SomeipMessageType {
    Request,
    RequestWithoutResponse,
    Response,
    ResponseWithError,
    Notification,
    ServiceOffer, // 实际上所有Sd报文都是Notifiction
    ServiceSubscribe,
    ServiceSubscribeAck,
}

pub struct SomeipMessage<'a> {
    pub timestamp: Duration,
    pub message_type: SomeipMessageType,
    pub service_id: SomeipServiceId,
    pub method_id: SomeipMethodId,
    pub client_id: SomeipClientId,
    pub session_id: SomeipSessionId,
    pub return_code: SomeipReturnCode,
    pub transport_protocol: SomeipTransportPortocol,
    // 注意TCP/UDP-SOMEIP-TP均需要解包出来再生成该结构体
    pub payload: Box<&'a [u8]>,
}
