use slog::Drain;
pub use slog::{debug, error, info, o, trace, warn};
pub use slog::{FnValue, Logger};
use slog_json::Json;
use std::sync::Mutex;

pub fn get_root_logger() -> Logger {
    Logger::root(
        Mutex::new(slog_envlogger::new(Json::default(std::io::stdout()))).map(slog::Fuse),
        o!(
            "file" => FnValue(move |info| info.file()),
            "module" => FnValue(move |info| info.module()),
            "function" => FnValue(move |info| info.function()),
            "line" => FnValue(move |info| format!("{}", info.line())),
        ),
    )
}
