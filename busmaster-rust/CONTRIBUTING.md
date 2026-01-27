# Contributing to BUSMASTER Rust

Thank you for your interest in contributing to BUSMASTER Rust! This document provides guidelines and information for contributors.

## Development Model

This is an **AI-assisted development project**. Code is primarily written by AI agents with human review and guidance. However, human contributions are welcome!

## Getting Started

### Prerequisites

1. **Rust Toolchain**: Install via [rustup](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Required Components** (installed automatically via rust-toolchain.toml):
   - rustfmt
   - clippy
   - rust-analyzer
   - llvm-tools-preview

3. **Optional Tools**:
   ```bash
   # Code coverage
   cargo install cargo-llvm-cov
   
   # Security audit
   cargo install cargo-audit
   
   # License checking
   cargo install cargo-deny
   
   # Benchmarking
   cargo install cargo-criterion
   ```

### Setting Up

```bash
# Clone the repository
git clone https://github.com/rbei-etas/busmaster.git
cd busmaster/source/busmaster-rust

# Verify setup
cargo check
cargo test
cargo clippy
```

## Code Style

### Formatting

All code must be formatted with `rustfmt`:

```bash
cargo fmt --all
```

Configuration is in `rustfmt.toml`. Key settings:
- Max line width: 100 characters
- 4-space indentation
- Grouped imports

### Linting

All code must pass `clippy` with no warnings:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Configuration is in `clippy.toml`. We use pedantic lints.

### Documentation

All public APIs must be documented:

```rust
/// Creates a new CAN frame with a standard (11-bit) identifier.
///
/// # Arguments
///
/// * `id` - The CAN identifier (0x000 to 0x7FF)
/// * `data` - The data bytes (0-8 bytes)
///
/// # Returns
///
/// A `Result` containing the new frame or an error if validation fails.
///
/// # Examples
///
/// ```
/// use busmaster_core::CanFrame;
///
/// let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4])?;
/// assert_eq!(frame.id(), 0x123);
/// # Ok::<(), busmaster_core::BusmasterError>(())
/// ```
///
/// # Errors
///
/// Returns `BusmasterError::InvalidCanId` if the ID exceeds 0x7FF.
/// Returns `BusmasterError::InvalidDataLength` if data exceeds 8 bytes.
pub fn new_standard(id: u32, data: &[u8]) -> Result<Self> {
    // ...
}
```

## Testing

### Unit Tests

Write unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_frame_creation() {
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3]).unwrap();
        assert_eq!(frame.id(), 0x123);
    }

    #[test]
    fn test_invalid_id_rejected() {
        let result = CanFrame::new_standard(0x800, &[]);
        assert!(result.is_err());
    }
}
```

### Property-Based Tests

Use `proptest` for property-based testing:

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn roundtrip_encoding(id in 0u32..=0x7FF, data in prop::collection::vec(any::<u8>(), 0..=8)) {
            let frame = CanFrame::new_standard(id, &data).unwrap();
            let encoded = encode(&frame);
            let decoded = decode(&encoded).unwrap();
            prop_assert_eq!(frame, decoded);
        }
    }
}
```

### Running Tests

```bash
# All tests
cargo test --all-features --workspace

# Specific crate
cargo test -p busmaster-core

# With output
cargo test -- --nocapture

# Coverage
cargo llvm-cov --all-features --workspace --html
```

## Pull Request Process

### Before Submitting

1. **Create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the code style guidelines

3. **Run all checks**:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features --workspace
   cargo doc --no-deps --all-features --workspace
   ```

4. **Update documentation** if needed

5. **Add changelog entry** in `docs/CHANGELOG.md`

### Submitting

1. Push your branch
2. Create a Pull Request against `develop` branch
3. Fill out the PR template
4. Wait for CI to pass
5. Request review

### PR Requirements

- [ ] All CI checks pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Tests added for new functionality
- [ ] Documentation updated
- [ ] Changelog entry added

## Architecture Guidelines

### Crate Dependencies

```
busmaster-core (no deps)
    ↑
busmaster-proto, busmaster-dil, busmaster-db, busmaster-log
    ↑
busmaster-hardware (depends on dil)
    ↑
busmaster-engine (depends on all above)
    ↑
busmaster-cli, busmaster-tui (depends on engine)
```

### Safety Rules

1. **busmaster-core** and **busmaster-dil**: `#![forbid(unsafe_code)]`
2. **busmaster-hardware**: Unsafe allowed only for FFI, with SAFETY comments
3. All unsafe blocks must have `// SAFETY:` comments explaining why it's safe

### Error Handling

Use `thiserror` for error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Communication

- **Issues**: Use GitHub Issues for bugs and feature requests
- **Discussions**: Use GitHub Discussions for questions

## License

By contributing, you agree that your contributions will be licensed under the LGPL-3.0 License.
