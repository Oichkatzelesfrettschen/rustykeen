# KenKen Solver - Android Example App

A minimal Android application demonstrating the KenKen solver via UniFFI bindings.

## Architecture

```
├── app/                          # Android app module
│   ├── src/
│   │   └── main/
│   │       ├── kotlin/
│   │       │   └── com/example/kenken/
│   │       │       ├── MainActivity.kt      # Main UI activity
│   │       │       ├── PuzzleViewModel.kt   # ViewModel (state management)
│   │       │       └── KeenApi.kt           # UniFFI wrapper
│   │       └── res/
│   │           ├── layout/
│   │           │   └── activity_main.xml    # UI layout
│   │           └── values/
│   │               └── strings.xml          # String resources
│   ├── build.gradle.kts          # Gradle build configuration
│   └── native-debug.properties    # Rust library configuration
│
└── build.gradle.kts              # Root Gradle configuration
```

## Features

1. **Parse puzzle from sgt-desc format** - Input grid size and description
2. **Solve puzzle** - Select deduction tier (None, Easy, Normal, Hard)
3. **Display solution** - Show grid with solution
4. **Count solutions** - Verify puzzle uniqueness (2 solutions = unique)

## Building

### Prerequisites

- Android SDK (API 21+)
- NDK r25c (specified in build.gradle.kts)
- Rust toolchain with Android targets
- cargo-ndk for cross-compilation

### Build Steps

1. **Set up Rust Android support:**
   ```bash
   rustup target add aarch64-linux-android
   rustup target add armv7-linux-androideabi
   rustup target add x86_64-linux-android
   ```

2. **Install cargo-ndk:**
   ```bash
   cargo install cargo-ndk --version 3.2.0
   ```

3. **Build native libraries:**
   ```bash
   cd ../../  # Go back to workspace root
   cargo ndk -t arm64-v8a -t armeabi-v7a build --release \
     -p kenken-uniffi --all-features
   ```

4. **Build Android app:**
   ```bash
   cd examples/android
   ./gradlew build  # or assembleDebug for APK
   ```

## Running

### On Physical Device

```bash
./gradlew installDebug   # Install to connected device
```

### On Emulator

```bash
./gradlew installDebug   # Install to running emulator
```

Then launch the "KenKen Solver" app from the device home screen.

## Implementation Notes

### UniFFI Integration

The app imports the `libkenken_uniffi.so` library (generated via UniFFI). The Kotlin
interface is auto-generated from `kenken-uniffi/src/keen.udl`.

### State Management

- **ViewModel** pattern for lifecycle-aware state
- **LiveData** for reactive UI updates
- Solver operations run on background thread to avoid ANR

### Error Handling

- Graceful fallback for invalid inputs
- Clear error messages for parsing failures
- Timeout handling for large grids

## Example Usage

1. Enter grid size (2-9)
2. Enter sgt-desc format string (e.g., `_5,a1a2a2a1` for 2x2)
3. Select deduction tier (Easy recommended for initial testing)
4. Tap "Solve" button
5. Solution displays in grid view

## Limitations

- No cage visualization (grid output only)
- No puzzle generation UI (use Rust CLI instead)
- Single-puzzle workflow (no save/load)

## Future Enhancements

1. **UI Polish**
   - Cage boundary visualization
   - Operation display (Add, Mul, Sub, Div)
   - Grid size picker

2. **Performance**
   - Benchmark different deduction tiers
   - Async solver with cancellation

3. **Features**
   - Puzzle generation from app
   - Solution history
   - Leaderboard (local storage)

## Troubleshooting

### "Native library not found"

Ensure `libkenken_uniffi.so` exists in `app/src/main/jniLibs/<abi>/`.

### Build fails with "cargo not found"

Set `CARGO` environment variable to full path of cargo binary.

### App crashes on startup

Check Logcat: `adb logcat | grep kenken`

## Security Considerations

- All computation is local (no network access)
- No data persistence by default
- FFI boundary validates all inputs

## References

- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)
- [Android Native Development Kit](https://developer.android.com/ndk)
- [Kotlin Coroutines](https://kotlinlang.org/docs/coroutines-overview.html)
