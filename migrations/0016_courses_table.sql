-- Create Courses Table
CREATE TABLE courses (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  subject VARCHAR(255),
  course_code VARCHAR(20),
  course_level grade_enum,
  teacher_id INT,
  academic_year school_year_enum NOT NULL,
  semester_period VARCHAR(20), -- e.g., "Fall", "Spring"
  credits DECIMAL(3,2) DEFAULT 1.00,
  description TEXT,
  max_students INT DEFAULT 30,
  room_number VARCHAR(20),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

  --Foreign Key Constraing to employees
  CONSTRAINT fk_courses_teacher
    FOREIGN KEY (teacher_id)
    REFERENCES employees(id)
    ON DELETE SET NULL
);

--Update student_enrollments to reference courses in the courses Table
ALTER TABLE student_enrollments
ADD COLUMN IF NOT EXISTS course_id INT,
ADD COLUMN IF NOT EXISTS final_grade VARCHAR(5),
ADD COLUMN IF NOT EXISTS grade_points DECIMAL(3,2);

-- Rename grade column to current_grade_level for clarity
ALTER TABLE students
RENAME COLUMN grade TO current_grade_level;

--Foreign key constraints for course_id
ALTER TABLE student_enrollments
ADD CONSTRAINT fk_student_enrollments_course
  FOREIGN KEY (course_id)
  REFERENCES courses(id)
  ON DELETE SET NULL;

-- Update students table to include graduation_year and student_status
ALTER TABLE students 
ADD COLUMN IF NOT EXISTS graduation_year INT,
ADD COLUMN IF NOT EXISTS student_status VARCHAR(20) DEFAULT 'active';

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_courses_teacher ON courses(teacher_id);
CREATE INDEX IF NOT EXISTS idx_courses_level_year ON courses(course_level, academic_year);
CREATE INDEX IF NOT EXISTS idx_courses_academic_year ON courses(academic_year);
CREATE INDEX IF NOT EXISTS idx_student_enrollments_course ON student_enrollments(course_id);

