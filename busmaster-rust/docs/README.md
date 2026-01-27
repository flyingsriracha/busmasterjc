# BUSMASTER Rust Development Documentation

This folder contains development documentation, decision logs, and traceability information for the BUSMASTER Rust conversion project.

## Documentation Structure

```
docs/
├── README.md                    # This file
├── ARCHITECTURE.md              # Architecture decisions and rationale
├── CHANGELOG.md                 # Development changelog by task
├── decisions/                   # Architecture Decision Records (ADRs)
│   └── 001-workspace-structure.md
├── traceability/                # Requirements traceability
│   └── requirements-mapping.md
└── dev-journal/                 # Daily development journal
    └── 2026-01-26.md
```

## Quick Links

- [Architecture Overview](ARCHITECTURE.md)
- [Development Changelog](CHANGELOG.md)
- [Requirements Mapping](traceability/requirements-mapping.md)

## Project References

- **Requirements**: `../.kiro/specs/busmaster-rust-conversion/requirements.md`
- **Design**: `../.kiro/specs/busmaster-rust-conversion/design.md`
- **Tasks**: `../.kiro/specs/busmaster-rust-conversion/tasks.md`

## Development Model

This is an **AI-ONLY** development project. All code is written by AI agents with iterative refinement.

**Key Principles:**
1. Rapid prototyping with iterative refinement
2. Test-driven development with property-based testing
3. Continuous integration with automated quality gates
4. Document decisions for traceability and debugging
