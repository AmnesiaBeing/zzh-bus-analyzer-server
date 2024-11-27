use std::{cell::RefCell, path::PathBuf};

use pnet::{datalink, transport::Config};

pub mod pcap_source;

mod pnet_packet_someip;

pub struct Source;
