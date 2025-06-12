# Coding Standards

This document provides guidance on how code should be structured and reviewed across the repository. Following these practices helps keep the codebase maintainable and approachable for all contributors.

## Design Principles

- **SOLID** – Apply the five SOLID principles when structuring modules and traits so each component has a clear responsibility and can evolve independently.
- **Separation of Concerns** – Keep business logic, CLI/UI, and data storage layers distinct. Aim for loosely coupled modules with well-defined interfaces.
- **KISS and DRY** – Prefer simple solutions and avoid unnecessary duplication. Reuse common helpers and extract shared code into libraries when practical.
- **Clean Architecture** – Favor a layered architecture where core domain types are decoupled from infrastructure details. Dependency injection (via traits) can help keep boundaries clear.
- **Use of Design Patterns** – Where appropriate, apply common patterns (builder, factory, command, etc.) to clarify intent and enable extension without modification.
- **Entry Points** – `main` functions should be minimal bootstrappers that delegate to library code. Keep implementations in modules so they can be tested in isolation.

## Rust Style

- Format all code with `cargo fmt` before committing. The repository provides a `.rustfmt.toml` configuration for consistency.
- Run `cargo clippy` and address warnings where practical. Lints help enforce idiomatic Rust and catch common mistakes.
- Prefer explicit error types over panics in library code. Use the `thiserror` crate or a similar approach for ergonomic error handling.
- Document public functions and modules using Rustdoc comments so generated docs remain helpful.
- Maintain **100% unit test coverage** for all code merged into the repository. Coverage reports should be generated and verified in CI.

Adhering to these standards keeps the project professional and maintainable as it grows.
