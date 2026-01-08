# Prontua Backend

A domain-centric modular monolith backend built with Rust, featuring user authentication, JWT-based authorization, and PostgreSQL persistence.

## Overview

Prontua Backend is a modern REST API built with **Axum** (async Rust web framework) and **Tokio** runtime. It implements a clean architecture with domain-driven design principles, focusing on modularity and maintainability.

## Tech Stack

- **Framework**: Axum 0.7 with Tokio runtime
- **Database**: PostgreSQL with SQLx for type-safe queries
- **Authentication**: JWT (jsonwebtoken) with bcrypt password hashing
- **Serialization**: Serde for JSON handling
- **Logging**: Tracing and tracing-subscriber for structured logging
- **Language**: Rust

## Project Structure

```
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library root with module exports
├── infrastructure/         # Infrastructure layer (database, config, external services)
│   ├── config/            # Configuration management
│   └── database/          # Database connection and setup
├── modules/               # Domain modules (features)
│   ├── auth/              # Authentication & authorization
│   │   ├── services/      # Business logic
│   │   └── repository/    # Data access
│   └── users/             # User management
│       ├── objects/       # Value objects (Email, Password)
│       ├── services/      # User business logic
│       ├── repository/    # User data access
│       └── user_model.rs  # User domain model
└── shared/                # Shared utilities and common code
    ├── app_state.rs       # Global application state
    ├── errors/            # Error handling
    ├── objects/           # Shared value objects
    └── kernel/            # Core utilities
```

## Getting Started

### Prerequisites

- **Rust**: 1.70+ (install from [rustup.rs](https://rustup.rs/))
- **Git**: For version control
- **SQLx CLI**: For running database migrations (see installation below)
- **Docker & Docker Compose**: For running PostgreSQL and Valkey in development (optional but recommended)

### Installation

#### Step 1: Install SQLx CLI

SQLx CLI is required to run database migrations. Install it with:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

This installs the SQLx command-line tool with PostgreSQL support only (without requiring Docker or other databases).

#### Step 2: Clone and Setup

1. **Clone the repository** (if applicable):

   ```bash
   git clone <repository-url>
   cd prontua/backend
   ```

2. **Setup environment variables**:

   ```bash
   cp .env.example .env
   ```

3. **Update `.env` with your configuration**:

   ```env
   # Server Configuration
   SERVER_HOST=127.0.0.1
   SERVER_PORT=3000

   # Database Configuration (when using docker-compose)
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres

   # JWT Configuration
   JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
   JWT_EXPIRY_HOURS=24
   ```

#### Step 3: Setup Database and Run Migrations

The project includes a `docker-compose.yml` file to run PostgreSQL and Valkey (Redis) services for development:

1. **Start services with Docker Compose**:

   ```bash
   docker-compose up -d
   ```

   This starts:
   - PostgreSQL on `localhost:5432` (credentials: `postgres:postgres`)
   - Valkey (Redis) on `localhost:6379`

2. **Run all pending migrations**:

   ```bash
   sqlx migrate run
   ```

   This will execute all migration files in the `migrations/` directory in order, creating the necessary tables and schemas.

#### Step 4: Build and Run

```bash
cargo build
cargo run
```

The server will start on `http://127.0.0.1:3000`

### Docker Compose for Development

The `docker-compose.yml` file provides auxiliary services for local development:

```yaml
services:
  valkey:      # Redis-compatible cache (port 6379)
  postgres:    # PostgreSQL database (port 5432)
```

**Using Docker Compose:**

- **Start services**: `docker-compose up -d`
- **View logs**: `docker-compose logs -f postgres` or `docker-compose logs -f valkey`
- **Stop services**: `docker-compose down`
- **Remove volumes** (reset database): `docker-compose down -v`

**Environment Variables** for docker-compose services:
- PostgreSQL: `postgres://postgres:postgres@localhost:5432/postgres`
- Valkey: `redis://localhost:6379`

Update your `.env` file to match these connection strings when using docker-compose.

### Database Migrations

Migrations are SQL scripts located in the `migrations/` directory. Each migration is prefixed with a version number (e.g., `001_`, `002_`) and contains DDL statements.

#### Running Migrations

To run migrations:

```bash
sqlx migrate run
```

#### Creating a New Migration

To create a new migration file:

```bash
sqlx migrate add <migration_name>
```

This creates a timestamped migration file in the `migrations/` directory that you can edit.

#### Reverting Migrations

SQLx doesn't support automatic rollbacks. To revert changes:

1. Edit the migration file to add `DROP TABLE` or other revert logic
2. Create a new migration with the revert changes
3. Run `sqlx migrate run` again

## Configuration

### Environment Variables

| Variable           | Description                        | Default                                               |
| ------------------ | ---------------------------------- | ----------------------------------------------------- |
| `SERVER_HOST`      | Server binding address             | `127.0.0.1`                                           |
| `SERVER_PORT`      | Server port                        | `3000`                                                |
| `DATABASE_URL`     | PostgreSQL connection string       | `postgres://postgres:postgres@localhost:5432/prontua` |
| `JWT_SECRET`       | Secret key for JWT signing         | `your-super-secret-jwt-key`                           |
| `JWT_EXPIRY_HOURS` | JWT token expiration time in hours | `24`                                                  |

## Development

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

### Code Organization Best Practices

This project follows domain-driven design:

- **Modules**: Self-contained feature domains (auth, users, etc.)
- **Services**: Business logic and orchestration
- **Repositories**: Data access abstraction
- **Objects**: Value objects with validation (Email, Password)
- **Infrastructure**: External concerns (DB, config)
- **Shared**: Cross-cutting utilities and common types

### Adding a New Feature

1. Create a new module in `src/modules/feature_name/`
2. Structure it with: `services/`, `repository/`, `objects/`, and `mod.rs`
3. Implement repository traits for database access
4. Add business logic in services
5. Create handlers and register routes in `main.rs`

## Error Handling

Errors are handled through a custom `AppError` type defined in `src/shared/errors/`:

- Provides consistent error responses
- Maps domain errors to HTTP status codes
- Supports error context and messages

## Logging

The application uses `tracing` for structured logging:

```bash
# Default level: DEBUG
# Set via environment variable:
RUST_LOG=info cargo run
```

## Security Considerations

- **Passwords**: Hashed with bcrypt before storage
- **Tokens**: JWT signed with secret key (change in production!)
- **Database**: Use environment-based connection strings
- **CORS**: Tower middleware for cross-origin handling
- **Input Validation**: Value objects enforce constraints

⚠️ **Production Checklist**:

- [ ] Change `JWT_SECRET` to a strong random value
- [ ] Use HTTPS for all endpoints
- [ ] Set appropriate CORS policies
- [ ] Configure database connection pooling
- [ ] Enable rate limiting
- [ ] Setup comprehensive logging and monitoring
- [ ] Use environment-specific configurations

## Performance

- **Async/Await**: Tokio runtime enables high concurrency
- **Connection Pooling**: SQLx handles efficient DB connections
- **Type Safety**: Compile-time guarantees reduce runtime errors
