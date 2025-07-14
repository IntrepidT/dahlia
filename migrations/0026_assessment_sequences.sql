-- Add column to assessments table
ALTER TABLE assessments ADD COLUMN test_sequence JSONB;

-- Create enum types for sequence behavior
CREATE TYPE sequence_behavior_enum AS ENUM (
    'attainment', 
    'node', 
    'optional', 
    'diagnostic', 
    'remediation', 
    'branching'
);
