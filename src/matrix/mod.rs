mod excel;
mod types;

/// TODO: load/save json


use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};
use log::{debug, error, info, trace};
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::types::{
    ServerPort, SomeipInstanceId, SomeipMajorVersion, SomeipMethodId, SomeipMinorVersion,
    SomeipServiceId, SomeipTransportPortocol,
};
