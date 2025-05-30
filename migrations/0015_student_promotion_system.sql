--Create school year enum for tracking academic years
CREATE TYPE school_year_enum AS ENUM (
  '2023-2024',
  '2024-2025',
  '2025-2026',
  '2026-2027',
  '2027-2028',
  '2028-2029',
  '2029-2030'
);

--Create status enum for tracking enrollment status
CREATE TYPE enrollment_status_enum As ENUM (
  'active',
  'inactive',
  'graduated',
  'transferred',
  'dropped'
);

--Create student promotion history table
CREATE TABLE student_enrollments (
  id SERIAL PRIMARY KEY,
  student_id INT NOT NULL,
  academic_year school_year_enum NOT NULL,
  grade_level grade_enum NOT NULL,
  teacher_id INT NOT NULL,
  status enrollment_status_enum NOT NULL DEFAULT 'active',
  enrollment_date DATE NOT NULL DEFAULT CURRENT_DATE,
  status_change_date DATE,
  notes TEXT,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

  -- Foreign key constraints
  CONSTRAINT fk_student_enrollments_student
    FOREIGN KEY (student_id)
    REFERENCES students(student_id)
    ON DELETE CASCADE,

  CONSTRAINT fk_student_enrollments_teacher
    FOREIGN KEY (teacher_id)
    REFERENCES employees(id)
    ON DELETE SET NULL
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_student_enrollments_student_id 
    ON student_enrollments(student_id);

CREATE INDEX IF NOT EXISTS idx_student_enrollments_academic_year 
    ON student_enrollments(academic_year);

CREATE INDEX IF NOT EXISTS idx_student_enrollments_grade_level 
    ON student_enrollments(grade_level);

CREATE INDEX IF NOT EXISTS idx_student_enrollments_teacher_id 
    ON student_enrollments(teacher_id);

CREATE INDEX IF NOT EXISTS idx_student_enrollments_status 
    ON student_enrollments(status);

CREATE INDEX IF NOT EXISTS idx_student_enrollments_student_year 
    ON student_enrollments(student_id, academic_year);

