# Codebase Reference

Quick reference for navigating and understanding the codebase structure.

## File Structure

```
backend/
├── src/                          # All source code
│   ├── lib.rs                    # Library root - exports all modules
│   ├── main.rs                   # Binary root - starts the server
│   │
│   ├── domain/                   # Business logic layer (shared)
│   │   ├── mod.rs                # Domain module exports
│   │   ├── entities/
│   │   │   ├── mod.rs            # Entity exports
│   │   │   └── user.rs           # User entity (core domain model)
│   │   │                          # Size: ~80 lines
│   │   │                          # Tests: User creation, verification, updates
│   │   └── value_objects/
│   │       ├── mod.rs            # Value object exports
│   │       ├── email.rs          # Email validation with regex
│   │       │                      # Size: ~100 lines
│   │       │                      # Tests: Valid/invalid formats
│   │       └── password.rs        # Password validation (8+ chars, mixed case, digit)
│   │                              # Size: ~90 lines
│   │                              # Tests: Strength requirements
│   │
│   ├── features/                 # Features (vertical slices)
│   │   ├── mod.rs                # Features module exports
│   │   └── auth/                 # Authentication feature
│   │       ├── mod.rs            # Auth module exports
│   │       ├── handlers/
│   │       │   └── mod.rs        # HTTP handlers (sign_up, sign_in, refresh)
│   │       │                      # Size: ~200 lines
│   │       │                      # Responsibilities:
│   │       │                      #   - Validate input using domain value objects
│   │       │                      #   - Call services
│   │       │                      #   - Return responses
│   │       ├── services/
│   │       │   ├── mod.rs        # Services exports
│   │       │   ├── jwt.rs        # JWT token generation/validation
│   │       │   │                  # Size: ~120 lines
│   │       │   │                  # Tests: Token gen, validation, expiry
│   │       │   └── password.rs    # Password hashing (bcrypt)
│   │       │                       # Size: ~50 lines
│   │       │                       # Tests: Hash, verify, wrong password
│   │       └── models/
│   │           └── mod.rs        # Request/Response DTOs
│   │                              # SignUp, SignIn, RefreshToken requests
│   │                              # AuthResponse with tokens
│   │
│   ├── infrastructure/           # Technical infrastructure
│   │   ├── mod.rs                # Infrastructure module exports
│   │   ├── database/
│   │   │   └── mod.rs            # Database connection pool setup
│   │   │                          # Creates PgPool with SQLx
│   │   │                          # Size: ~40 lines
│   │   │                          # Tests: Config initialization
│   │   └── config/
│   │       └── mod.rs            # Configuration from environment
│   │                              # SERVER_HOST, PORT, DATABASE_URL, JWT_SECRET
│   │                              # Size: ~35 lines
│   │                              # Tests: Server address formatting
│   │
│   └── shared/                   # Cross-cutting concerns
│       ├── mod.rs                # Shared module exports
│       ├── errors/
│       │   └── mod.rs            # AppError type and HTTP response mapping
│       │                          # Size: ~90 lines
│       │                          # Includes: Conversion from sqlx/jwt errors
│       │                          # Tests: Error display and conversion
│       └── kernel/
│           └── mod.rs            # Core types and traits
│                                  # - Result type (AppResult)
│                                  # - ResultExt trait (for context)
│                                  # - Validate and HasId traits
│                                  # Size: ~50 lines
│                                  # Tests: Result extension trait
│
├── migrations/                   # Database migrations (SQL)
│   ├── 001_create_users_table.sql
│   │   - Creates users table with email index
│   │   - Fields: id, email, password_hash, created_at, updated_at
│   │
│   └── 002_create_refresh_tokens_table.sql
│       - Creates refresh_tokens table with user foreign key
│       - Fields: id, user_id, token_hash, expires_at, created_at
│       - Indexes: user_id, expires_at
│
├── Cargo.toml                    # Rust package manifest
│   - Dependencies listed with versions
│   - Features: tokio full, sqlx postgres, etc.
│
├── .env.example                  # Environment variables template
│   - Copy to .env and fill in values
│   - Contains: SERVER_*, DATABASE_URL, JWT_*
│
├── README.md                     # Main documentation
│   - Architecture overview
│   - Features list
│   - API endpoint documentation with examples
│   - Configuration guide
│   - Security considerations
│   - Development instructions
│
├── QUICKSTART.md                 # Setup and running guide
│   - Prerequisites
│   - Database setup (Docker or local)
│   - Running migrations
│   - Starting the server
│   - Testing endpoints with curl
│   - Troubleshooting
│
├── ARCHITECTURE.md               # Architecture decisions
│   - Directory structure explained
│   - User vs Auth coupling analysis
│   - Three tiers of sharing (infrastructure, domain, feature-specific)
│   - Guidelines for adding features
│   - Dependency direction rules
│
├── SUMMARY.md                    # Project overview
│   - What was built
│   - Architecture highlights
│   - Testing coverage
│   - Key design decisions
│   - Feature list with examples
│
├── IMPLEMENTATION_TRACKING.md    # Progress tracking
│   - What's completed
│   - What's in progress
│   - What's TODO
│   - Build status
│   - Phase breakdown
│
└── COUPLING_ANALYSIS.md          # Deep dive on module coupling
    - Why User and Auth are separate
    - Trade-offs analysis
    - Real-world examples
    - Decision criteria used
```

