# Bezy Custom Logger

## Overview

Bezy uses a custom logging system that provides clean, formatted logs without timestamps but with color-coding for log levels. This document explains what the custom logger is, how it works, and why it was implemented.

## What It Is

The custom logger is a specialized configuration of the `tracing_subscriber` logging framework that:

- Omits timestamps from log messages
- Maintains color coding for different log levels
- Uses hierarchical filtering to control verbosity
- Displays target information to help identify log sources
- Is defined in its own module (`src/logger.rs`)

## How It Works

### Implementation Details

The logger works by creating a custom time formatter that effectively does nothing, which removes timestamps from the log output. Here's how it's implemented:

1. A custom `EmptyTime` struct is defined that implements the `FormatTime` trait
2. The `format_time` method is implemented to return `Ok(())` without writing anything
3. A `tracing_subscriber` registry is configured with this formatter and other options
4. Log filtering is set up to show `info` level logs by default, with stricter filtering for noisy components

```rust
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
```

The logger is initialized once at application startup in `app::create_app()`.

### Log Format

Logs appear in the format:
```
INFO module_name: This is a log message
```

Where:
- `INFO` is the log level (color-coded as blue)
- `module_name` is the Rust module that generated the log
- The rest is the actual log message

## Why It Exists

The custom logger exists for several reasons:

1. **Cleaner Output**: By removing timestamps, logs are more concise and readable, especially during development
2. **Targeted Filtering**: The logger configuration filters out excessive messages from dependencies like `wgpu` while allowing Bezy's own logs through
3. **Maintainability**: Having the logger in its own module makes it easier to modify logging behavior without affecting other parts of the application
4. **Consistency**: Provides a uniform logging appearance across the application

## Using the Logger

The logger is automatically initialized during application startup, so you don't need to do anything special to use it. Simply use the standard Rust logging macros:

```rust
// Different log levels
info!("This is an informational message");
warn!("This is a warning");
error!("This is an error");
debug!("This is a debug message");
trace!("This is a trace message");
```

### Debug Logging

For more detailed logging, you can set the `BEZY_DEBUG` environment variable when running the application:

```sh
BEZY_DEBUG=1 cargo run -- --load-ufo your_font.ufo
```

## Extending the Logger

To extend the logger with new features:

1. Modify the `logger.rs` file to add new configuration options
2. Consider adding support for log output to files
3. You could implement different log formats for different environments (development vs. production)
4. Add support for structured logging if needed for complex debugging

The modular design makes it easy to add these features without affecting the rest of the application. 