-- Create refresh_tokens table for token management
CREATE TABLE refresh_tokens (
    id CHAR(26) PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id CHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on user_id for faster lookups
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);

-- Create index on expires_at to find valid tokens
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
