-- Add the user_role_enum type
CREATE TYPE user_role_enum AS ENUM (
    'admin',
    'teacher',
    'guest',
    'superadmin',
    'user'
);

-- Fix the assignment to the user role column
ALTER TABLE users ALTER COLUMN role DROP DEFAULT;

ALTER TABLE users
ALTER COLUMN role TYPE user_role_enum USING role::text::user_role_enum;

ALTER TABLE users ALTER COLUMN role SET DEFAULT 'guest';
