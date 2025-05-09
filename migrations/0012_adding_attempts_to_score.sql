ALTER TABLE scores 
ADD COLUMN IF NOT EXISTS attempt INT NOT NULL DEFAULT 1;

-- Add a unique constraint to ensure we don't have duplicate entries
ALTER TABLE scores 
ADD CONSTRAINT unique_score_attempt UNIQUE (student_id, test_id, test_variant, attempt);

-- Create a function to automatically determine the next attempt number
CREATE OR REPLACE FUNCTION next_attempt_number(p_student_id INT, p_test_id UUID, p_test_variant INT) 
RETURNS INT AS $$
DECLARE
    next_attempt INT;
BEGIN
    SELECT COALESCE(MAX(attempt), 0) + 1 INTO next_attempt
    FROM scores
    WHERE student_id = p_student_id 
      AND test_id = p_test_id 
      AND test_variant = p_test_variant;
    
    RETURN next_attempt;
END;
$$ LANGUAGE plpgsql;
