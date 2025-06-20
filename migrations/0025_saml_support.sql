-- Table for SAML configuration per institution
CREATE TABLE saml_configs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  institution_name VARCHAR(255) NOT NULL UNIQUE,
  entity_id VARCHAR(255) NOT NULL UNIQUE,
  sso_url TEXT NOT NULL,
  slo_url TEXT,
  x509_cert TEXT NOT NULL,
  metadata_url TEXT,
  active BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

  -- Configuration options
  attribute_mapping JSONB DEFAULT '{}',
  role_mapping JSONB DEFAULT '{}',
  auto_provision BOOLEAN NOT NULL DEFAULT true,
  require_encrypted_assertions BOOLEAN NOT NULL DEFAULT false
);

-- Table to map users to their SAML identities
CREATE TABLE saml_user_mappings (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  institution_id VARCHAR(255) NOT NULL,
  saml_name_id VARCHAR(255) NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  last_login TIMESTAMP WITH TIME ZONE,

  UNIQUE (user_id, institution_id),
  UNIQUE (institution_id, saml_name_id)
);

-- Need to add auth_provider to sessions table to track how user logged in
ALTER TABLE sessions
ADD COLUMN auth_provider VARCHAR(50) DEFAULT 'local',
ADD COLUMN institution_id VARCHAR(255);

-- Add auth_provider tracking to users table
ALTER TABLE users
ADD COLUMN last_auth_provider VARCHAR(50) DEFAULT 'local',
ADD COLUMN last_institution_id VARCHAR(255);

-- Indexes for performance
CREATE INDEX idx_saml_configs_institution ON saml_configs(institution_name);
CREATE INDEX idx_saml_configs_active ON saml_configs(active);
CREATE INDEX idx_saml_user_mappings_user_id ON saml_user_mappings(user_id);
CREATE INDEX idx_saml_user_mappings_institution ON saml_user_mappings(institution_id);
CREATE INDEX idx_saml_user_mappings_name_id ON saml_user_mappings(saml_name_id);
CREATE INDEX idx_sessions_auth_provider ON sessions(auth_provider);

-- Function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_saml_configs_updated_at()
RETURNS TRIGGER AS $$
BEGIN 
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

-- Trigger to automatically update the updated_at column
CREATE TRIGGER update_saml_configs_updated_at
  BEFORE UPDATE ON saml_configs
  FOR EACH ROW
  EXECUTE FUNCTION update_saml_configs_updated_at();
