ALTER TABLE tests
ADD COLUMN school_year VARCHAR(20);

ALTER TABLE tests
ADD COLUMN benchmark_categories JSONB;

ALTER TABLE tests
ADD COLUMN test_variant INT NOT NULL DEFAULT 1;

ALTER TABLE tests
ADD COLUMN grade_level grade_enum;
