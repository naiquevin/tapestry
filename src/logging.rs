use env_logger::WriteStyle;
use log::LevelFilter;

pub fn init(verbosity: u8) {
    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter(None, level)
        .write_style(WriteStyle::Always)
        .init();
}
