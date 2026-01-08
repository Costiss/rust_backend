# Prontua Backend - Agent Guidelines

This document provides essential information for agentic coding systems operating in this repository.

## Quick Reference

**Language**: Rust 1.70+  
**Framework**: Axum + Tokio (async web framework)  
**Database**: PostgreSQL (via SQLx)  
**Architecture**: Domain-driven modular monolith

## Build & Run Commands

```bash
# Development watch mode (auto-rebuild on changes)
cargo watch -x run

# Build the project
cargo build

# Build optimized release
cargo build --release

# Run the server
cargo run

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a single test
cargo test <test_name> -- --exact

# Format code
cargo fmt

# Check code style and common mistakes
cargo clippy

# Generate OpenAPI specification
cargo run --bin generate-openapi

# Run database migrations
sqlx migrate run

# Create a new migration
sqlx migrate add <migration_name>

# Watch mode with logging
RUST_LOG=debug cargo watch -x run
```

## Project Structure

```
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library root
├── infrastructure/         # External concerns (DB, config, cache)
├── modules/                # Feature domains (auth, users, etc.)
│   ├── auth/              # Authentication & authorization
│   └── users/             # User management
└── shared/                 # Cross-cutting utilities
    ├── app_state.rs       # Global application state
    ├── errors/            # Error handling (AppError, AppResult)
    ├── objects/           # Value objects (Email, Password)
    ├── cache/             # Caching abstraction
    └── kernel/            # Core utilities & traits
```

## Code Style Guidelines

### Imports

- Group imports in three sections: std, external crates, internal modules
- Use absolute paths via `crate::` prefix for internal imports
- Avoid wildcard imports; be explicit about what you're using
- Place module declarations at top, then imports

```rust
use crate::shared::errors::AppError;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
```

### Formatting

- Use `cargo fmt` for automatic formatting (enforced by default in Rust)
- Line width: Follow Rust defaults (100 chars)
- Indentation: 4 spaces (automatic)

### Types & Type Annotations

- Always specify return types for public functions
- Use `AppResult<T>` for fallible operations (defined in `src/shared/errors/mod.rs`)
- Implement `Debug`, `Clone` for domain types where appropriate
- Use `#[derive(...)]` for standard traits (Debug, Serialize, Deserialize)
- Leverage Rust's type system; avoid `String` where specific types work

```rust
pub async fn get_user(&self, user_id: Uuid) -> AppResult<User> { ... }
```

### Naming Conventions

- **Functions**: `snake_case`
- **Types/Structs**: `PascalCase`
- **Constants**: `UPPER_SNAKE_CASE`
- **Modules**: `snake_case` (lowercase with underscores)
- **Private functions**: prefix with `_` if unused or for internal use
- **Async functions**: suffix with `_async` only if necessary for clarity

### Error Handling

- Return `AppResult<T>` from fallible functions (never `unwrap()` in library code)
- Define custom error enums for domain-specific errors with `Display` implementation
- Implement `From<CustomError> for AppError` to map custom errors to HTTP responses
- Specify error code and HTTP status in the `From` impl, not in helper methods
- Log errors using `tracing::error!()`, `tracing::warn!()`
- Always provide unique identifiers for error codes `AppError.new(message, "ERROR_CODE", StatusCode::BAD_REQUEST)`

```rust
// Good: Define custom error enum
pub enum EmailError {
    InvalidFormat,
    AlreadyExists,
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailError::InvalidFormat => write!(f, "Invalid email format"),
            EmailError::AlreadyExists => write!(f, "Email already registered"),
        }
    }
}

impl From<EmailError> for AppError {
    fn from(err: EmailError) -> Self {
        let message = err.to_string();
        match err {
            EmailError::InvalidFormat => {
                AppError::new(message, "INVALID_EMAIL_FORMAT", StatusCode::BAD_REQUEST)
            }
            EmailError::AlreadyExists => {
                AppError::new(message, "EMAIL_ALREADY_EXISTS", StatusCode::CONFLICT)
            }
        }
    }
}

pub fn create_user(&self, email: &str) -> AppResult<User> {
    let _email = Email::new(email)?;  // Automatically converts EmailError → AppError
    // ...
}

// Bad: Don't use generic AppError helpers like this
// AppError::validation("msg") ❌ Avoid this
```

### Testing

