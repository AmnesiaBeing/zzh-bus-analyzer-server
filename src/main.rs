mod someippacketparser;
mod someiptypes;

use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
use std::path::PathBuf;

extern crate clap;
use clap::Parser;
use pnet::datalink::Channel::Ethernet;

use log::{debug, error, info, log_enabled, Level};

use crate::someippacketparser::someip_packet_parser::{handle_packet_loop, init_from_path};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 后台运行，持续读取指定网卡报文
    #[arg(short, long)]
    daemon: bool,

    /// 矩阵文件
    #[arg(short, long)]
    matrix: Option<PathBuf>,

    /// 需要筛选的serivceid和methodid，用:间隔，支持十六进制和十进制，如果加载了矩阵文件，可以支持信号名
    /// methodid支持不使用:间隔，表示筛选所有信号
    #[arg(short, long)]
    signals: Option<String>,

    /// 回放报文文件
    #[arg(short, long)]
    file: Option<PathBuf>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    debug!("{:?}", args);

    let mut wb: Xlsx<_> = open_workbook(args.matrix.unwrap()).expect("Cannot open file");

    let range = wb.worksheet_range("ServiceInterfaces").unwrap();
    let mut iter =
        RangeDeserializerBuilder::with_headers(&["Service InterFace Name", "Service ID"])
            .from_range(&range)
            .unwrap();
    let result = iter.next();
    let (service_name, service_id): (String, String) = result.unwrap().expect("");
    debug!("{},{}", service_id, service_name);
    let result = iter.next();
    let (service_name, service_id): (String, String) = result.unwrap().expect("");

    debug!("{},{}", service_id, service_name);

    // TODO: 针对signals进行处理，筛选出要匹配的服务

    // let mut ret: Vec<SomeipTransportMessage> = vec![];

    // 最大是0xFFFF，用u16即可
    // let filter_sid = u16::from_str_radix(&args.signals[2..], 16).unwrap();

    let pp = init_from_path(args.file.unwrap());
    handle_packet_loop(&pp);
}
