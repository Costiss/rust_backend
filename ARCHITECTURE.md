# Architecture: Domain-Centric Modular Monolith

This backend follows a **vertical slice / domain-centric architecture** pattern inspired by Milan Jovanović's approach to modular monoliths. Each domain feature is organized vertically, containing its own models, handlers, and services.

## Directory Structure

```
src/
├── domain/               # Shared domain layer (entities, value objects, domain services)
│   ├── entities/         # Core business entities (User, etc.)
│   ├── value_objects/    # Immutable value types (Email, Password, etc.)
│   └── services/         # Domain-level business logic
├── features/             # Feature slices (vertical slices)
│   └── auth/             # Authentication feature
│       ├── handlers/      # HTTP handlers/endpoints
│       ├── services/      # Feature-specific services
│       └── models/        # Request/response DTOs
├── infrastructure/       # Technical infrastructure concerns
│   ├── database/         # Database connection, migrations
│   └── config/           # Configuration management
└── shared/               # Cross-cutting concerns
    ├── kernel/           # Core types (Result type, common traits)
    └── errors/           # Global error handling
```

## User vs Auth Module Coupling: Trade-offs Analysis

### Current Design: Decoupled Pattern

**Decision**: User and Auth are **separate concerns** but share the same domain model (User entity).

#### Rationale:

1. **User Entity Lives in Domain**
   - The `User` entity (id, email, password hash) is a shared domain model
   - Both Auth and User features can reference the same entity
   - Business rules about users are in the User entity itself

2. **Auth Feature is Separate**
   - Auth feature handles: JWT generation, refresh token logic, sign-in/sign-up
   - Auth does NOT manage user profiles, user updates, or user-specific data
   - This keeps auth focused on what it does best: authentication

3. **Why This is Better**

   | Aspect | Coupled (Single Module) | Decoupled (Separate) |
   |--------|------------------------|-----------------------|
   | **Change Frequency** | Auth changes ≠ user profile changes | Each changes independently |
   | **Reusability** | Only auth can use user operations | Future modules can reference User |
   | **Testing** | Test User and Auth together | Test features independently |
   | **Scalability** | Single module grows large | Features stay focused |
   | **Team Work** | One team owns everything | Teams can own features |
   | **Dependency Direction** | Unclear | Clear: Auth → User (domain) |

#### Trade-offs:

**Pro-Coupling Arguments** (and why we reject them):
- "Less code duplication" → User entity is shared anyway
- "Simpler to implement" → Only true initially; grows complex with feature requests
- "Fewer service calls" → Database is single source of truth anyway
- "Less 'boilerplate'" → Boilerplate is cheap; coupling is expensive

**Our Approach: Coupling Where It Matters**
- User and Auth are **tightly coupled on the domain model** (they share the User entity)
- User and Auth are **loosely coupled on behavior** (separate features with clear boundaries)
- Auth can't accidentally modify user profiles because it doesn't own that feature
- Future user-profile feature reuses the User entity without touching Auth

### Shared Code Strategy

Following the article's **three tiers of sharing**:

#### Tier 1: Technical Infrastructure (Share Freely)
- Database connection pooling
- Logging adapters
- JWT middleware
- Error responses
- **Location**: `infrastructure/` and `shared/kernel/`

#### Tier 2: Domain Concepts (Share via Entities)
- User entity with business rules
- Email and Password value objects
- Domain-level validation
- **Location**: `domain/entities/` and `domain/value_objects/`

#### Tier 3: Feature-Specific Logic (Keep Local)
- Auth token generation specifics
- User profile formatting (if created later)
- Feature-specific validation
- **Location**: `features/auth/services/` or `features/user/shared/`

### Database Schema Perspective

The User table will have:
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL
);
```

**Note**: Auth feature manages refresh tokens. User feature would manage user profile. Single table, logical separation.

## Guidelines

### When Adding New Features

1. Create a new feature folder in `src/features/`
2. If it uses shared domain entities, import from `domain/`
3. If you need to share code between features:
   - Check if it's infrastructure → move to `infrastructure/`
   - Check if it's business logic → move to `domain/services/`
   - Check if it's specific to 2-3 related slices → create `features/[name]/shared/`
   - Otherwise, keep it local

### Dependency Direction

```
features/auth → domain → shared/kernel
              ↓
         infrastructure
```

Features depend on domain and infrastructure, never on other features.

## References

- [Vertical Slice Architecture: Where Does the Shared Logic Live?](https://www.milanjovanovic.tech/blog/vertical-slice-architecture-where-does-the-shared-logic-live)
- Milan Jovanović's Modular Monolith Architecture course

