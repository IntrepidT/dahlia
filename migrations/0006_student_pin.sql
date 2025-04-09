ALTER TABLE students
ADD COLUMN pin INT NOT NULL DEFAULT 0;

ALTER TABLE scores
ADD CONSTRAINT scores_student_id_fkey
FOREIGN KEY (student_id) REFERENCES students(student_id) ON DELETE CASCADE;
