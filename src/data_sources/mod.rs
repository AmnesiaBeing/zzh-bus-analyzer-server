use std::{cell::RefCell, path::PathBuf};

use pnet::{datalink, transport::Config};

pub mod pcap_source;

mod pnet_packet_someip;

pub struct DataSource {
    config: DataSourceConfig,
    rx: Box<dyn DataSourceReceiver>,
}

pub enum DataSourceConfig {
    PcapFile { file_path: PathBuf },
    DataLinkInterface {},
    AdbInterface {},
}

pub trait DataSourceReceiver: Send {
    // 尽量从源文件来，不参与拷贝
    fn next(&mut self) -> std::io::Result<&[u8]>;
}
