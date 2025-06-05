--Modify assessments table to support assessments + tests localization
ALTER TABLE assessments ADD COLUMN course_id INTEGER;
ALTER TABLE assessments ADD CONSTRAINT fk_assessment_course
  FOREIGN KEY (course_id) REFERENCES courses(id);

--Modify test table to support assessments + tests localization
ALTER TABLE tests ADD COLUMN course_id INTEGER;
ALTER TABLE tests ADD CONSTRAINT fk_test_course
  FOREIGN KEY (course_id) REFERENCES courses(id);

-- ADD scope field to clarify assessment/test type
CREATE TYPE assessment_scope_enum AS ENUM (
  'course', --tied to specific course
  'grade_level', -- all courses in a grade
  'all-required' -- attainment based (all grades)
);

ALTER TABLE assessments ADD COLUMN scope assessment_scope_enum;
ALTER TABLE tests ADD COLUMN scope assessment_scope_enum;


--Link scores to assessment and enrollment context
ALTER TABLE scores ADD COLUMN enrollment_id INTEGER;
ALTER TABLE scores ADD CONSTRAINT fk_scores_enrollment
  FOREIGN KEY (enrollment_id) REFERENCES student_enrollments(id);
