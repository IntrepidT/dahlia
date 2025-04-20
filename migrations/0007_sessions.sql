-- Create enum types needed for session status
CREATE TYPE session_status_enum AS ENUM ('active', 'inactive', 'expired');

-- Create sessions table
CREATE TABLE websocket_sessions (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_active TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    owner_id UUID,
    status session_status_enum NOT NULL DEFAULT 'active',
    max_users INTEGER DEFAULT 0,
    current_users INTEGER DEFAULT 0,
    is_private BOOLEAN DEFAULT false,
    password_required BOOLEAN DEFAULT false,
    password_hash VARCHAR(255),
    metadata JSONB
);

-- Create index for quick lookups by status
CREATE INDEX sessions_status_idx ON websocket_sessions(status);

-- Create index for sorting by activity
CREATE INDEX sessions_last_active_idx ON websocket_sessions(last_active);
