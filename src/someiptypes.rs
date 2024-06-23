pub mod someip_types {
    use std::time::Duration;

    pub type SomeipServiceId = u16;
    pub type SomeipMethodId = u16;
    pub type SomeipClientId = u16;
    pub type SomeipSessionId = u16;

    pub type PacketIndex = isize;

    pub enum SomeipTransportPortocol {
        UNDEFINED,
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

    pub(crate) struct SomeipMessage<'a> {
        pub(crate) timestamp: Duration,
        pub(crate) message_type: SomeipMessageType,
        pub(crate) service_id: SomeipServiceId,
        pub(crate) method_id: SomeipMethodId,
        pub(crate) client_id: SomeipClientId,
        pub(crate) session_id: SomeipSessionId,
        pub(crate) return_code: SomeipReturnCode,
        pub(crate) transport_protocol: SomeipTransportPortocol,
        // 注意TCP/UDP-SOMEIP-TP均需要解包出来再生成该结构体
        pub(crate) payload: Box<&'a [u8]>,
    }


}
