-- Remove the previous constraint defined in 0001
ALTER TABLE question_table
DROP CONSTRAINT IF EXISTS question_table_check;

-- Add the new constraint for weighted multiple choice questions to escape
ALTER TABLE question_table
ADD CONSTRAINT question_table_check
CHECK (
  question_type = 'Weighted Multiple Choice'::questiontype_enum
  OR correct_answer = ANY(options)
);
