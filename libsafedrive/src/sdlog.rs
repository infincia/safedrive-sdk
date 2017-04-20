use log::{LogLevel, LogLevelFilter, LogMetadata, LogRecord, Log};
use simplelog::{Config, SharedLogger};

use LOG;

pub struct SDLogger {
    level: LogLevelFilter,
    config: Config,
}

impl SDLogger {
    pub fn new(log_level: LogLevelFilter, config: Config) -> Box<SDLogger> {
        Box::new(SDLogger { level: log_level, config: config })
    }
}

impl Log for SDLogger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            match record.level() {
                LogLevel::Debug | LogLevel::Trace => {},
                _ => {
                    let mut log = LOG.write();
                    let line = format!("{}", record.args());

                    (*log).push(line);
                },
            }
        }
    }
}

impl SharedLogger for SDLogger {

    fn level(&self) -> LogLevelFilter {
        self.level
    }

    fn config(&self) -> Option<&Config>
    {
        Some(&self.config)
    }

    fn as_log(self: Box<Self>) -> Box<Log> {
        Box::new(*self)
    }

}
