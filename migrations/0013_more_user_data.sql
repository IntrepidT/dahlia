CREATE TYPE account_status_enum AS ENUM (
  'pending',
  'active',
  'suspended',
  'deleted'
);

ALTER TABLE users
  ADD COLUMN IF NOT EXISTS password_salt VARCHAR(255),
  
  -- Account status and security
  ADD COLUMN IF NOT EXISTS account_status account_status_enum NOT NULL DEFAULT 'pending',
  ADD COLUMN IF NOT EXISTS email_verified BOOLEAN NOT NULL DEFAULT FALSE,
  ADD COLUMN IF NOT EXISTS phone_number VARCHAR(20),
  ADD COLUMN IF NOT EXISTS phone_verified BOOLEAN NOT NULL DEFAULT FALSE,

  -- Profile information 
  ADD COLUMN IF NOT EXISTS display_name VARCHAR(255),
  ADD COLUMN IF NOT EXISTS first_name VARCHAR(255),
  ADD COLUMN IF NOT EXISTS last_name VARCHAR(255),

  -- Password reset and recovery
  ADD COLUMN IF NOT EXISTS password_reset_token VARCHAR(255),
  ADD COLUMN IF NOT EXISTS password_reset_expires TIMESTAMP WITH TIME ZONE;
