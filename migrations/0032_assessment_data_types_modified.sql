--fix column changes for assessment_data_types
ALTER TABLE assessments 
  ALTER COLUMN subject DROP DEFAULT,
  ALTER COLUMN subject DROP NOT NULL;
