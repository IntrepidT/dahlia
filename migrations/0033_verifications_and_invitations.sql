--BEGIN;

--set transaction isolation level
--SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;

--create a savepoint 
--SAVEPOINT before_migration;

--Migration
ALTER TABLE users ADD COLUMN IF NOT EXISTS school_id VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS invitation_code VARCHAR(255);

--Update email_verification and phone_verfied columns to have proper defaults if not set
DO $$
BEGIN
    -- Check if email_verified has a default value
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' 
        AND column_name = 'email_verified' 
        AND column_default IS NOT NULL
    ) THEN
        ALTER TABLE users ALTER COLUMN email_verified SET DEFAULT FALSE;
        --RAISE NOTICE 'Set default FALSE for email_verified column';
    ELSE
        --RAISE NOTICE 'email_verified column already has a default value';
    END IF;
END $$;
DO $$
BEGIN
    -- Check if phone_verified has a default value
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' 
        AND column_name = 'phone_verified' 
        AND column_default IS NOT NULL
    ) THEN
        ALTER TABLE users ALTER COLUMN phone_verified SET DEFAULT FALSE;
        --RAISE NOTICE 'Set default FALSE for phone_verified column';
    ELSE
        --RAISE NOTICE 'phone_verified column already has a default value';
    END IF;
END $$;

--Create invitations and verifications tables
CREATE TABLE IF NOT EXISTS invitations (
  id SERIAL PRIMARY KEY,
  code VARCHAR(255) NOT NULL UNIQUE,
  school_name VARCHAR(255) NOT NULL DEFAULT 'Default School',
  invidted_by_user_id BIGINT REFERENCES users(id),
  role VARCHAR(50) NOT NULL DEFAULT 'user',
  max_uses INTEGER DEFAULT 1,
  current_uses INTEGER DEFAULT 0,
  expires_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS verification_codes (
  id SERIAL PRIMARY KEY,
  user_id BIGINT REFERENCES users(id),
  code VARCHAR(6) NOT NULL,
  type VARCHAR(10) NOT NULL, 
  expires_at TIMESTAMP NOT NULL,
  used_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW()
);

--MIGRATE existing users
DO $$
DECLARE
    user_count INTEGER;
    updated_count INTEGER;
BEGIN
    -- Count existing users
    SELECT COUNT(*) INTO user_count FROM users;
    --RAISE NOTICE 'Found % existing users', user_count;
    
    -- Update existing users based on their account_status
    -- Users with 'active' status should be considered email verified
    UPDATE users 
    SET email_verified = CASE 
        WHEN account_status = 'active' THEN TRUE 
        ELSE COALESCE(email_verified, FALSE)
    END,
    phone_verified = COALESCE(phone_verified, FALSE)
    WHERE email_verified IS NULL OR phone_verified IS NULL;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    --RAISE NOTICE 'Updated verification status for % users', updated_count;
    
    -- Set default school for existing users who don't have one
    UPDATE users 
    SET school_id = 'Legacy School'
    WHERE school_id IS NULL;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    --RAISE NOTICE 'Set default school for % users', updated_count;
    
    -- Normalize existing phone numbers to E.164 format if they exist
    UPDATE users 
    SET phone_number = CASE 
        -- Handle 10-digit numbers (add +1)
        WHEN phone_number ~ '^\d{10}$' THEN '+1' || phone_number
        -- Handle 11-digit numbers starting with 1 (add +)
        WHEN phone_number ~ '^1\d{10}$' THEN '+' || phone_number
        -- Handle numbers that already have + (keep as is)
        WHEN phone_number ~ '^\+\d{11,15}$' THEN phone_number
        -- Remove non-digits and try again
        WHEN LENGTH(regexp_replace(phone_number, '\D', '', 'g')) = 10 
        THEN '+1' || regexp_replace(phone_number, '\D', '', 'g')
        WHEN LENGTH(regexp_replace(phone_number, '\D', '', 'g')) = 11 
             AND regexp_replace(phone_number, '\D', '', 'g') ~ '^1\d{10}$'
        THEN '+' || regexp_replace(phone_number, '\D', '', 'g')
        -- Invalid numbers set to NULL
        ELSE NULL
    END
    WHERE phone_number IS NOT NULL AND phone_number != '';
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    --RAISE NOTICE 'Normalized phone numbers for % users', updated_count;
    
END $$;

-- Verify migration
--DO $$
--DECLARE
--    total_users INTEGER;
--    email_verified_count INTEGER;
--    phone_verified_count INTEGER;
--    users_with_school INTEGER;
--    users_with_phone INTEGER;
--    invitations_count INTEGER;
--    verification_codes_count INTEGER;
--BEGIN
--    -- Check user counts
--    SELECT COUNT(*) INTO total_users FROM users;
--    SELECT COUNT(*) INTO email_verified_count FROM users WHERE email_verified = TRUE;
--    SELECT COUNT(*) INTO phone_verified_count FROM users WHERE phone_verified = TRUE;
--    SELECT COUNT(*) INTO users_with_school FROM users WHERE school_id IS NOT NULL;
--    SELECT COUNT(*) INTO users_with_phone FROM users WHERE phone_number IS NOT NULL;
--    
--    -- Check new tables
--    SELECT COUNT(*) INTO invitations_count FROM invitations;
--    SELECT COUNT(*) INTO verification_codes_count FROM verification_codes;
--    
--    -- Validation checks
--    IF total_users = 0 THEN
--        RAISE EXCEPTION 'VALIDATION FAILED: No users found after migration!';
--    END IF;
--    
--    IF users_with_school = 0 THEN
--        RAISE EXCEPTION 'VALIDATION FAILED: No users have school_id set!';
--    END IF;
--    
--    -- Report results
--    RAISE NOTICE '=== MIGRATION VALIDATION ===';
--    RAISE NOTICE 'Total users: %', total_users;
--    RAISE NOTICE 'Email verified: % (% %%)', email_verified_count, 
--        ROUND(100.0 * email_verified_count / NULLIF(total_users, 0), 1);
--    RAISE NOTICE 'Phone verified: % (% %%)', phone_verified_count, 
--        ROUND(100.0 * phone_verified_count / NULLIF(total_users, 0), 1);
--    RAISE NOTICE 'Users with school: % (% %%)', users_with_school, 
--        ROUND(100.0 * users_with_school / NULLIF(total_users, 0), 1);
--    RAISE NOTICE 'Users with phone: % (% %%)', users_with_phone, 
--        ROUND(100.0 * users_with_phone / NULLIF(total_users, 0), 1);
--    RAISE NOTICE 'Invitations created: %', invitations_count;
--    RAISE NOTICE 'Verification codes: %', verification_codes_count;
--    RAISE NOTICE 'VALIDATION PASSED ✓';
--END $$;

--commit if everything is fine
--COMMIT;
