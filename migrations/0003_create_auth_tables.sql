-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGSERIAL NOT NULL,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index on sessions token for fast lookups
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);

-- Create index on sessions expiration for cleanup
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

--ADD foreign key constraint after both tables EXISTS
ALTER TABLE sessions
ADD CONSTRAINT sessions_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
