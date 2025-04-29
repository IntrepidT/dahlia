-- Create enum types first
CREATE TYPE subject_enum AS ENUM (
    'Reading',
    'Math',
    'Literacy',
    'Phonics',
    'History',
    'Science',
    'Social Studies',
    'Other'
);

-- Create the assessments table
CREATE TABLE assessments (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    frequency INTEGER,
    grade VARCHAR(50),
    version INTEGER NOT NULL DEFAULT 1,
    tests UUID[] NOT NULL DEFAULT '{}',
    composite_score INTEGER,
    risk_benchmarks JSONB,
    national_benchmarks JSONB,
    subject subject_enum NOT NULL DEFAULT 'Other',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

