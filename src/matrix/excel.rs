use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};
use log::{debug, error, info};
use serde::{de, Deserialize, Deserializer};

use crate::errors::MyError;
use crate::types::SomeipServiceId;

use super::types::*;

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

#[allow(dead_code)]
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
    #[serde(rename = "Server MAC")]
    server_mac: String,
    #[serde(rename = "Server IP")]
    server_ip: String,
    #[serde(rename = "Server Port")]
    server_port: u16,
    #[serde(rename = "Client")]
    client: String,
    #[serde(rename = "Client MAC")]
    client_mac: String,
    #[serde(rename = "Client IP")]
    client_ip: String,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
    pub fn from_excel_file<P>(path: P) -> Result<Matrix, MyError>
    where
        P: AsRef<Path>,
    {
        let mut wb: Xlsx<_> = open_workbook(path)?;
        let version = wb
            .worksheet_range("Cover")?
            .get_value((6, 0))
            .unwrap()
            .to_string()
            .strip_prefix("Version:")
            .unwrap()
            .to_string();
        debug!("{:?}", version);

        // Fill Services
        let range = wb.worksheet_range("Deployment").unwrap();
        let iter_records = RangeDeserializerBuilder::with_deserialize_headers::<DeploymentRecord>()
            .from_range(&range)?;

        let mut services: HashMap<SomeipServiceId, MatrixService> = HashMap::new();
        let mut roles: HashMap<String, MatrixRole> = HashMap::new();

        fn get_or_insert_role_for_roles(
            roles: &mut HashMap<String, MatrixRole>,
            role_name: &String,
            role_ip: &String,
            role_mac: &String,
        ) -> MatrixRoleRef {
            roles.entry(role_name.clone()).or_insert(MatrixRole {
                name: role_name.clone(),
                ip_addr: role_ip
                    .parse()
                    .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                mac_addr: role_mac.as_bytes().try_into().unwrap_or([0, 0, 0, 0, 0, 0]),
            })
        }

        for result in iter_records {
            let record: DeploymentRecord = result?;

            let server_role: *const MatrixRole = get_or_insert_role_for_roles(
                &mut roles,
                &record.server,
                &record.server_ip,
                &record.server_mac,
            );

            let client_role: *const MatrixRole = get_or_insert_role_for_roles(
                &mut roles,
                &record.client,
                &record.client_ip,
                &record.client_mac,
            );

            let server_client_pair = MatrixRoleServerClientPair {
                server: record.server.clone(),
                server_ref: Some(server_role),
                server_port: record.server_port,
                client: record.client.clone(),
                client_ref: Some(client_role),
            };

            let service = services.entry(record.service_id).or_insert(MatrixService {
                service_id: record.service_id,
                service_name: record.service_interface_name.clone(),
                service_description: "".to_string(),
                instance_id: record.instance_id,
                major_verison: record.major_version,
                minor_version: record.minor_version,
                methods: HashMap::new(),
                server_client: vec![].into(),
            });

            if !service
                .server_client
                .borrow()
                .iter()
                .any(|s| s == &server_client_pair)
            {
                service.server_client.borrow_mut().push(server_client_pair);
            }
        }

        info!("Fill Services Completed.");

        // Fill Data Type

        // 一些便于解析的小函数
        fn parse_string_array_length(
            record: &DataTypeDefinitionRecord,
        ) -> Result<StringArrayLength, MyError> {
            Ok(
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
                        return Err(MyError::Custom(format!(
                            "parse length type error:{}",
                            record.string_array_length_type.clone().unwrap()
                        )));
                    }
                },
            )
        }

        fn parse_number_data_type(record_data_type: &String) -> Result<MatrixType, MyError> {
            Ok(MatrixType::Number {
                size: NumberType::try_from(record_data_type.clone())?,
            })
        }

        fn parse_string_encoding_data_type(
            record_data_type: &String,
        ) -> Result<StringEncoding, MyError> {
            Ok(match record_data_type.as_str() {
                "utf-8" => StringEncoding::UTF8,
                "utf-16" => StringEncoding::UTF16LE,
                _ => {
                    return Err(MyError::ParseExcelMatrixFileError(format!(
                        "parse encoding error:{}",
                        record_data_type
                    )));
                }
            })
        }

        let range = wb.worksheet_range("DataTypeDefinition").unwrap();
        let iter_records =
            RangeDeserializerBuilder::with_deserialize_headers::<DataTypeDefinitionRecord>()
                .from_range(&range)?;

        let mut data_types: HashMap<String, MatrixDataNode> = HashMap::new();
        // let mut last_key: String = Default::default();
        // let mut last_node: MatrixDataNodeRef = Default::default();
        // let mut last_record_data_category: String = Default::default();

        for result in iter_records {
            let record: DataTypeDefinitionRecord = result?;

            // 跳过空行
            if record.parameter_data_type_name.is_none() {
                debug!("parameter_data_type_name is empty, perhaps empty row, skip.");
                continue;
            }

            let record_parameter_data_type_name = record.parameter_data_type_name.clone().unwrap();

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
                return Err(MyError::ParseExcelMatrixFileError(format!(
                    "data_category is empty, sth error. parameter_data_type_name:{:?}",
                    record_parameter_data_type_name
                )));
            }
            let last_record_data_category = record.data_category.clone().unwrap().to_lowercase();

            let last_key = record_parameter_data_type_name;

            let last_node: &mut MatrixDataNode =
                data_types
                    .entry(last_key.clone())
                    .or_insert(MatrixDataNode {
                        name: last_key.clone(),
                        description: record_data_type_description.clone(),
                        data_type: Default::default(),
                    });

            last_node.description = record.data_type_description.clone().unwrap_or_default();

            let record_data_type = record
                .data_type
                .clone()
                .unwrap_or_else(|| "".to_string())
                .to_lowercase();

            // if let Some(ref mut last_node_mut) = last_node {
            // if let last_node_mut = last_node {
            match last_record_data_category.as_str() {
                "struct" => {
                    // 首次确定类型需初始化
                    if let MatrixType::Unimplemented {} = last_node.data_type {
                        last_node.borrow_mut().data_type = MatrixType::Struct {
                            members: Default::default(),
                        };
                    }

                    let record_member_name = match &record.member_name {
                        Some(s) => s,
                        None => {
                            return Err(MyError::ParseExcelMatrixFileError(format!(
                                "parse record member name error. {:?}",
                                record.parameter_data_type_name
                            )))
                        }
                    };

                    let record_member_description = record
                        .member_description
                        .clone()
                        .unwrap_or_else(|| "".to_string());

                    let ptr: *const MatrixDataNode = match record_data_type.as_str() {
                        "struct" | "array" | "/" | "" | "union" | "string" | "utf-8" => {
                            // 先按顺序猜测信息
                            let record_member_data_type_reference = &record
                                .member_data_type_reference
                                .clone()
                                .unwrap_or_default();

                            // Member Datatype Reference 优先级高于 Member Name
                            let struct_array_union_in_struct_key_name =
                                if record_member_data_type_reference.is_empty()
                                    || record_member_data_type_reference.starts_with("/")
                                {
                                    record_member_name
                                } else {
                                    record_member_data_type_reference
                                };

                            data_types
                                .entry(struct_array_union_in_struct_key_name.clone())
                                .or_insert(MatrixDataNode {
                                    name: struct_array_union_in_struct_key_name.clone(),
                                    description: record_member_description.clone(),
                                    data_type: Default::default(),
                                })
                        }
                        _ => &MatrixDataNode {
                            name: record_member_name.clone(),
                            description: record_member_description.clone(),
                            data_type: parse_number_data_type(&record_data_type)?,
                        },
                    };

                    let last_node_mut = data_types.get_mut(&last_key.clone()).unwrap();
                    if let MatrixType::Struct { ref mut members } = last_node_mut.data_type {
                        // (*children).push(record_member_name.clone());
                        // (*children_refs).push(ptr);
                        (*members).push(MatrixMember {
                            member_name: record_member_name.clone(),
                            member_description: record_member_description.clone(),
                            member_ref: Some(ptr),
                        })
                    }
                }
                "array" => {
                    // 首次确定类型需初始化
                    if let MatrixType::Unimplemented {} = last_node.data_type {
                        last_node.borrow_mut().data_type = MatrixType::Array {
                            length: Default::default(),
                            member: Default::default(),
                        };
                    }

                    let record_member_name = &record.member_name.clone().unwrap_or_default();
                    let record_member_description =
                        record.member_description.clone().unwrap_or_default();
                    let record_member_data_type_reference = &record
                        .member_data_type_reference
                        .clone()
                        .unwrap_or_default();

                    let ptr: *const MatrixDataNode = match record_data_type.as_str() {
                        "struct" | "array" | "/" | "" | "union" | "string" | "utf-8" => {
                            // Member Datatype Reference 优先级高于 Member Name
                            // 且member_name一定不为空
                            if record_member_name.is_empty() {
                                return Err(MyError::ParseExcelMatrixFileError(format!(
                                    "parse array error. {}",
                                    last_key
                                )));
                            }
                            let struct_array_union_in_struct_key_name =
                                if record_member_data_type_reference.is_empty()
                                    || record_member_data_type_reference.starts_with("/")
                                {
                                    record_member_name
                                } else {
                                    record_member_data_type_reference
                                };

                            data_types
                                .entry(struct_array_union_in_struct_key_name.clone())
                                .or_insert(MatrixDataNode {
                                    name: struct_array_union_in_struct_key_name.clone(),
                                    description: record_member_description.clone(),
                                    data_type: Default::default(),
                                })
                        }
                        _ => {
                            // 对于数组中的数值类型，无名、无描述、仅有数据类型
                            &MatrixDataNode {
                                name: Default::default(),
                                description: Default::default(),
                                data_type: parse_number_data_type(&record_data_type)?,
                            }
                        }
                    };

                    let last_node_mut = data_types.get_mut(&last_key.clone()).unwrap();
                    if let MatrixType::Array {
                        ref mut length,
                        ref mut member,
                    } = last_node_mut.data_type
                    {
                        (*length) = parse_string_array_length(&record)?;
                        (*member) = MatrixMember {
                            member_name: record_member_name.clone(),
                            member_description: record_member_description.clone(),
                            member_ref: Some(ptr),
                        };
                    }
                }
                "string" => {
                    // 首次确定类型需初始化
                    last_node.data_type = MatrixType::String {
                        length: parse_string_array_length(&record)?,
                        encoding: parse_string_encoding_data_type(&record_data_type)?,
                    };
                }
                "integer" | "enumeration" | "float" | "double" => {
                    // TODO: enumeration offset min max ...
                    last_node.data_type = parse_number_data_type(&record_data_type)?;
                }
                _ => {
                    return Err(MyError::ParseExcelMatrixFileError(format!(
                        "parse data category error:{}",
                        last_record_data_category
                    )));
                }
            };
            // }
        }

        // Fill Methods
        let range = wb.worksheet_range("ServiceInterfaces")?;
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
            debug!("{:?}", record.service_interface_name);

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
        let serialization_parameter = MatrixSerializationParameter {
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

        Ok(Matrix {
            version,
            serialization_parameter,
            roles,
            services,
            services_map_by_name: HashMap::new(),
            data_types,
        })
    }
}

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::env::set_var;
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::path::Path;

    set_var("RUST_LOG", "debug");
    env_logger::init();

    let matrix = Matrix::from_excel_file("./matrix.xlsx").expect("error file");

    info!("{:?}", &matrix.services.keys());
    info!("{:?}", &matrix.services.len());
    // info!("{:?}", &matrix.data_types.keys());
    info!("{:?}", &matrix.data_types.len());
    info!(
        "{:?}",
        &matrix
            .data_types
            .get(&"Struct_PickUpPointDetailInfo".to_string())
            .unwrap()
    );
    info!(
        "{:?}",
        &matrix
            .data_types
            .get(&"Struct_GPSPoint".to_string())
            .unwrap()
    );
    info!(
        "{:?}",
        &matrix
            .data_types
            .get(&"String_DynamicStringData200".to_string())
            .unwrap()
    );

    let tmp_dir = env::temp_dir();
    let tmp_file_path = tmp_dir.join("temp_data.txt");
    let mut file = File::create(&tmp_file_path)?;

    let data = serde_json::to_string(&matrix).unwrap();

    file.write_all(data.as_bytes())?;

    file.sync_all()?;

    Ok(())
}
