# User vs Auth Module Coupling: Deep Dive

This document explains the architectural trade-offs for coupling/decoupling User and Auth modules in detail.

## The Question

Should User and Auth be:
1. **Tightly Coupled**: One module handling both user management and authentication?
2. **Loosely Coupled**: Separate modules sharing a domain model?
3. **Completely Decoupled**: Separate modules with separate user representations?

## Our Decision: Loosely Coupled (Option 2)

```
User Entity (Domain) ← Shared by both
      ↓
    ┌─────────────────┬─────────────────┐
    ↓                 ↓
Auth Feature      User Feature
(sign-up,         (profile,
 sign-in,          settings,
 refresh)          updates)
```

## Why Not Option 1: Tightly Coupled

### Single "UserAuth" Module
```rust
// ❌ Not what we did
pub struct UserAuth {
    user_id: Uuid,
    email: Email,
    password_hash: String,
    // profile fields
    first_name: String,
    last_name: String,
    avatar_url: Option<String>,
    // settings
    email_notifications: bool,
    two_factor_enabled: bool,
}
```

### Problems

**1. Different Change Frequencies**
- Auth changes: New token algorithm, refresh strategy, session management
- User profile changes: New user fields, settings, preferences
- Single module gets touched too often for unrelated reasons

**2. Code Reuse Overkill**
```rust
// Auth endpoints need only:
- sign_up(email, password)
- sign_in(email, password)
- refresh_token()

// Profile endpoints need only:
- get_profile(user_id)
- update_profile(user_id, fields)
- update_settings(user_id, settings)

// Why would these share a module?
```

**3. Growing Complexity**
```rust
impl UserAuth {
    // Auth methods
    fn verify_password(&self, plain: &str) -> bool { ... }
    fn generate_jwt(&self) -> String { ... }
    
    // Profile methods
    fn update_profile(&mut self, profile: UpdateRequest) { ... }
    fn get_settings(&self) -> Settings { ... }
    
    // Unclear responsibilities as it grows
}
```

**4. Testing Nightmare**
```rust
#[test]
fn test_sign_up() {
    let user = UserAuth::new(email, password);
    // But we also need to set up:
    // - first_name, last_name (profile fields)
    // - email_notifications (settings)
    // All for a test that only cares about password hashing
}
```

**5. Team Scalability Issue**
- Two teams need to coordinate changes to one module
- Merge conflicts when updating different fields
- Authentication team can't deploy without profile team approval

## Why Not Option 3: Completely Decoupled

### Separate User Tables/Services
```rust
// ❌ Also problematic
// In Auth
pub struct AuthUser {
    id: Uuid,
    email: Email,
    password_hash: String,
}

// In User
pub struct UserProfile {
    id: Uuid,
    email: Email,  // Duplicate!
    first_name: String,
    last_name: String,
}
```

### Problems

**1. Massive Duplication**
- Email exists in both places
- User ID exists in both places
- Keep them in sync? Nightmare

**2. Data Integrity Issues**
- Change email in Auth, but not in User profile?
- Delete user in Auth but it still exists in User?
- Which is the source of truth?

**3. Complex Queries**
```sql
-- To get user profile with auth info
SELECT u.*, a.password_hash 
FROM user_profiles u
JOIN auth_users a ON u.email = a.email  -- Join on email!
WHERE u.id = $1;
```

**4. No Shared Behavior**
- Can't define "valid email" once
- Can't share validation logic
- Violates DRY principle

## Why Our Approach Works

### Shared Domain Model, Separate Features

```rust
// Domain (shared)
pub struct User {
    id: Uuid,
    email: Email,
    password_hash: String,  // Still secret, only accessed by Auth
    created_at: DateTime,
}

// Feature: Auth
// Uses: User entity, password hashing
// Responsible for: JWT generation, refresh tokens
pub async fn sign_up(email: Email, password: Password) -> Result<User> { ... }
pub async fn sign_in(email: Email, password: Password) -> Result<User> { ... }

// Feature: User (created later)
// Uses: User entity, but not for auth
// Responsible for: profile updates, user management
pub async fn get_profile(user_id: Uuid) -> Result<UserProfile> { ... }
pub async fn update_profile(user_id: Uuid, updates: UpdateRequest) { ... }
```

### Benefits

