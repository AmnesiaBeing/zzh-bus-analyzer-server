/// 从Excel或者json文件中加载一个矩阵
/// TODO: load/save json
pub mod matrix_loader {

    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use std::rc::Rc;

    use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
    use log::{debug, error, info, log_enabled, Level};
    use serde::{de, Deserialize, Deserializer};

    use crate::types::{
        ClientMatrixRole, Matrix, MatrixRole, MatrixService, ServerMatrixRole, SomeipInstantId,
        SomeipMajorVersion, SomeipMinorVersion, SomeipServiceId,
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

    #[derive(Deserialize)]
    struct Record {
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

    #[test]
    fn test() -> Result<(), Box<dyn std::error::Error>> {
        use std::env::set_var;

        set_var("RUST_LOG", "debug");
        env_logger::init();
        let mut wb: Xlsx<_> = open_workbook("./matrix.xlsx").expect("Cannot open file");

        let version = wb
            .worksheet_range("Cover")
            .unwrap()
            .get_value((6, 0))
            .unwrap()
            .to_string()
            .strip_prefix("Version:")
            .unwrap()
            .to_string();

        println!("{:?}", version);

        // let range = wb.worksheet_range("ServiceInterfaces").unwrap();

        // let mut iter = range.rows();
        // // 从第一行读取标题
        // let title = iter.next().unwrap();
        // debug!("title:{:?}", title);
        // // 忽略第二个空行
        // iter.next();

        let range = wb.worksheet_range("Deployment").unwrap();

        let iter_records = RangeDeserializerBuilder::with_headers(&[
            "Service InterFace Name",
            "Service ID",
            "Instance ID",
            "Major Version",
            "Minor Version",
            "Server",
            "Server IP",
            "Server MAC",
            "Server Port",
            "Client",
            "Client IP",
            "Client MAC",
        ])
        .from_range(&range)?;

        let mut services: HashMap<SomeipServiceId, MatrixService> = HashMap::new();
        let mut roles: HashMap<String, Rc<MatrixRole>> = HashMap::new();

        for result in iter_records {
            let record: Record = result?;

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
                    // methods: Vec::new(),
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

        println!("{:?}", &services.get(&0x5005).unwrap());
        println!("{:?}", &services.get(&0x8008).unwrap());
        println!("{:?}", &services.get(&0x8009).unwrap());
        println!("{:?}", &services.get(&0x800A).unwrap());
        println!("{:?}", &services.get(&0x1081).unwrap());
        println!("{:?}", &services.get(&0x2038).unwrap());
        println!("{:?}", &services.get(&0x2039).unwrap());
        println!("{:?}", &services.get(&0x0029).unwrap());
        println!("{:?}", &services.get(&0x106E).unwrap());
        println!("{:?}", &services.get(&0x5025).unwrap());
        println!("{:?}", &services.keys());
        println!("{:?}", &services.len());

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

        // let matrix = Matrix {
        //     version,
        //     service_interfaces: todo!(),
        //     data_type_definition: todo!(),
        //     serialization_parameter: matrix_serialazion_parameter,
        // };

        Ok(())
    }
}
