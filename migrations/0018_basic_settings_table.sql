-- Create a table to store user_settings
ALTER TABLE users ADD COLUMN IF NOT EXISTS settings JSONB DEFAULT '{"ui": {"dark_mode": false, "pinned_sidebar": false}}';


--Create index for fast query
CREATE INDEX IF NOT EXISTS idx_users_settings ON users USING GIN ((settings ->'ui'));
