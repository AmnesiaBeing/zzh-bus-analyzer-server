use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::rc::{Rc, Weak};

use serde::{Deserialize, Serialize};

use crate::errors::MyError;
use crate::types::{
    ServerPort, SomeipInstanceId, SomeipMajorVersion, SomeipMethodId, SomeipMinorVersion,
    SomeipServiceId, SomeipTransportPortocol,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MatrixServiceMethodFieldType {
    Getter,
    Setter,
    Notifier,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MatrixServiceMethodType {
    RRMethod {
        data_in: Vec<String>,
        #[serde(skip)]
        data_in_ref: Vec<MatrixDataNodeConstRef>,
        data_out: String,
        #[serde(skip)]
        data_out_ref: Option<MatrixDataNodeConstRef>,
    },
    FFMethod {
        data_in: Vec<String>,
        #[serde(skip)]
        data_in_ref: Vec<MatrixDataNodeConstRef>,
    },
    EVENT {
        data_out: String,
        #[serde(skip)]
        data_out_ref: Option<MatrixDataNodeConstRef>,
    },
    FIELD {
        field_type: MatrixServiceMethodFieldType,
        data: String,
        #[serde(skip)]
        data_ref: Option<MatrixDataNodeConstRef>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
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

impl TryFrom<String> for NumberType {
    type Error = MyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "boolean" => NumberType::Boolean,
            "uint8" => NumberType::Uint8,
            "uint16" => NumberType::Uint16,
            "uint32" => NumberType::Uint32,
            "uint64" => NumberType::Uint64,
            "sint8" => NumberType::Sint8,
            "sint16" => NumberType::Sint16,
            "sint32" => NumberType::Sint32,
            "sint64" => NumberType::Sint64,
            "float" => NumberType::Float32,
            "double" => NumberType::Float64,
            _ => return Err(MyError::Custom("parse number type error.".to_owned())),
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(untagged)]
pub enum StringEncoding {
    #[default]
    UTF8,
    UTF16LE,
    UTF16BE,
}

pub type StringArrayLengthFixed = usize;
pub type StringArrayLengthMin = usize;
pub type StringArrayLengthMax = usize;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringArrayLength {
    FIXED(StringArrayLengthFixed),
    DYNAMIC(StringArrayLengthMin, StringArrayLengthMax),
}

impl Default for StringArrayLength {
    fn default() -> Self {
        Self::FIXED(StringArrayLengthFixed::default())
    }
}

/// 根据Someip规范，payload的数据类型有且仅有：
/// 1. Integer数值类型：u8,u16,u32,u64,i8,i16,i32,i64,f32,f64
/// 2. String字符串类型：UTF-8,UTF-16LE,UTF-16BE
/// 3. Enumeration枚举类型：可按照u8,u16,u32,u64进行填充
/// 4. Array数组类型：所有可允许类型，可嵌套
/// 5. Struct结构体类型：所有可允许类型，可嵌套
/// 6. Union联合体类型：所有可允许的类型，可嵌套
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MatrixType {
    // TODO: Enumeration如何处理，其实很多Integer类型实际是Enumeration类型
    // 对于Number类型，还有逻辑值与物理值之间的映射关系
    Number {
        size: NumberType,
    },
    String {
        length: StringArrayLength,
        encoding: StringEncoding,
    },
    Array {
        length: StringArrayLength,
        member: MatrixMember
    },
    Struct {
        members: Vec<MatrixMember>
    },
    #[default]
    Unimplemented,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(tag = "member")]
pub struct MatrixMember {
    pub member_name: String,
    pub member_description: String,
    #[serde(skip)]
    pub member_ref: Option<MatrixDataNodeConstRef>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MatrixDataNode {
    pub name: String,
    pub description: String,
    #[serde(flatten)]
    pub data_type: MatrixType,
}

pub type MatrixDataNodeRef<'a> = Option<&'a mut MatrixDataNode>;
pub type MatrixDataNodeConstRef = *const MatrixDataNode;
// pub type MatrixDataNodeWeakRef = Weak<RefCell<MatrixDataNode>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct MatrixServiceMethod {
    method_id: SomeipMethodId,
    method_name: String,
    method_type: MatrixServiceMethodType,
    transport_protocol: SomeipTransportPortocol,
    #[serde(skip)]
    mother_service_ref: Weak<MatrixService>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MatrixRole {
    pub name: RoleName,
    pub ip_addr: IpAddr,
    pub mac_addr: [u8; 6],
}

pub type MatrixRoleRef = *const MatrixRole;

impl Default for MatrixRole {
    fn default() -> Self {
        Self {
            name: Default::default(),
            ip_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            mac_addr: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct MatrixRoleServerClientPair {
    pub server: String,
    #[serde(skip)]
    pub server_ref: Option<MatrixRoleRef>,
    pub server_port: ServerPort,
    pub client: String,
    #[serde(skip)]
    pub client_ref: Option<MatrixRoleRef>,
}

impl PartialEq for MatrixRoleServerClientPair {
    fn eq(&self, other: &Self) -> bool {
        self.server == other.server
            && self.server_port == other.server_port
            && self.client == other.client
        // && self.server_ref.is_some()
        // && self.client_ref.is_some()
        // && other.server_ref.is_some()
        // && other.client_ref.is_some()
        // && core::ptr::addr_eq(self.server_ref.unwrap(), other.server_ref.unwrap())
        // && core::ptr::addr_eq(self.server_ref.unwrap(), other.server_ref.unwrap())
    }
}

pub type RoleName = String;

#[derive(Debug, Deserialize, Serialize)]
pub struct MatrixService {
    pub service_id: SomeipServiceId,
    pub service_name: String,
    pub service_description: String,
    pub instance_id: SomeipInstanceId,
    pub major_verison: SomeipMajorVersion,
    pub minor_version: SomeipMinorVersion,
    pub methods: HashMap<SomeipMethodId, MatrixServiceMethod>,
    pub server_client: RefCell<Vec<MatrixRoleServerClientPair>>,
}

pub type MatrixServiceRef = Rc<MatrixService>;

#[derive(Debug, Deserialize, Serialize, Default)]
pub enum MatrixSerializationParameterSize {
    #[default]
    B8,
    B16,
    B32,
    B64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Matrix {
    pub version: String,
    pub services: HashMap<SomeipServiceId, MatrixService>,
    #[serde(skip)]
    pub services_map_by_name: HashMap<String, MatrixServiceRef>,
    pub data_types: HashMap<String, MatrixDataNode>,
    pub serialization_parameter: MatrixSerializationParameter,
    pub roles: HashMap<RoleName, MatrixRole>,
}
