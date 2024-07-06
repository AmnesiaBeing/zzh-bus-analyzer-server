/// 从Excel或者json文件中加载一个矩阵
/// TODO: load/save json
pub mod matrix_loader {

    use std::collections::HashMap;

    use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
    use log::{debug, error, info, log_enabled, Level};
    use serde::Deserialize;

    use crate::types::{
        Matrix, MatrixService, SomeipInstantId, SomeipMajorVersion, SomeipMinorVersion,
        SomeipServiceId,
    };
    use crate::types::{
        MatrixSerializationParameter, MatrixSerializationParameterSize, StringEncoding,
    };

    #[derive(Deserialize)]
    struct Record {
        service_interface_name: String,
        service_id: Option<SomeipServiceId>,
        instance_id: Option<SomeipInstantId>,
        major_version: Option<SomeipMajorVersion>,
        minor_version: Option<SomeipMinorVersion>,
        server: String,
        server_ip: String,
        server_ip_subnetmask: String,
        server_port: String,
        client: String,
        client_ip: String,
        client_ip_subnetmask: String,
        client_port: String,
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
            "Server IP SubnetMask",
        ])
        .from_range(&range)?;

        let mut services: HashMap<SomeipServiceId, MatrixService> = HashMap::new();

        for result in iter_records.skip(1) {
            let record: Record = result?;
            if let Some(key) = record.service_id {
                if !services.contains_key(&key) {
                    services.insert(
                        key,
                        MatrixService {
                            service_id: key,
                            service_name: record.service_interface_name,
                            service_description: "".to_string(),
                            instance_id: todo!(),
                            major_verison: todo!(),
                            minor_version: todo!(),
                            methods: todo!(),
                            server: todo!(),
                            client: todo!(),
                        },
                    );
                }
            }
        }

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
