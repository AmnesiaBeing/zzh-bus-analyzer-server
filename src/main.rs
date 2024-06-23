// mod someippacketparser;
// mod someiptypes;
mod someipmatrixloader;

use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
use std::path::PathBuf;

use std::env::set_var;

extern crate clap;
use clap::Parser;
// use pnet::datalink::Channel::Ethernet;

use log::{debug, error, info, log_enabled, Level};

// use crate::someippacketparser::someip_packet_parser::{handle_packet_loop, init_from_path};

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
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let args = Args::parse();

    debug!("{:?}", args);

    let mut wb: Xlsx<_> = open_workbook(args.matrix.unwrap()).expect("Cannot open file");

    let range = wb.worksheet_range("ServiceInterfaces").unwrap();

    let mut iter = range.rows();
    // 从第一行读取标题
    let title = iter.next().unwrap();
    debug!("title:{:?}", title);
    // 忽略第二个空行
    iter.next();

    //     "Service InterFace Name",
    //     "Service ID",
    //     "Service Description",
    //     "Method/Event/Field",
    //     "Setter/Getter/Notifier",
    //     "Element Name",
    //     "Element Description",
    //     "Method ID/Event ID",
    //     "Eventgroup Name",
    //     "Eventgroup ID",
    //     "Send Strategy",
    //     "Cyclic Time (ms)",
    //     "Parameter Name",
    //     "IN/OUT",
    //     "Parameter Description",
    //     "Parameter Data Type",
    //     "UDP/TCP",
    //     "AutoSAR E2E Protection (Profile 6)",

    // iter.next();
    // let result = iter.next().unwrap();
    // debug!("result:{:?}", result.expect(""));

    // TODO: 针对signals进行处理，筛选出要匹配的服务

    // let mut ret: Vec<SomeipTransportMessage> = vec![];

    // 最大是0xFFFF，用u16即可
    // let filter_sid = u16::from_str_radix(&args.signals[2..], 16).unwrap();

    // let pp = init_from_path(args.file.unwrap());
    // handle_packet_loop(&pp);
}
