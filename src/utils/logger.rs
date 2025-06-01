use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::prelude::*;

/// Custom logger initialization to exclude timestamps but keep colors.
/// This provides cleaner logs by removing the timestamp prefix.
///
/// Use BEZY_LOG=info or BEZY_LOG=debug environment variable to increase verbosity.
/// Example: BEZY_LOG=info cargo run
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

    // Check if user wants to override log level (default to warn for minimal noise)
    let default_level =
        std::env::var("BEZY_LOG").unwrap_or_else(|_| "warn".to_string());

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
                        // Set default to warn level for minimal noise
                        .add_directive(default_level.parse().unwrap())
                        // Keep only critical startup messages at info level
                        .add_directive("bezy::io::ufo=info".parse().unwrap())
                        .add_directive(
                            "bevy_winit::system=info".parse().unwrap(),
                        )
                        // Suppress very noisy render layer messages completely
                        .add_directive("wgpu_core=error".parse().unwrap())
                        .add_directive("wgpu_hal=error".parse().unwrap())
                        .add_directive("bevy_render=error".parse().unwrap()),
                ),
        )
        .init();
}
