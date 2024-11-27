pub mod pcap_source;

// pub trait SourceContext {}

// pub trait Source {
//     // 不同的Source有不同的创建参数，但必须至少有个new函数
//     fn new(ctx: Box<dyn SourceContext>) -> Self;
//     // 每个Source都需要有一个执行方式，Source只有一个输出
//     fn start(self, send_data: crossbeam_channel::Sender<Vec<u8>>) -> std::io::Result<std::thread::JoinHandle<()>>;
// }
