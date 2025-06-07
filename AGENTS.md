# Project Agents.md Guide for AI Agents

This Agents.md file provides comprehensive guidance for AI agents working with the rkit codebase, a Rust CLI toolkit for Git repository management.

## Project Structure for AI Navigation

- `/src`: Core source code
  - `/commands`: CLI command implementations
  - `main.rs`: Application entry point and CLI setup
  - `lib.rs`: Library exports
  - `error.rs`: Error handling and custom error types
  - `config.rs`: Configuration management
  - `cache.rs`: Caching functionality
- `/benches`: Benchmark tests
- `/assets`: Static assets and resources
- `/etc`: Configuration templates and defaults
- `/target`: Build artifacts (should not be modified directly)

## Coding Conventions for AI

### General Conventions

- Use Rust 2021 edition for all new code
- Follow Rust's official style guide and formatting conventions
- Use meaningful variable and function names
- Add documentation comments for public APIs using `///` syntax
- Keep functions focused and single-purpose
- Use appropriate error handling with `thiserror` and `anyhow`

### Error Handling Guidelines

- Use custom error types defined in `error.rs`
- Implement proper error conversion traits
- Provide meaningful error messages
- Use `?` operator for error propagation
- Handle errors at appropriate levels

### Configuration Management

- Follow the existing configuration structure in `config.rs`
- Use YAML for configuration files
- Support platform-specific configurations
- Implement proper configuration validation
- Use environment variables when appropriate

### Performance Considerations

- Use `rayon` for parallel processing where beneficial
- Implement proper benchmarking in `/benches`
- Consider memory usage and resource management
- Use appropriate data structures for performance
- Profile code before optimization

## Testing Requirements

AI agents should run tests with the following commands:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench

# Run with logging
RUST_LOG=debug cargo test
```

## Pull Request Guidelines

When AI agents help create a PR, please ensure it:

1. Includes a clear description of the changes
2. References any related issues
3. Ensures all tests pass
4. Includes performance impact analysis if relevant
5. Updates documentation as needed
6. Follows Rust's safety and correctness principles
7. Ensure all commits adhere to [Conventional Commits specification](https://www.conventionalcommits.org/en/v1.0.0/#specification)

## Programmatic Checks

Before submitting changes, run:

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy 

# Run tests
cargo test

# Check documentation
cargo doc

# Build in release mode
cargo build --release
```

All checks must pass and all warnings should be fixed before code can be merged.

## Dependencies

- Use the versions specified in Cargo.toml
- Document any new dependencies added
- Consider the impact on binary size
- Ensure compatibility with existing dependencies

## Documentation

- Keep README.md up to date
- Document all public APIs
- Include examples in documentation
- Update installation instructions if needed
- Document any breaking changes

## Security Considerations

- Follow Rust's security best practices
- Handle sensitive data appropriately
- Validate all user input
- Use secure defaults
- Document security-related decisions

## Performance Benchmarks

- Maintain existing benchmarks in `/benches`
- Add new benchmarks for new features
- Compare performance against previous versions
- Document performance characteristics
- Consider edge cases in benchmarks 