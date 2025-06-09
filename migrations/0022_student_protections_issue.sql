UPDATE global_settings SET updated_by = 1 WHERE updated_by IS NULL;

--make the clumn NOT NULL with a default
ALTER TABLE global_settings ALTER COLUMN updated_by SET NOT NULL;
ALTER TABLE global_settings ALTER COLUMN updated_by SET DEFAULT 0;
