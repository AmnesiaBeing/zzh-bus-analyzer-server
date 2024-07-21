use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::rc::{Rc, Weak};
use std::time::Duration;

pub type SomeipServiceId = u16;
pub type SomeipMethodId = u16;
pub type SomeipClientId = u16;
pub type SomeipSessionId = u16;
pub type SomeipInstantId = u16;
pub type SomeipMajorVersion = u16;
pub type SomeipMinorVersion = u16;

pub type PacketIndex = isize;

pub type Port = u16;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum MatrixServiceMethodType {
    RRMethod(RRMethod),
    FFMethod(FFMethod),
    EVENT(EventMethod),
    FIELD(FieldMethod),
}

// 尽可能内存中只有一份payload的描述，因此都用box类型包裹

#[derive(Debug)]
pub struct RRMethod {
    data_in: Vec<Box<MatrixDataTypeDefinition>>,
    data_out: Box<MatrixDataTypeDefinition>,
}

#[derive(Debug)]
pub struct FFMethod {
    data_in: Vec<Box<MatrixDataTypeDefinition>>,
}

#[derive(Debug)]
pub struct EventMethod {
    data: Vec<Box<MatrixDataTypeDefinition>>,
}

// field类型中，setter、getter、notifier不一定是必须的，因此增加option进行修饰
#[derive(Debug)]
pub struct FieldMethod {
    // setter带payload，并且server的返回值也带payload，表示设置成功
    setter: Option<Box<MatrixDataTypeDefinition>>,
    // getter，client发送不带payload，server返回值带，可以认为是event的变种
    getter: Option<Box<MatrixDataTypeDefinition>>,
    // notifier，没有client，只有server发payload过来
    notifier: Option<Box<MatrixDataTypeDefinition>>,
}

#[derive(Debug)]
pub enum NumberType {
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

#[derive(Debug)]
pub enum StringEncoding {
    UTF8,
    UTF16LE,
    UTF16BE,
}

pub type StringArrayLengthFixed= usize;
pub type StringArrayLengthMin = usize;
pub type StringArrayLengthMax = usize;

#[derive(Debug)]
pub enum StringArrayLength {
    FIXED(StringArrayLengthFixed),
    DYNAMIC(StringArrayLengthMin, StringArrayLengthMax),
}

#[derive(Debug)]
pub struct NumberPayload {
    pub size: NumberType,
    // TODO: Initial/Invalid Value Offset Min Max
}

#[derive(Debug)]
pub struct StringPayload {
    pub length: StringArrayLength,
    pub encoding: StringEncoding,
}

#[derive(Debug)]
pub struct ArrayPayload {
    pub payload: MatrixDataType,
    pub length: StringArrayLength,
}

// #[derive(Debug)]
// pub struct ArrayStuctPayload {
//     pub payload: MatrixDataTypeDefinition,
//     pub length: StringArrayLength,
// }

#[derive(Debug)]
pub struct StructPayload {
    pub payload: Vec<MatrixDataTypeDefinition>,
}

pub type MatrixDataTypeDefinitionName = String;

/// 根据Someip规范，payload的数据类型有且仅有：
/// 1. Integer数值类型：u8,u16,u32,u64,i8,i16,i32,i64,f32,f64
/// 2. String字符串类型：UTF-8,UTF-16LE,UTF-16BE
/// 3. Enumeration枚举类型：可按照u8,u16,u32,u64进行填充
/// 4. Array数组类型：所有可允许类型，可嵌套
/// 5. Struct结构体类型：所有可允许类型，可嵌套
/// 6. Union联合体类型：所有可允许的类型，可嵌套
#[derive(Debug)]
pub enum MatrixDataType {
    Number(NumberPayload),
    String(StringPayload),
    // TODO: Enumeration如何处理，其实很多Integer类型实际是Enumeration类型
    // Enumeration(),
    // 下面的类型需要使用按顺序的类型，否则影响解析
    Array(Box<ArrayPayload>),
    // ArrayStruct(Box<ArrayStuctPayload>),
    Struct(Box<StructPayload>),
    // 实际上在矩阵中并没有使用，先屏蔽处理
    // Union(Vec<MatrixDataTypeDefinition>),
    // 因为存在嵌套结构，这里考虑先读取成一个临时的String-Custom(String)/或MatrixDataTypeDefinition
    // 二轮处理时，如果遇到String，再将其转化为MatrixDataTypeDefinition
    // 这里是临时存储String，二轮处理再转换为上述结构体
    Custom(MatrixDataTypeDefinitionName)
}

#[derive(Debug)]
pub struct MatrixDataTypeDefinition {
    pub name: MatrixDataTypeDefinitionName,
    pub description: String,
    pub payload: MatrixDataType,
}

#[derive(Debug)]
pub struct MatrixServiceMethod {
    service: Weak<MatrixService>,
    method_id: SomeipMethodId,
    method_name: String,
    method_type: MatrixServiceMethodType,
    transport_protocol: SomeipTransportPortocol,
}

#[derive(Debug)]
pub struct MatrixRole {
    pub name: RoleName,
    pub ip_addr: IpAddr,
    // pub mac_addr: IpAddr,
}

pub type RoleName = String;
pub type ServerMatrixRole = MatrixRole;
pub type ServerPort = Port;
pub type ClientMatrixRole = MatrixRole;

#[derive(Debug)]
pub struct MatrixService {
    pub service_id: SomeipServiceId,
    pub service_name: String,
    pub service_description: String,
    pub instance_id: SomeipInstantId,
    pub major_verison: SomeipMajorVersion,
    pub minor_version: SomeipMinorVersion,
    // TODO: 如何存储MatrixServiceMethod，能够通过methodid实现快速查找
    pub methods: HashMap<SomeipMethodId, MatrixServiceMethod>,
    pub server_client: Vec<(Rc<ServerMatrixRole>, ServerPort, Rc<ClientMatrixRole>)>,
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
    // TODO: map by id or name?
    pub service_interfaces: HashMap<SomeipServiceId, MatrixService>,
    pub data_type_definition: HashMap<MatrixDataTypeDefinitionName, Rc<MatrixDataTypeDefinition>>,
    pub serialization_parameter: MatrixSerializationParameter,
    pub matrix_role: HashMap<RoleName, Rc<MatrixRole>>,
}
