--Create new ENUM type to replace existing Intervention() implementation
CREATE TYPE intervention_enum AS ENUM ('Literacy', 'Math');

--Modify students table and replace existing Intervention() implementation
ALTER TABLE students
  ALTER COLUMN intervention DROP NOT NULL;

ALTER TABLE students
  ALTER COLUMN intervention DROP DEFAULT;

ALTER TABLE students
  ALTER COLUMN intervention TYPE intervention_enum
    USING CASE
      WHEN intervention = true THEN 'Literacy'::intervention_enum
      WHEN intervention = false THEN NULL
    END;

ALTER TABLE students
  ALTER COLUMN intervention SET DEFAULT NULL;
