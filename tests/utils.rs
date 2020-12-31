use log::LevelFilter;
use std::io::Write;
use std::sync::Once;

static LOGGER: Once = Once::new();

pub fn enable_logging() {
    LOGGER.call_once(logger_init)
}

fn logger_init() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Trace)
        .init();
}
