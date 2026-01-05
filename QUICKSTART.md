# Quick Start Guide

This guide will help you get the Prontua Backend running locally.

## Prerequisites

- **Rust**: 1.70 or later (install from https://rustup.rs/)
- **PostgreSQL**: 12 or later
- **SQLx CLI**: `cargo install sqlx-cli --no-default-features --features postgres`

## Step 1: Setup Database

### Option A: Using Docker (Easiest)

```bash
docker run -d \
  --name prontua-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=prontua \
  -p 5432:5432 \
  postgres:15-alpine
```

### Option B: Using Local PostgreSQL

```bash
# On macOS with Homebrew
brew install postgresql@15
brew services start postgresql@15

# Or create database manually
createdb prontua
```

## Step 2: Configure Environment

```bash
# Copy the example environment file
cp .env.example .env

# Verify DATABASE_URL in .env points to your PostgreSQL
# Default: postgres://postgres:postgres@localhost:5432/prontua
```

## Step 3: Run Migrations

```bash
# Run all migrations
sqlx migrate run

# Or if using Docker, check migrations are in place
ls migrations/
```

## Step 4: Start the Server

```bash
cargo run
```

You should see:
```
Starting Prontua Backend
Server will listen on 127.0.0.1:3000
Database connection established
Server running on 127.0.0.1:3000
```

## Step 5: Test the API

### Sign Up
```bash
curl -X POST http://localhost:3000/api/auth/sign-up \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "SecurePass123"
  }'
```

Response:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLC...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLC...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### Sign In
```bash
curl -X POST http://localhost:3000/api/auth/sign-in \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "SecurePass123"
  }'
```

### Refresh Token
```bash
curl -X POST http://localhost:3000/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJ0eXAiOiJKV1QiLC..."
  }'
```

## Testing

### Run All Tests
```bash
cargo test
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Specific Test
```bash
cargo test test_valid_email
```

## Development Commands

```bash
# Check for compilation errors without building
cargo check

# Format code according to Rust standards
cargo fmt

# Lint code for common mistakes
cargo clippy

# Build release binary
cargo build --release

# Clean build artifacts
cargo clean
```

## Troubleshooting

### Database Connection Error
```
Error: Database connection failed
```

**Solution**: Check that PostgreSQL is running and DATABASE_URL in .env is correct.

```bash
# Test connection
psql postgres://postgres:postgres@localhost:5432/prontua
```

### Migration Errors
```
Error: migration error
```

**Solution**: Ensure migrations directory exists and database is clean.

```bash
# Reset database (⚠️ WARNING: Deletes all data)
dropdb prontua
createdb prontua
sqlx migrate run
```

### Port Already in Use
```
Error: Address already in use
```

**Solution**: Change SERVER_PORT in .env or kill the process on port 3000.

```bash
# Kill process on port 3000 (macOS/Linux)
lsof -ti :3000 | xargs kill -9
```

## Next Steps

1. Read [ARCHITECTURE.md](./ARCHITECTURE.md) for architecture details
2. Read [README.md](./README.md) for complete documentation
3. Check [IMPLEMENTATION_TRACKING.md](./IMPLEMENTATION_TRACKING.md) for what's been done
4. Explore the code structure in `src/`

## Getting Help

- Check the code comments (every module is documented)
- Read the tests for usage examples
- Review the error messages (they're designed to be helpful)
- Open an issue on GitHub

## Common Passwords to Test With

```
✅ Valid (meet all requirements):
- SecurePass123
- MyP@ssw0rd
- TestPass999
- Demo2024Secure

❌ Invalid (various failures):
- password         # Too short, no uppercase/digit
- PASSWORD123     # No lowercase
- Shortpas1       # 8 chars but meets all rules (actually valid)
- Short1          # Too short
```

## Production Deployment

See [README.md](./README.md) section "Next Steps" for deployment options.
