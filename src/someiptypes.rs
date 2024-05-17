pub mod someip_types {
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

    // 基础SomeIP消息类型，这里涵盖了服务发现的报文类型
    // 对于SomeIP-TP类型，不包含在此处，自动解包至单个SomeIP包
    pub enum SomeipMessageType {
        Request,
        RequestWithoutResponse,
        Response,
        ResponseWithError,
        Notification,
        SdOffer,
        SdSubscribe,
        SdSubscribeAck,
    }

}
