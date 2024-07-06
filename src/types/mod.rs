
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::time::Duration;

pub type SomeipServiceId = u16;
pub type SomeipMethodId = u16;
pub type SomeipClientId = u16;
pub type SomeipSessionId = u16;
pub type SomeipInstantId = u16;
pub type SomeipMajorVersion = u16;
pub type SomeipMinorVersion = u16;

pub type PacketIndex = isize;

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

// ------- For Matrix 下面的结构体适用于描述Someip矩阵

pub enum MatrixServiceMethodType {
    RRMethod(RRMethod),
    FFMethod(FFMethod),
    EVENT(EventMethod),
    FIELD(FieldMethod),
}

pub struct MatrixDataTypeDefinition {
    pub name: String,
    pub description: String,
}

// 尽可能内存中只有一份payload的描述，因此都用box类型包裹

pub struct RRMethod {
    data_in: Vec<Box<MatrixPayload>>,
    data_out: Box<MatrixPayload>,
}

pub struct FFMethod {
    data_in: Vec<Box<MatrixPayload>>,
}

pub struct EventMethod {
    data: Vec<Box<MatrixPayload>>,
}

// field类型中，setter、getter、notifier不一定是必须的，因此增加option进行修饰
pub struct FieldMethod {
    // setter带payload，并且server的返回值也带payload，表示设置成功
    setter: Option<Box<MatrixPayload>>,
    // getter，client发送不带payload，server返回值带，可以认为是event的变种
    getter: Option<Box<MatrixPayload>>,
    // notifier，没有client，只有server发payload过来
    notifier: Option<Box<MatrixPayload>>,
}

pub enum NumberSize {
    Boolean,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Sint8,
    Sint16,
    Sint32,
    Sint64,
    Float32,
    Float64,
}

pub enum StringEncoding {
    UTF8,
    UTF16LE,
    UTF16BE,
}

/// 根据Someip规范，payload的数据类型有且仅有：
/// 1. Integer数值类型：u8,u16,u32,u64,i8,i16,i32,i64,f32,f64
/// 2. String字符串类型：UTF-8,UTF-16LE,UTF-16BE
/// 3. Enumeration枚举类型：可按照u8,u16,u32,u64进行填充
/// 4. Array数组类型：所有可允许类型，可嵌套
/// 5. Struct结构体类型：所有可允许类型，可嵌套
/// 6. Union联合体类型：所有可允许的类型，可嵌套
pub enum MatrixPayload {
    Number(NumberSize),
    String(StringEncoding),
    Array(Vec<MatrixPayload>),
    Struct(Vec<MatrixPayload>),
    Union(Vec<MatrixPayload>),
}

pub struct MatrixServiceMethod {
    method_id: SomeipMethodId,
    method_name: String,
    method_type: MatrixServiceMethodType,
    transport_protocol: SomeipTransportPortocol,
}

pub struct MatrixRole {
    name: String,
    ip_addr: IpAddr,
    ip_netmask: IpAddr,
    port: u16,
}

pub struct MatrixService {
    pub service_id: SomeipServiceId,
    pub service_name: String,
    pub service_description: String,
    pub instance_id: SomeipInstantId,
    pub major_verison: SomeipMajorVersion,
    pub minor_version: SomeipMinorVersion,
    pub methods: Vec<MatrixServiceMethod>,
    // 应该不会有矩阵不同服务的相同server、client地址不同吧？
    pub server: MatrixRole,
    pub client: MatrixRole,
    // TODO: 考虑是否要增加：vlan? remark?
}

pub enum MatrixSerializationParameterSize {
    B8,
    B16,
    B32,
    B64,
}

pub struct MatrixSerializationParameter {
    pub alignment: MatrixSerializationParameterSize,
    pub padding_for_fix_length: bool,
    pub length_field_for_struct: bool,
    pub tag_for_serialization: bool,
    pub string_encoding: StringEncoding,
    pub struct_length_field_size: MatrixSerializationParameterSize,
    pub string_length_field_size: MatrixSerializationParameterSize,
    pub array_length_field_size: MatrixSerializationParameterSize,
    pub union_length_field_size: MatrixSerializationParameterSize,
    pub union_type_selector_field_size: MatrixSerializationParameterSize,
    pub union_null: bool,
}

pub struct Matrix {
    pub version: String,
    pub service_interfaces: HashMap<SomeipServiceId, MatrixService>,
    pub data_type_definition: Vec<MatrixDataTypeDefinition>,
    pub serialization_parameter: MatrixSerializationParameterSize,
    // TODO: e2e_protection?
}