- Place tests in same file with `#[cfg(test)]` module
- Use `#[test]` for unit tests
- Test function names should describe what they test: `test_<function>_<scenario>`
- Keep tests focused and independent
- **Always use AAA pattern**: Arrange → Act → Assert
- **Always mock I/O function calls** (database, external services, cache, file system)
- Never make real network/database calls in unit tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation_rejects_invalid_format() {
        // Arrange
        let invalid_email = "invalid-email";

        // Act
        let result = Email::new(invalid_email);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email format");
    }

    #[test]
    fn test_create_user_returns_error_when_email_exists() {
        // Arrange: Setup mock repository that returns existing user
        let mock_repo = MockUserRepository::new();
        mock_repo.expect_find_by_email()
            .returning(|_| Ok(Some(User::stub())));
        let service = UserService::new(mock_repo);

        // Act
        let result = service.create_user("existing@example.com", "hash").await;

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "EMAIL_ALREADY_EXISTS");
    }
}
```

**Mocking Guidelines:**

- Use trait abstractions for testability (Repository pattern)
- Mock cache layers using `MockCacheService` (see `src/shared/cache/mock.rs`)
  - In-memory store for testing without Redis/Valkey
  - Call logging for verifying interactions: `mock_cache.assert_call("set")`
- Mock external service calls (APIs, file systems)
- Never test through real I/O in unit tests
- Avoid integration tests (E2E automated tests)

```rust
// Example: Using MockCacheService in tests
#[tokio::test]
async fn test_user_service_caches_result() {
    // Arrange
    let mock_cache = MockCacheService::new();
    let mock_repo = MockUserRepository::new();
    mock_repo.expect_find_by_id()
        .returning(|_| Ok(User::stub()));
    let service = UserService::new(mock_repo, Arc::new(mock_cache.clone()));

    // Act
    let user = service.get_user("123").await;

    // Assert
    assert!(user.is_ok());
    mock_cache.assert_call("set");  // Verify cache was written
}
```

### Documentation

- Add doc comments for public modules, types, and functions using `///`
- Describe purpose, parameters, return values, and errors
- Include examples for complex behavior

```rust
/// Authenticates a user with email and password
///
/// # Arguments
/// * `email` - User's email address
/// * `password` - User's plain-text password
///
/// # Returns
/// * `Ok(token)` - JWT token on success
/// * `Err(AppError)` - Authentication error
pub async fn authenticate(&self, email: &str, password: &str) -> AppResult<String> {
    // ...
}
```

### Async Code

- Use `async fn` and `await` syntax
- Prefer async traits using `#[async_trait]`
- Keep async functions in Tokio runtime (`#[tokio::main]` for main, `#[tokio::test]` for tests)

### Module Organization

- **services/**: Business logic and orchestration
- **repository/**: Data access abstraction (traits)
- **objects/**: Domain value objects (Email, Password)
- **handlers/**: HTTP request handlers and routing
- Keep modules focused on a single responsibility

## Configuration & Environment

- Load config from `.env` file using `dotenv` crate
- Never commit `.env` files; use `.env.example` as template
- Required variables (see `.env.example`):
  - `DATABASE_URL` - PostgreSQL connection
  - `REDIS_URL` - Redis/Valkey connection
  - `JWT_SECRET` - JWT signing key
  - `SERVER_HOST`, `SERVER_PORT` - Server binding

## Database

- Use SQLx for type-safe queries
- Write migrations in `migrations/` directory
- Naming: `YYYYMMDDHHMMSS_description.sql`
- Run migrations with: `sqlx migrate run`
- All database access through repository pattern

## Testing Single Tests

```bash
# Run a specific test by name
cargo test test_email_validation -- --exact

# Run tests in a specific module
cargo test modules::auth::

# Run with output (println!, eprintln!)
cargo test -- --nocapture

# Run in release mode
cargo test --release

# Run ignored tests
cargo test -- --ignored
```

## Key Patterns

### Result Type

All fallible operations return `AppResult<T>` = `Result<T, AppError>`

### Repository Pattern

Abstract data access with trait implementations:

```rust
pub trait UserRepository {
    async fn find_by_email(&self, email: &Email) -> AppResult<User>;
}
```

### Value Objects

Encapsulate validation in types:

```rust
pub struct Email(String); // Email is always valid
```

### Error Mapping

Implement `From` for automatic error conversion:

```rust
impl From<sqlx::Error> for AppError { ... }
impl From<jsonwebtoken::errors::Error> for AppError { ... }
```

## Performance Notes

- Tokio async runtime handles high concurrency
- SQLx connection pooling is automatic
- Redis caching for hot data
- Compile-time type checking prevents runtime errors
- Use `--release` builds for production

## Common Commands for Agents

```bash
# Full workflow
cargo fmt && cargo clippy && cargo test && cargo build

# Check for issues
cargo clippy -- -D warnings

# Explore codebase
grep -r "pub fn" src/  # Find public functions
find src/ -name "*.rs" # List all Rust files

# Run specific test module
cargo test modules::users:: -- --nocapture
```

---

_Last updated: Jan 2026_  
_For questions about Prontua architecture, see README.md_
