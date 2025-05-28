use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::prelude::*;

/// Custom logger initialization to exclude timestamps but keep colors.
/// This provides cleaner logs by removing the timestamp prefix.
pub fn init_custom_logger() {
    // Empty time formatter that doesn't print anything
    struct EmptyTime;
    impl FormatTime for EmptyTime {
        fn format_time(
            &self,
            _: &mut tracing_subscriber::fmt::format::Writer<'_>,
        ) -> std::fmt::Result {
            // Do nothing, effectively removing timestamps
            Ok(())
        }
    }

    // Set up a custom tracing subscriber with our configuration
    let format = format()
        .with_timer(EmptyTime)
        .with_level(true)
        .with_target(true)
        .with_ansi(true); // Keep colors

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(format)
                .with_filter(
                    tracing_subscriber::filter::EnvFilter::from_default_env()
                        .add_directive("info".parse().unwrap())
                        .add_directive("wgpu_core=warn".parse().unwrap())
                        .add_directive("wgpu_hal=warn".parse().unwrap()),
                ),
        )
        .init();
}
