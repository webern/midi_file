#![allow(dead_code)]

use log::LevelFilter;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Once;

pub const ADESTE_FIDELES: &str = "adeste_fideles.mid";
pub const ALS_DIE_ROEMER: &str = "als_die_roemer.mid";
pub const AVE_MARIS_STELLA: &str = "ave_maris_stella.mid";
pub const BARITONE_SAX: &str = "baritone_saxophone.error.mid";
pub const B_GUAJEO: &str = "b_guajeo.mid";
pub const LATER_FOLIA: &str = "later_folia.mid";
pub const LOGIC_PRO: &str = "logic_pro.mid";
pub const PHOBOS_DORICO: &str = "phobos_dorico.mid";
pub const TOBEFREE: &str = "tobefree.mid";

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
        .filter(None, LevelFilter::Warn)
        .init();
}

pub fn test_file<S: AsRef<str>>(filename: S) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(filename.as_ref());
    p.canonicalize()
        .unwrap_or_else(|_| panic!("bad path '{}'", p.display()))
}