**1. Independent Changes**
```rust
// Auth can change without touching User
pub async fn sign_up(email: Email, password: Password) -> Result<(User, String)> {
    // Returns JWT directly instead of separate step
    // User feature unaffected
}

// User can change without touching Auth
// New fields: avatar_url, preferences
// Auth still works perfectly
```

**2. Focused Responsibilities**
- Auth: "I generate tokens and manage sessions"
- User: "I manage user profiles and settings"
- User entity: "I am a person in the system"

**3. Clean Separation**
```
Auth doesn't know about user profiles
User doesn't know about JWT tokens

They only know about User entity
```

**4. Easy to Test**
```rust
#[test]
fn test_sign_up() {
    let user = User::new(email, password_hash);
    assert_eq!(user.email, email);
    // Done! No need to set up profile fields
}

#[test]
fn test_update_profile() {
    let user = create_test_user();
    let profile = UserProfile::from(&user);
    // Update profile without touching auth fields
}
```

**5. Team Autonomy**
- Auth team owns: JWT logic, token management
- User team owns: Profile, settings, preferences
- Shared: Domain model in src/domain/

## Coupling Analysis

### Direction of Dependencies

```
features/auth → domain/User ✅ CORRECT
features/user → domain/User ✅ CORRECT
features/auth → features/user ❌ NEVER
features/user → features/auth ❌ NEVER
```

**Auth never imports from User feature**
**User never imports from Auth feature**

They only share through the domain layer.

### Why This Matters

```rust
// ❌ BAD - Auth depends on User feature
use crate::features::user::UserProfileService;

#[post("/sign-up")]
async fn sign_up(payload: SignUpRequest) -> Result<AuthResponse> {
    let user = create_user(&payload.email)?;
    UserProfileService::initialize(user.id)?;  // Coupling!
}
```

```rust
// ✅ GOOD - Auth is independent
#[post("/sign-up")]
async fn sign_up(payload: SignUpRequest) -> Result<AuthResponse> {
    let email = Email::new(&payload.email)?;
    let password = Password::new(&payload.password)?;
    
    let user = User::new(email, hash_password(password))?;
    // User feature can be added later without touching Auth
}
```

## Trade-offs Summary

| Factor | Tightly Coupled | Loosely Coupled (Ours) | Decoupled |
|--------|-----------------|------------------------|-----------|
| **Code Duplication** | Low | Medium (domain model) | High |
| **Independence** | Low | High | Complete |
| **Data Integrity** | High | High | Medium |
| **Change Frequency** | Low (no changes) | High (aligned) | Medium |
| **Team Coordination** | Required | Minimal | None |
| **Testing Ease** | Hard | Easy | Hard |
| **Feature Addition** | Hard | Easy | Hard |
| **Learning Curve** | Low | Medium | High |

## Decision Criteria We Used

1. **Is this infrastructural or domain?**
   - → Shared domain model ✓

2. **How stable is this concept?**
   - User identity: Very stable
   - Authentication mechanisms: Evolves frequently
   - → Keep separate features ✓

3. **Am I past the "Rule of Three"?**
   - Not yet at 3 duplications
   - Unlikely to ever have multiple auth systems
   - → Share at domain level ✓

## Real-World Example

### When You Add User Profiles Later

```rust
// User feature (new)
#[post("/profile")]
async fn update_profile(
    user_id: Uuid,
    payload: UpdateProfileRequest
) -> Result<Json<UserProfile>> {
    // Access to User entity only
    let user = fetch_user(user_id)?;
    
    // Updates happen here
    // Auth feature has NO IDEA
    // Auth tokens still work
}

// Auth feature (unchanged)
#[post("/sign-in")]
async fn sign_in(payload: SignInRequest) -> Result<AuthResponse> {
    // Auth still works exactly the same
    // Never heard of profiles
}
```

### When You Add 2FA

```rust
// Auth feature (updated)
pub async fn sign_in_with_2fa(
    email: Email,
    password: Password,
    code: String
) -> Result<AuthResponse> {
    // New auth flow
    // User feature doesn't care
    // Profiles still work
}

// User feature (unchanged)
// All profile operations continue working
```

## Conclusion

**Our approach (loosely coupled) balances:**
- ✅ Code reuse through shared domain models
- ✅ Independence through separate features
- ✅ Clear responsibility boundaries
- ✅ Easy testing and maintenance
- ✅ Team scalability
- ✅ Future extensibility

This is the sweet spot between "coupled so tightly everything breaks" and "decoupled so completely you duplicate everything."