## Key Files by Purpose

### Core Domain
- `src/domain/entities/user.rs` - User entity (business logic)
- `src/domain/value_objects/email.rs` - Email validation
- `src/domain/value_objects/password.rs` - Password validation

### Authentication
- `src/features/auth/handlers/mod.rs` - API endpoints
- `src/features/auth/services/jwt.rs` - Token generation
- `src/features/auth/services/password.rs` - Password hashing
- `src/features/auth/models/mod.rs` - Request/response DTOs

### Infrastructure
- `src/infrastructure/database/mod.rs` - PostgreSQL setup
- `src/infrastructure/config/mod.rs` - Configuration loading
- `src/main.rs` - Server initialization

### Shared
- `src/shared/errors/mod.rs` - Error handling
- `src/shared/kernel/mod.rs` - Core types

### Database
- `migrations/001_*.sql` - Users table
- `migrations/002_*.sql` - Refresh tokens table

## Code Statistics

| Module | Lines | Purpose | Tests |
|--------|-------|---------|-------|
| domain/entities/user.rs | ~80 | User entity | 3 |
| domain/value_objects/email.rs | ~100 | Email validation | 4 |
| domain/value_objects/password.rs | ~90 | Password validation | 5 |
| features/auth/handlers/mod.rs | ~200 | API endpoints | - |
| features/auth/services/jwt.rs | ~120 | JWT tokens | 3 |
| features/auth/services/password.rs | ~50 | Password hashing | 3 |
| features/auth/models/mod.rs | ~35 | DTOs | - |
| infrastructure/database/mod.rs | ~40 | DB setup | 1 |
| infrastructure/config/mod.rs | ~35 | Config | 1 |
| shared/errors/mod.rs | ~90 | Error handling | 2 |
| shared/kernel/mod.rs | ~50 | Core types | 1 |
| **TOTAL** | **~985** | **Core implementation** | **24** |

## Module Dependencies

```
main.rs
└── lib.rs
    ├── domain/
    │   ├── entities/user.rs
    │   └── value_objects/
    │       ├── email.rs
    │       └── password.rs
    ├── features/auth/
    │   ├── handlers/
    │   │   ├── domain/
    │   │   ├── features/auth/services/
    │   │   └── infrastructure/
    │   ├── services/
    │   │   ├── jwt.rs
    │   │   │   └── shared/
    │   │   └── password.rs
    │   │       └── shared/
    │   └── models/
    ├── infrastructure/
    │   ├── config/
    │   └── database/
    └── shared/
        ├── errors/
        └── kernel/
```

**Key Rule**: No module imports from other features.

## How to Navigate

### To understand User entity
→ Start at `src/domain/entities/user.rs`

### To understand authentication flow
→ Start at `src/features/auth/handlers/mod.rs`
→ Then read `src/features/auth/services/jwt.rs`
→ Then read `src/features/auth/services/password.rs`

### To add a new endpoint
→ Read `src/features/auth/handlers/mod.rs` for pattern
→ Read `src/features/auth/services/` for service pattern
→ Create similar structure in new feature

### To understand error handling
→ Read `src/shared/errors/mod.rs`
→ See how it's used in handlers

### To understand value objects
→ Read `src/domain/value_objects/email.rs`
→ See how it's used in handlers and entities

## Testing

All modules have embedded tests. Run with:
```bash
cargo test              # Run all tests
cargo test --lib       # Run only library tests
cargo test test_name    # Run specific test
```

Test locations:
- Domain: `#[cfg(test)]` in each file
- Auth services: `#[cfg(test)]` in each service
- Error handling: `#[cfg(test)]` in errors
- Infrastructure: `#[cfg(test)]` in config/database

## Development Tips

1. **Start small**: Each file is ~50-200 lines, manageable to understand
2. **Tests are documentation**: Look at tests to understand how to use code
3. **Domain first**: Read domain layer before features
4. **Follow patterns**: Each layer has clear patterns (handlers, services, models)
5. **Error handling**: Always use `AppResult<T>` for fallible operations

## Next: Adding Features

See [Adding New Features](./ARCHITECTURE.md#adding-new-features) in ARCHITECTURE.md for how to extend this system.
