/// 从Excel或者json文件中加载一个矩阵
/// TODO: load/save json
pub mod matrix_loader {

    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::Path;
    use std::rc::{Rc, Weak};

    use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};
    use log::{debug, error, info, trace};
    use serde::{de, Deserialize, Deserializer, Serialize};

    use crate::types::{
        ServerPort, SomeipInstanceId, SomeipMajorVersion, SomeipMethodId, SomeipMinorVersion,
        SomeipServiceId, SomeipTransportPortocol,
    };

    // ------- For Matrix 下面的结构体适用于描述Someip矩阵
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
            data_in_ref: Vec<MatrixDataNodeRef>,
            data_out: String,
            #[serde(skip)]
            data_out_ref: MatrixDataNodeRef,
        },
        FFMethod {
            data_in: Vec<String>,
            #[serde(skip)]
            data_in_ref: Vec<MatrixDataNodeRef>,
        },
        EVENT {
            data_out: String,
            #[serde(skip)]
            data_out_ref: MatrixDataNodeRef,
        },
        FIELD {
            field_type: MatrixServiceMethodFieldType,
            data: String,
            #[serde(skip)]
            data_ref: MatrixDataNodeRef,
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

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum StringEncoding {
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
            children: String,
            #[serde(skip)]
            children_ref: MatrixDataNodeWeakRef,
        },
        Struct {
            children: Vec<String>,
            #[serde(skip)]
            children_refs: Vec<MatrixDataNodeWeakRef>,
        },
        #[default]
        Unimplemented,
    }

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct MatrixDataNode {
        pub name: String,
        pub description: String,
        #[serde(flatten)]
        pub data_type: MatrixType,
    }

    pub type MatrixDataNodeRef = Rc<RefCell<MatrixDataNode>>;
    pub type MatrixDataNodeWeakRef = Weak<RefCell<MatrixDataNode>>;

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

    impl Default for MatrixRole {
        fn default() -> Self {
            Self {
                name: Default::default(),
                ip_addr: IpAddr::V4(),
                mac_addr: Default::default(),
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MatrixRoleServerClientPair {
        server: String,
        #[serde(skip)]
        server_ref: Rc<ServerMatrixRole>,
        server_port: ServerPort,
        client: String,
        #[serde(skip)]
        client_ref: Rc<ClientMatrixRole>,
    }

    pub type RoleName = String;
    pub type ServerMatrixRole = MatrixRole;
    pub type ClientMatrixRole = MatrixRole;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MatrixService {
        pub service_id: SomeipServiceId,
        pub service_name: String,
        pub service_description: String,
        pub instance_id: SomeipInstanceId,
        pub major_verison: SomeipMajorVersion,
        pub minor_version: SomeipMinorVersion,
        pub methods: HashMap<SomeipMethodId, MatrixServiceMethod>,
        pub server_client: Vec<MatrixRoleServerClientPair>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum MatrixSerializationParameterSize {
        B8,
        B16,
        B32,
        B64,
    }

    #[derive(Debug, Deserialize, Serialize)]
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
        pub data_type_definition: HashMap<String, MatrixDataNodeRef>,
        pub serialization_parameter: MatrixSerializationParameter,
        pub matrix_role: HashMap<RoleName, Rc<MatrixRole>>,
    }

    fn deserialize_hex<'de, D>(deserializer: D) -> Result<u16, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        u16::from_str_radix(s.trim_start_matches("0x"), 16).map_err(de::Error::custom)
    }

    fn deserialize_empty_or_hex<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Option<String> = Option::deserialize(deserializer)?;

        match value {
            Some(s) if s.is_empty() => Ok(None), // Handle empty string as None
            Some(s) if s.starts_with("0x") => {
                // Parse hexadecimal number
                u16::from_str_radix(&s[2..], 16)
                    .map(Some)
                    .map_err(de::Error::custom)
            }
            Some(s) => {
                // Parse as regular u32
                s.parse::<u16>().map(Some).map_err(de::Error::custom)
            }
            None => Ok(None),
        }
    }

    #[derive(Deserialize)]
    struct DeploymentRecord {
        #[serde(rename = "Service InterFace Name")]
        service_interface_name: String,
        #[serde(rename = "Service ID", deserialize_with = "deserialize_hex")]
        service_id: u16,
        #[serde(rename = "Instance ID", deserialize_with = "deserialize_hex")]
        instance_id: u16,
        #[serde(rename = "Major Version", deserialize_with = "deserialize_hex")]
        major_version: u16,
        #[serde(rename = "Minor Version", deserialize_with = "deserialize_hex")]
        minor_version: u16,
        #[serde(rename = "Server")]
        server: String,
        #[serde(rename = "Server IP")]
        server_ip: String,
        #[serde(rename = "Server Port")]
        server_port: u16,
        #[serde(rename = "Client")]
        client: String,
        #[serde(rename = "Client IP")]
        client_ip: String,
    }

    #[derive(Deserialize)]
    struct DataTypeDefinitionRecord {
        #[serde(rename = "Parameter Data Type Name")]
        parameter_data_type_name: Option<String>,
        #[serde(rename = "DataType Description")]
        data_type_description: Option<String>,
        #[serde(rename = "Data Category")]
        data_category: Option<String>,
        #[serde(rename = "String/Array Length Type")]
        string_array_length_type: Option<String>, // fixed or dynamic or invalid
        #[serde(rename = "String/Array Length Min")]
        string_array_length_min: Option<String>,
        #[serde(rename = "String/Array Length Max")]
        string_array_length_max: Option<String>,
        #[serde(rename = "Member Name")]
        member_name: Option<String>,
        #[serde(rename = "Member Description")]
        member_description: Option<String>,
        #[serde(rename = "Member Datatype Reference")]
        member_data_type_reference: Option<String>,
        #[serde(rename = "Datatype")]
        data_type: Option<String>,
        // #[serde(rename = "Resolution")]
        // resolution: String,
        // #[serde(rename = "Offset")]
        // offset: String,
        // #[serde(rename = "Physical Min")]
        // physical_min: String,
        // #[serde(rename = "Physical Max")]
        // physical_max: String,
        // #[serde(rename = "Initial Value")]
        // initial_value: String,
        // #[serde(rename = "Invalid Value")]
        // invalid_value: String,
        // #[serde(rename = "Unit")]
        // unit: String,
        // #[serde(rename = "Discrete Value Defination")]
        // discrete_value_defination: String,
    }

    #[derive(Deserialize)]
    struct ServiceInterfacesRecord {
        #[serde(rename = "Service InterFace Name")]
        service_interface_name: String,
        #[serde(rename = "Service ID", deserialize_with = "deserialize_empty_or_hex")]
        service_id: Option<u16>,
        #[serde(rename = "Service Description")]
        service_description: String,
        // #[serde(rename = "Method/Event/Field")]
        // method_event_field: String,
        // #[serde(rename = "Setter/Getter/Notifier")]
        // setter_getter_notifier: String,
        // #[serde(rename = "Element Name")]
        // element_name: String,
        // #[serde(rename = "Element Description")]
        // element_description: String,
        // #[serde(rename = "Method ID/Event ID", deserialize_with = "deserialize_hex")]
        // method_id: u16,
        // #[serde(rename = "Eventgroup Name")]
        // eventgroup_name: String,
        // #[serde(rename = "Eventgroup ID")]
        // eventgroup_id: String,
        // #[serde(rename = "Send Strategy")]
        // send_strategy: String,
        // #[serde(rename = "Cyclic Time (ms)")]
        // cyclic_time_ms: Option<usize>,
        // #[serde(rename = "Parameter Name")]
        // parameter_name: String,
        // #[serde(rename = "IN/OUT")]
        // in_out: String,
        // #[serde(rename = "Parameter Description")]
        // parameter_description: String,
        // #[serde(rename = "Parameter Data Type")]
        // parameter_data_type: String,
        // #[serde(rename = "UDP/TCP")]
        // udp_tcp: String,
        // #[serde(rename = "AutoSAR E2E Protection (Profile 6)")]
        // e2e_protection: String,
    }

    impl Matrix {
        fn from_excel_file<P>(path: P) -> Result<Matrix, Box<dyn std::error::Error>>
        where
            P: AsRef<Path>,
        {
            let mut wb: Xlsx<_> = open_workbook(path)?;
            let version = wb
                .worksheet_range("Cover")
                .unwrap()
                .get_value((6, 0))
                .unwrap()
                .to_string()
                .strip_prefix("Version:")
                .unwrap()
                .to_string();
            info!("{:?}", version);

            // Fill Services
            let range = wb.worksheet_range("Deployment").unwrap();
            let iter_records =
                RangeDeserializerBuilder::with_deserialize_headers::<DeploymentRecord>()
                    .from_range(&range)?;

            let mut services: HashMap<SomeipServiceId, MatrixService> = HashMap::new();
            let mut roles: HashMap<String, Rc<MatrixRole>> = HashMap::new();

            for result in iter_records {
                let record: DeploymentRecord = result?;

                // 获取或插入 server_role 和 client_role
                let server_role = roles
                    .entry(record.server.clone())
                    .or_insert_with(|| {
                        Rc::new(ServerMatrixRole {
                            name: record.server.clone(),
                            ip_addr: record
                                .server_ip
                                .parse()
                                .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                        })
                    })
                    .clone();

                let client_role = roles
                    .entry(record.client.clone())
                    .or_insert_with(|| {
                        Rc::new(ClientMatrixRole {
                            name: record.client.clone(),
                            ip_addr: record
                                .client_ip
                                .parse()
                                .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                        })
                    })
                    .clone();

                let service = services
                    .entry(record.service_id)
                    .or_insert_with(|| MatrixService {
                        service_id: record.service_id,
                        service_name: record.service_interface_name.clone(),
                        service_description: "".to_string(),
                        instance_id: record.instance_id,
                        major_verison: record.major_version,
                        minor_version: record.minor_version,
                        methods: HashMap::new(),
                        server_client: vec![(
                            Rc::clone(&server_role),
                            record.server_port.clone(),
                            Rc::clone(&client_role),
                        )],
                    });

                // 如果已经存在的 service，需要添加新的 server_client 对
                if !service
                    .server_client
                    .iter()
                    .any(|(s, _, c)| Rc::ptr_eq(s, &server_role) && Rc::ptr_eq(c, &client_role))
                {
                    service.server_client.push((
                        server_role.clone(),
                        record.server_port.clone(),
                        client_role.clone(),
                    ));
                }
            }

            info!("Fill Services Completed.");

            // Fill Data Type
            let range = wb.worksheet_range("DataTypeDefinition").unwrap();
            let iter_records =
                RangeDeserializerBuilder::with_deserialize_headers::<DataTypeDefinitionRecord>()
                    .from_range(&range)?;

            let mut data_type_definitions: HashMap<String, MatrixDataNodeRef> = HashMap::new();
            let mut last_key: String = Default::default();
            let mut last_node: MatrixDataNodeRef;
            let mut last_record_data_category: String = Default::default();

            for result in iter_records {
                let record: DataTypeDefinitionRecord = match result {
                    Ok(ret) => ret,
                    Err(err) => {
                        debug!("parse record error:{:?}", err);
                        panic!()
                    }
                };

                // 跳过空行
                if record.parameter_data_type_name.is_none() {
                    debug!("parameter_data_type_name is empty, perhaps empty row, skip.");
                    continue;
                }

                let record_parameter_data_type_name =
                    record.parameter_data_type_name.clone().unwrap();

                debug!(
                    "name:{:?}, category:{:?}",
                    record.parameter_data_type_name, record.data_category
                );

                // 当前行与之前行内容不相等
                // if record_parameter_data_type_name != last_key {
                let record_data_type_description = record
                    .data_type_description
                    .clone()
                    .unwrap_or_else(|| "".to_string())
                    .to_lowercase();

                if record.data_category.is_none() {
                    debug!(
                        "data_category is empty, sth error. parameter_data_type_name:{:?}",
                        record_parameter_data_type_name
                    );
                    panic!()
                }
                last_record_data_category = record.data_category.clone().unwrap().to_lowercase();

                last_node = data_type_definitions
                    .entry(record_parameter_data_type_name.clone())
                    .or_insert(Rc::new(RefCell::new(MatrixDataNode {
                        name: record_parameter_data_type_name.clone(),
                        description: record_data_type_description.clone(),
                        data_type: Default::default(),
                    })))
                    .clone();
                // }

                let record_data_type = record
                    .data_type
                    .clone()
                    .unwrap_or_else(|| "".to_string())
                    .to_lowercase();

                // 一些便于解析的小函数
                let parse_string_array_length =
                    |record: &DataTypeDefinitionRecord| -> StringArrayLength {
                        match record.string_array_length_type.clone().unwrap().as_str() {
                            "Fixed" => StringArrayLength::FIXED(0),
                            "Dynamic" => StringArrayLength::DYNAMIC(
                                record
                                    .string_array_length_min
                                    .clone()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(),
                                record
                                    .string_array_length_max
                                    .clone()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(),
                            ),
                            _ => {
                                error!(
                                    "parse length type error:{}",
                                    record.string_array_length_type.clone().unwrap()
                                );
                                panic!();
                            }
                        }
                    };

                let parse_number_data_type = |record_data_type: &String| -> Option<MatrixType> {
                    Some(MatrixType::Number {
                        size: match record_data_type.as_str() {
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
                            _ => return None,
                        },
                    })
                };

                // let mut last_node_mut = last_node.borrow_mut();
                if let Some(mut last_node_mut) = last_node.try_borrow_mut().ok() {
                    match last_record_data_category.as_str() {
                        "struct" => {
                            // 首次确定类型需初始化
                            if let MatrixType::Unimplemented {} = last_node_mut.data_type {
                                last_node_mut.data_type = MatrixType::Struct {
                                    children: Default::default(),
                                    children_refs: Default::default(),
                                };
                            }
                            if let MatrixType::Struct {
                                ref mut children,
                                ref mut children_refs,
                            } = last_node_mut.data_type
                            {
                                let record_member_name = match &record.member_name {
                                    Some(s) => s,
                                    None => {
                                        error!(
                                            "parse record member name error. {:?}",
                                            record.parameter_data_type_name
                                        );
                                        panic!()
                                    }
                                };

                                let record_member_description = record
                                    .member_description
                                    .clone()
                                    .unwrap_or_else(|| "".to_string());

                                children.push(record_member_name.clone());

                                match record_data_type.as_str() {
                                    "struct" | "array" | "/" | "" | "union" | "string"
                                    | "utf-8" => {
                                        // 先按顺序猜测信息
                                        let record_member_data_type_reference = &record
                                            .member_data_type_reference
                                            .clone()
                                            .unwrap_or_default();

                                        // Member Datatype Reference 优先级高于 Member Name
                                        let struct_array_union_in_struct_key_name =
                                            if record_member_data_type_reference.is_empty()
                                                || record_member_data_type_reference
                                                    .starts_with("/")
                                            {
                                                record_member_name
                                            } else {
                                                record_member_data_type_reference
                                            };
                                        let new_node = Rc::new(RefCell::new(MatrixDataNode {
                                            name: record_parameter_data_type_name,
                                            description: record_member_description,
                                            data_type: Default::default(),
                                        }));
                                        // 遇到不认识的节点，先在主树中创建，占位置，类型暂时不处理，后续读取到的时候会修改其类型的
                                        data_type_definitions.insert(
                                            struct_array_union_in_struct_key_name.clone(),
                                            Rc::clone(&new_node),
                                        );
                                        children_refs.push(Rc::downgrade(&new_node));
                                    }
                                    _ => {
                                        children_refs.push(Rc::downgrade(&Rc::new(RefCell::new(
                                            MatrixDataNode {
                                                name: record_member_name.clone(),
                                                description: record_member_description,
                                                data_type: match parse_number_data_type(
                                                    &record_data_type,
                                                ) {
                                                    Some(s) => s,
                                                    None => {
                                                        error!(
                                                            "parse record_data_type error.{}",
                                                            record_data_type
                                                        );
                                                        panic!();
                                                    }
                                                },
                                            },
                                        ))));
                                    }
                                }
                            }
                        }
                        "array" => {
                            // 首次确定类型需初始化
                            if let MatrixType::Unimplemented {} = last_node_mut.data_type {
                                last_node_mut.data_type = MatrixType::Array {
                                    length: Default::default(),
                                    children: Default::default(),
                                    children_ref: Default::default(),
                                };
                            }
                            if let MatrixType::Array {
                                ref mut length,
                                ref mut children,
                                ref mut children_ref,
                            } = last_node_mut.data_type
                            {
                                *length = parse_string_array_length(&record);

                                match record_data_type.as_str() {
                                    "struct" | "array" | "/" | "" | "union" | "string"
                                    | "utf-8" => {
                                        // 先按顺序猜测信息
                                        let record_member_name = match &record.member_name {
                                            Some(s) => s,
                                            None => {
                                                error!(
                                                    "parse record member name error. {:?}",
                                                    record.parameter_data_type_name
                                                );
                                                panic!()
                                            }
                                        };

                                        let record_member_description = record
                                            .member_description
                                            .clone()
                                            .unwrap_or_else(|| "".to_string());

                                        let record_member_data_type_reference = &record
                                            .member_data_type_reference
                                            .clone()
                                            .unwrap_or_default();

                                        // Member Datatype Reference 优先级高于 Member Name
                                        let struct_array_union_in_struct_key_name =
                                            if record_member_data_type_reference.is_empty()
                                                || record_member_data_type_reference
                                                    .starts_with("/")
                                            {
                                                record_member_name
                                            } else {
                                                record_member_data_type_reference
                                            };
                                        let new_node = Rc::new(RefCell::new(MatrixDataNode {
                                            name: record_parameter_data_type_name.clone(),
                                            description: record_member_description.clone(),
                                            data_type: Default::default(),
                                        }));
                                        // 遇到不认识的节点，先在主树中创建，占位置，类型暂时不处理，后续读取到的时候会修改其类型的
                                        data_type_definitions.insert(
                                            struct_array_union_in_struct_key_name.clone(),
                                            new_node.clone(),
                                        );
                                        *children = record_member_name.clone();

                                        *children_ref = Rc::downgrade(&new_node.clone());
                                    }
                                    _ => {
                                        // 对于数组中的数值类型，无名、无描述、仅有数据类型
                                        *children_ref =
                                            Rc::downgrade(&Rc::new(RefCell::new(MatrixDataNode {
                                                name: Default::default(),
                                                description: Default::default(),
                                                data_type: match parse_number_data_type(
                                                    &record_data_type,
                                                ) {
                                                    Some(s) => s,
                                                    None => {
                                                        error!("parse record_data_type error.");
                                                        panic!();
                                                    }
                                                },
                                            })));
                                    }
                                }
                            }
                        }
                        "string" => {
                            // 首次确定类型需初始化
                            last_node_mut.data_type = MatrixType::String {
                                length: parse_string_array_length(&record),
                                encoding: match record_data_type.as_str() {
                                    "utf-8" => StringEncoding::UTF8,
                                    "utf-16" => StringEncoding::UTF16LE,
                                    _ => {
                                        error!("parse encoding error:{}", record_data_type);
                                        panic!()
                                    }
                                },
                            };
                        }
                        "integer" | "enumeration" | "float" | "double" => {
                            // TODO: enumeration offset min max ...
                            last_node_mut.data_type =
                                match parse_number_data_type(&record_data_type) {
                                    Some(s) => s,
                                    _ => {
                                        error!("parse data type error: {}", record_data_type);
                                        panic!();
                                    }
                                }
                        }
                        _ => {
                            error!("parse data category error:{}", last_record_data_category);
                            panic!();
                        }
                    };
                }
            }

            // Fill Methods
            let range = wb.worksheet_range("ServiceInterfaces").unwrap();
            let iter_records =
                RangeDeserializerBuilder::with_deserialize_headers::<ServiceInterfacesRecord>()
                    .has_headers(false)
                    .from_range(&range)?
                    .skip(2);

            // 同一个服务的方法必然连续
            let mut last_service: &mut MatrixService;
            let mut last_service_id = 0;

            for result in iter_records {
                let record: ServiceInterfacesRecord = result?;
                // 遇到空行跳过当前行
                if record.service_id.is_none() {
                    continue;
                }
                trace!("{:?}", record.service_interface_name);

                if record.service_id.unwrap() != last_service_id {
                    last_service_id = record.service_id.unwrap();
                    if let Some(service) = services.get_mut(&last_service_id) {
                        last_service = service;
                        last_service.service_description = record.service_description;
                    } else {
                        // 读取错误也跳过当前行
                        error!("Invalid Service ID");
                        continue;
                    }
                }
            }

            // TODO: Read From File
            let matrix_serialazion_parameter = MatrixSerializationParameter {
                alignment: MatrixSerializationParameterSize::B8,
                padding_for_fix_length: false,
                length_field_for_struct: true,
                tag_for_serialization: false,
                string_encoding: StringEncoding::UTF8,
                struct_length_field_size: MatrixSerializationParameterSize::B32,
                string_length_field_size: MatrixSerializationParameterSize::B32,
                array_length_field_size: MatrixSerializationParameterSize::B32,
                union_length_field_size: MatrixSerializationParameterSize::B32,
                union_type_selector_field_size: MatrixSerializationParameterSize::B32,
                union_null: false,
            };

            let matrix = Matrix {
                version,
                service_interfaces: services,
                data_type_definition: data_type_definitions,
                serialization_parameter: matrix_serialazion_parameter,
                matrix_role: roles,
            };

            Ok(matrix)
        }
    }

    #[test]
    fn test() -> Result<(), Box<dyn std::error::Error>> {
        use std::env::set_var;

        set_var("RUST_LOG", "debug");
        env_logger::init();

        let matrix = Matrix::from_excel_file("./matrix.xlsx").expect("error file");

        info!("{:?}", &matrix.service_interfaces.get(&0x5025).unwrap());
        info!("{:?}", &matrix.service_interfaces.keys());
        info!("{:?}", &matrix.service_interfaces.len());

        Ok(())
    }
}
