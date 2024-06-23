pub mod someip_matrix_loader {

    enum SOAMatrixServiceMethodType {
        UNDEFINED,
        RRMethod(SOARRMethod),
        FFMethod(SOAFFMethod),
        EVENT(SOAEventMethod),
        FIELD,
    }

    struct SOAMatrixDataTypeDefinition {}

    struct SOARRMethod {
        data_in: Vec<String>,
        data_out: String,
    }

    struct SOAFFMethod {
        data_in: Vec<String>,
    }

    struct SOAEventMethod {}

    enum SOAMatrixServiceMethodTransportPortocol {
        TCP,
        UDP,
    }

    struct SOAMatrixServiceMethod {
        transport_protocol: SOAMatrixServiceMethodTransportPortocol,
        method_type: SOAMatrixServiceMethodType,
        method_name: String,
        method_id: u16,
        data: Box<dyn Any>,
    }

    struct SOAMatrixService {
        service_id: u16,
        service_name: String,
        service_description: String,
        methods: Vec<SOAMatrixServiceMethod>,
    }

    enum SOASerializationParameterSize {
        BIT8,
        BIT16,
        BIT32,
        BIT64,
    }

    /*
    Following requirements are common for both fixed length and dynamic length strings.
    [PRS_SOMEIP_00372] Different Unicode encoding shall be supported including
    UTF-8, UTF-16BE and UTF-16LE.c(RS_SOMEIP_00038)
    [PRS_SOMEIP_00948] UTF-8 strings shall be zero terminated with a "\0" character.
    This means they shall end with a 0x00 Byte.c(RS_SOMEIP_00038)
    [PRS_SOMEIP_00084] UTF-16LE and UTF-16BE strings shall be zero terminated
    with a "\0" character. This means they shall end with (at least) two 0x00 Bytes.c(RS_-
    SOMEIP_00038)
    [PRS_SOMEIP_00085] UTF-16LE and UTF-16BE strings shall have an even length.c
    (RS_SOMEIP_00038)
    [PRS_SOMEIP_00086] UTF-16LE and UTF-16BE strings having an odd length the
    last byte shall be ignored.c(RS_SOMEIP_00038)
    [PRS_SOMEIP_00087] All strings shall always start with a Byte Order Mark (BOM)
    in the first three (UTF-8) or two (UTF-16) bytes of the to be serialized array containing
    the string. The BOM shall be included in fixed-length-strings as well as dynamic-length
    strings. BOM allows the possibility to detect the used encoding.c(RS_SOMEIP_00038)
    */

    enum SOASerializationParameterStringEncoding {
        UTF8,
        UTF16LE,
        UTF16BE,
    }

    struct SOASerializationParameter {
        alignment: SOASerializationParameterSize,
        padding_for_fix_length: bool,
        length_field_for_struct: bool,
        tag_for_serialization: bool,
        string_encoding: SOASerializationParameterStringEncoding,
        struct_length_field_size: SOASerializationParameterSize,
        string_length_field_size: SOASerializationParameterSize,
        array_length_field_size: SOASerializationParameterSize,
        union_length_field_size: SOASerializationParameterSize,
        union_type_selector_field_size: SOASerializationParameterSize,
        union_null: bool,
    }

    struct SOAMatrix {
        version: String,
        service_interfaces: Vec<SOAMatrixService>,
        data_type_definition: Vec<SOAMatrixDataTypeDefinition>,
        serialization_parameter: SOASerializationParameter,
    }
}
