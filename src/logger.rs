use chrono::Utc;
use chrono_tz::Asia::Tokyo;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::{EnvFilter, fmt};

struct JstTime;

impl FormatTime for JstTime {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = Utc::now().with_timezone(&Tokyo);
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

pub fn init() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(true)
        .with_file(true)
        .with_timer(JstTime)
        .init();
}
