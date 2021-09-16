use ark_std::{add_to_trace, end_timer, start_timer};
use log::{debug, error, info};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

/// log utils
pub struct LogUtils {}

impl LogUtils {
    pub fn init(fpath: &str) {
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Warn,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                File::create(fpath).unwrap(),
            ),
        ])
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // config.ini content
    // debug = false
    // port = 3223
    // host = "0.0.0.0"
    #[test]
    fn test_init() {
        let gc = start_timer!(|| "test start_timer");
        add_to_trace!(|| "title", || "interesting");
        end_timer!(gc);
        LogUtils::init("test.log");
        error!("Bright red error");
        info!("This only appears in the log file");
        debug!("This level is currently not enabled for any logger");
    }
}
