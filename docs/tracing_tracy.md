# Tracing/Tracy Integration (2025-12-31T22:17:59.005Z)

## Dependencies
- tracing, tracing-subscriber, tracing-tracy, tracing-android (Android), tracing-oslog (Apple).

## Init (cross-platform)
```rust
use tracing::{info, span, Level};
#[cfg(target_os = "android")] use tracing_android::AndroidLayer;
#[cfg(any(target_os = "macos", target_os = "ios", target_os = "tvos"))] use tracing_oslog::OsLogger;
use tracing_subscriber::{registry, prelude::*, EnvFilter};
#[cfg(feature="tracy")] use tracing_tracy::TracyLayer;

pub fn init_tracing() {
  let filter = EnvFilter::from_default_env()
    .add_directive("kenken=info".parse().unwrap())
    .add_directive("kenken_solver=warn".parse().unwrap())
    .add_directive("kenken_gen=debug".parse().unwrap());
  let reg = registry().with(filter);
  #[cfg(feature="tracy")] let reg = reg.with(TracyLayer::new());
  #[cfg(target_os = "android")] let reg = reg.with(AndroidLayer::new("KenKen"));
  #[cfg(any(target_os = "macos", target_os = "ios", target_os = "tvos"))] let reg = reg.with(OsLogger::new("KenKen"));
  let reg = reg.with(tracing_subscriber::fmt::layer().with_target(false).with_ansi(false));
  reg.init();
  info!("tracing initialized");
}
```

## Usage
```rust
let span = span!(Level::INFO, "generator", size = n);
let _enter = span.enter();
info!(puzzles=?count, "batch done");
```

## Perfetto bridge (Android)
- Use tracing spans + ATrace via ndk-glue if needed; or rely on tracing-android routing to Logcat and Perfetto.
