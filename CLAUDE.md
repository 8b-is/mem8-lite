# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

MEM8-FS Lite is a lightning-fast wave-based filesystem implementation that achieves 973× performance improvements over traditional storage systems. It's the filesystem-only version of the full MEM8 consciousness system - all the speed without the consciousness framework.

## Build and Test Commands

### Building
```bash
# Standard build
cargo build

# Release build with optimizations
cargo build --release

# Build with FUSE mounting support
cargo build --features fuse-mount

# Build with async support
cargo build --features async

# Build with all features
cargo build --all-features
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test fs::
```

### Running Examples
```bash
# Basic example (simple storage and filesystem demos)
cargo run --example basic

# Filesystem example with FUSE mounting (when implemented)
cargo run --example filesystem --features fuse-mount
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Run clippy with all features
cargo clippy --all-features -- -D warnings
```

## Architecture

### Core Components

1. **Wave Storage Layer** (`WaveStorage`)
   - Converts data to Complex64 wave patterns
   - Uses blake3 for signatures
   - Append-only storage format
   - Memory-mapped I/O for performance

2. **Simple Storage API** (`Mem8Lite` in `lite` module - needs implementation)
   - Key-value interface using wave signatures
   - `store()` → signature, `retrieve()` → data
   - Configurable wave frequencies

3. **Filesystem API** (`Mem8Fs`)
   - Full filesystem semantics
   - Path-based file operations
   - Directory support with indexing
   - Metadata tracking

4. **FUSE Mount** (`mount` module, optional feature)
   - Mount as real Linux/Mac filesystem
   - Standard filesystem operations
   - Transparent wave encoding

### Wave Encoding

Data is encoded as waves using Complex64 numbers:
- Frequency component encodes data patterns
- Phase component adds tamper detection
- Interference patterns provide natural compression
- Configurable base frequency (default: 1.618 golden ratio)

### File Structure

```
/path/to/mem8.m8/
├── data.waves      # Append-only wave storage
├── index.m8        # Path → signature mapping
└── metadata.json   # Filesystem metadata
```

## Common Development Tasks

### Adding New Storage Features

1. Implement in `WaveStorage` struct (lib.rs)
2. Expose through `Mem8Lite` API (lite module)
3. Add filesystem semantics in `Mem8Fs`
4. Update FUSE operations if applicable

### Implementing Missing Modules

The project currently needs:
- `src/lite.rs` - Simple key-value storage implementation
- `benches/wave_ops.rs` - Performance benchmarks

### Performance Optimization Areas

- SIMD operations (behind `simd` feature flag)
- Batch operations for multiple files
- Cache tuning in `WaveStorage`
- Memory-mapped I/O optimizations

## Port Conventions

When integrating with other 8b-is services:
- 8420: MEM8 API endpoints
- 8422: Cheet API
- 8424: Internal services
- 8428: LLM endpoints

## Testing Approach

1. Unit tests for wave encoding/decoding
2. Integration tests for filesystem operations
3. Performance benchmarks comparing to traditional storage
4. FUSE mount tests (when feature enabled)

## Key Implementation Notes

- **Missing lite module**: The `lite` module is referenced but not implemented. Create `src/lite.rs` with `Mem8Lite` struct and `WavePacket` for simple storage.

- **Missing benchmark**: Remove the benchmark from Cargo.toml or create `benches/wave_ops.rs`.

- **Wave frequency tuning**: Different frequencies affect performance:
  - 1.618 (golden ratio): Balanced performance
  - Higher (>5.0): Better compression, slower
  - Lower (<1.0): Faster access, less compression

- **Error handling**: Use `anyhow::Result` for APIs, custom errors where needed

- **Thread safety**: Use `RwLock` for shared state, filesystem operations should be thread-safe

## Marine Algorithm Integration

The repository now includes the **Marine algorithm** for salience detection and temporal perspective storage, based on the research paper. This adds "sense of wonder" detection to stored data!

### Marine Algorithm Features
- **Salience Detection**: O(1) peak detection with jitter analysis
- **Temporal Perspectives**: Store memories from different viewpoints
  - Diary Writer: First-person emotional depth
  - Shared Witness: Friend/sibling perspective with overlap
  - Third Party: Objective observer viewpoint
- **Wonder Detection**: Finds moments of beauty in data patterns
- **Emotion Signatures**: Detects emotional patterns (Wondrous, Energetic, Peaceful, Musical, Flowing)

### Audio Processing Support
Multiple sample rates supported:
- 16kHz (Phone quality)
- 22.05kHz (Broadcast)
- 44.1kHz (CD quality - optimal)
- 48kHz (DVD)
- 96kHz (Studio)
- 192kHz (Audiophile)

### Running Marine Examples
```bash
# Audio processing with temporal perspectives
cargo run --example audio_marine

# The example demonstrates:
# - Same audio stored from 3 perspectives
# - Different "wonder" detection thresholds
# - Emotional signature extraction
# - Cross-perspective analysis
```

### Marine Module API
```rust
use mem8_fs_lite::{MarineProcessor, AudioProcessor, SampleRate};

// Process audio with Marine algorithm
let mut processor = MarineProcessor::for_audio(44100.0);
processor.wonder_threshold = 0.7;  // Adjust sensitivity

// Multi-format audio processing
let format = AudioFormat::cd_quality();
let mut audio_proc = AudioProcessor::new(format, "/tmp/audio.m8")?;
```

## Integration with MEM8 Ecosystem

While this is the "lite" version without consciousness:
- Wave encoding is compatible with full MEM8
- Can be used as storage backend for MEM8 systems
- Signatures are interchangeable between versions
- Marine algorithm adds temporal perspective layer