-- Migration 0006: Add token rotation and expiration fields

-- Add new columns to refresh_tokens table
ALTER TABLE refresh_tokens
ADD COLUMN expires_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (NOW() + INTERVAL '7 days'),
ADD COLUMN is_used BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN used_at TIMESTAMP WITH TIME ZONE;

-- Add index for checking expired tokens (for cleanup jobs)
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

-- Add index for checking if token was used (for reuse detection)
CREATE INDEX idx_refresh_tokens_is_used ON refresh_tokens(is_used);

-- Update existing tokens to have expiration (7 days from now)
UPDATE refresh_tokens
SET expires_at = NOW() + INTERVAL '7 days'
WHERE expires_at IS NULL;
