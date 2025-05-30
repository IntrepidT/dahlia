-- Alter the assessments table to use the grade_enum (mistake made when defining assessment table)
ALTER TABLE assessments
ALTER COLUMN grade TYPE grade_enum
USING grade::grade_enum;
