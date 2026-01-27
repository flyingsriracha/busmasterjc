# ADR-001: Cargo Workspace Structure

## Status
Accepted

## Date
2026-01-26

## Context
We need to establish the project structure for the BUSMASTER Rust conversion. The design document specifies a modular architecture with multiple crates, but we need to decide on the exact workspace configuration.

## Decision
We will use a Cargo workspace with the following structure:

1. **Workspace root** at `source/busmaster-rust/`
2. **All crates** under `crates/` subdirectory
3. **Workspace inheritance** for common settings (version, edition, authors)
4. **Shared dependencies** defined at workspace level

### Crate Organization (MVP)
```
crates/
├── busmaster-core/      # Core types, no dependencies
├── busmaster-proto/     # Protocol implementations
├── busmaster-dil/       # Driver interface traits
├── busmaster-hardware/  # Driver implementations
├── busmaster-db/        # Database parsers
├── busmaster-log/       # Log format handlers
├── busmaster-engine/    # Main orchestrator
├── busmaster-cli/       # CLI application
├── busmaster-tui/       # TUI application
└── busmaster-platform/  # Platform abstraction
```

### Dependency Rules
1. `busmaster-core` has no internal dependencies
2. All other crates depend on `busmaster-core`
3. `busmaster-hardware` depends on `busmaster-dil`
4. `busmaster-engine` depends on all library crates
5. Applications depend on `busmaster-engine`

## Consequences

### Positive
- Clear separation of concerns
- Independent crate testing
- Parallel compilation
- Easy to add new crates (drivers, protocols)
- Workspace inheritance reduces duplication

### Negative
- More files to maintain
- Need to manage inter-crate dependencies carefully
- Longer initial setup time

### Risks
- Circular dependencies if not careful
- Version synchronization across crates

## Alternatives Considered

1. **Single crate with modules**: Simpler but less modular
2. **Separate repositories**: Too fragmented for this project
3. **Flat crate structure**: Harder to navigate

## References
- Design document section 2 (Crate Structure)
- Requirements section 10.3 (Code Quality)
