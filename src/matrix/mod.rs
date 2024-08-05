/// 从Excel或者json文件中加载一个矩阵
/// TODO: load/save json
pub mod matrix_loader {

    use std::borrow::BorrowMut;
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::Path;
    use std::rc::{Rc, Weak};

    use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
    use log::{debug, error, info, log_enabled, trace, Level};
    use serde::{de, Deserialize, Deserializer};

    use crate::types::{
        ClientMatrixRole, Matrix, MatrixDataNode, MatrixDataNodeRef, MatrixRole, MatrixService,
        MatrixType, NumberType, ServerMatrixRole, SomeipServiceId, StringArrayLength,
    };
    use crate::types::{
        MatrixSerializationParameter, MatrixSerializationParameterSize, StringEncoding,
    };

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
