--add the developer settings category and toggle for student protections
CREATE TABLE IF NOT EXISTS global_settings (
  id SERIAL PRIMARY KEY,
  key VARCHAR(255) UNIQUE NOT NULL,
  value JSONB NOT NULL,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_by INTEGER REFERENCES users(id)
);

--INSERT the new setting for student protections
INSERT INTO global_settings (key, value, updated_by)
VALUES ('student_protections', 'false', NULL)
ON CONFLICT (key) DO NOTHING;

-- create index for efficient global settings lookup
CREATE INDEX IF NOT EXISTS idx_global_settings_key ON global_settings (key);
