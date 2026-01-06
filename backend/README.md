# Prontua Backend

A production-ready Rust backend using **domain-centric modular monolith architecture** (vertical slice pattern) with PostgreSQL, JWT authentication, and type-safe SQL with SQLx.

## Architecture

This project follows a **vertical slice / domain-centric architecture** inspired by Milan Jovanović's approach. Each domain feature is organized as a vertical slice containing its own handlers, services, and models.

### Key Principles

1. **Vertical Slices**: Features are organized by domain, not by technical layers
2. **Domain-First**: Business logic lives in domain entities and value objects
3. **Decoupled Features**: Auth and User are separate but share domain models
4. **Type Safety**: Extensive use of value objects for domain concepts

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed architecture decisions and trade-offs.

### Directory Structure

```
src/
├── domain/               # Shared domain layer (entities, value objects)
│   ├── entities/         # User, etc.
│   └── value_objects/    # Email, Password, etc.
├── features/             # Vertical slices (features)
│   └── auth/             # Authentication feature
│       ├── handlers/      # HTTP handlers
│       ├── services/      # Auth-specific services
│       └── models/        # Request/response DTOs
├── infrastructure/       # Technical infrastructure
│   ├── database/         # PostgreSQL connection
│   └── config/           # Configuration
└── shared/               # Cross-cutting concerns
    ├── errors/           # Error handling
    └── kernel/           # Core types and traits
```

## Features

- **Authentication System**
  - Sign-up with email validation and password strength requirements
  - Sign-in with password verification
  - JWT access tokens with automatic refresh
  - Refresh token rotation for security

- **Domain-Driven Design**
  - Email value object with validation
  - Password value object with strength validation
  - User entity with business logic
  - Result type for ergonomic error handling

- **Database**
  - PostgreSQL with SQLx (compile-time checked SQL)
  - Migrations system
  - Connection pooling
  - Indexes for performance

- **Security**
  - Bcrypt password hashing
  - JWT token generation and validation
  - Refresh token management with hashing
  - Input validation at domain level

- **Production Ready**
  - Structured logging with tracing
  - Comprehensive error handling
  - Environment-based configuration
  - Type safety throughout

## Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- SQLx CLI (for migrations): `cargo install sqlx-cli`

## Quick Start

### 1. Clone and Setup

```bash
# Copy environment template
cp .env.example .env

# Edit .env with your database credentials
# Default: postgres://postgres:postgres@localhost:5432/prontua
```

### 2. Create Database

```bash
# Using PostgreSQL CLI
psql -U postgres -c "CREATE DATABASE prontua;"

# Or using your PostgreSQL client
```

### 3. Run Migrations

```bash
# Using sqlx CLI
sqlx migrate run

# Or manually using psql
psql -U postgres -d prontua -f migrations/001_create_users_table.sql
psql -U postgres -d prontua -f migrations/002_create_refresh_tokens_table.sql
```

### 4. Run Server

```bash
cargo run
```

The server will start on `http://127.0.0.1:3000`

## API Endpoints

### Authentication

#### Sign Up
```bash
POST /api/auth/sign-up

{
  "email": "user@example.com",
  "password": "SecurePass123"
}

Response:
{
  "access_token": "eyJ0eXAiOiJKV1QiLC...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLC...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

#### Sign In
```bash
POST /api/auth/sign-in

{
  "email": "user@example.com",
  "password": "SecurePass123"
}

Response:
{
  "access_token": "eyJ0eXAiOiJKV1QiLC...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLC...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

#### Refresh Token
```bash
POST /api/auth/refresh

{
  "refresh_token": "eyJ0eXAiOiJKV1QiLC..."
}

Response:
{
  "access_token": "eyJ0eXAiOiJKV1QiLC...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLC...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

## Password Requirements

Passwords must meet the following criteria:
- Minimum 8 characters
- Maximum 128 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one digit

## Email Validation

Emails are validated using RFC 5322 compliant regex. Valid email formats include:
- `user@example.com`
- `user.name@example.co.uk`
- `user+tag@example.com`

## Configuration

Configure via environment variables (see `.env.example`):

```env
# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/prontua

# JWT
JWT_SECRET=your-super-secret-key-min-32-chars-recommended
JWT_EXPIRY_HOURS=24
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_valid_email
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Check types
cargo check
```

## Project Conventions

### Naming

- **Domain models** (entities, value objects): PascalCase
- **Functions/methods**: snake_case
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case

### Error Handling

All fallible operations return `AppResult<T>` which is `Result<T, AppError>`. The `AppError` type handles conversion to HTTP responses automatically.

```rust
use prontua_backend::AppResult;

pub async fn my_handler() -> AppResult<Json<Response>> {
    // Your code here
    // Errors are automatically converted to HTTP responses
}
```

### Adding New Features

1. Create a new feature module in `src/features/[feature_name]/`
2. Structure as: `handlers/`, `services/`, `models/`
3. Share domain models, not implementations
4. Keep handlers thin - logic goes in services

Example:
```
src/features/users/
├── handlers/mod.rs      # HTTP handlers
├── services/mod.rs      # Business logic
└── models/mod.rs        # Request/response DTOs
```

## Security Considerations

- **Passwords**: Always hashed with bcrypt (cost factor 12)
- **Tokens**: JWT with HS256 algorithm
- **Refresh tokens**: Hashed in database and rotated on use
- **Input validation**: All inputs validated at domain level
- **SQL injection**: Type-safe SQL with SQLx

## Performance

- Connection pooling with configurable max connections
- Database indexes on frequently queried columns
- Lazy validation (only when needed)
- Efficient JWT validation

## Next Steps

- [ ] Add API documentation (OpenAPI/Swagger)
- [ ] Implement user profile feature
- [ ] Add email verification
- [ ] Implement password reset flow
- [ ] Add rate limiting
- [ ] Set up Docker and CI/CD

## License

MIT

## References

- [Vertical Slice Architecture: Where Does the Shared Logic Live?](https://www.milanjovanovic.tech/blog/vertical-slice-architecture-where-does-the-shared-logic-live)
- [Axum Framework](https://github.com/tokio-rs/axum)
- [SQLx](https://github.com/launchbadge/sqlx)
- [Jsonwebtoken](https://github.com/Keats/jsonwebtoken)
