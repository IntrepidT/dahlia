--First create courses for all existing teachers in the table
INSERT INTO courses (
  name,
  subject,
  course_code,
  course_level,
  teacher_id,
  academic_year,
  semester_period,
  credits,
  description,
  max_students
)
SELECT DISTINCT
  CONCAT('Grade ', s.current_grade_level, ' Homeroom - ', e.firstname, ' ', e.lastname) as name,
  'Core Curriculum' as subject,
  CONCAT('HR-', s.current_grade_level, '-', LEFT(e.lastname, 3), '-', e.id) as course_code,
  s.current_grade_level as course_level,
  e.id as teacher_id,
  '2024-2025'::school_year_enum as academic_year,
  'Full Year' as semester_period,
  1.00 as credits,
  CONCAT('Homeroom and core curriculum for Grade ', s.current_grade_level, ' students') as description,
  30 as max_students
FROM students s 
JOIN employees e ON e.lastname = s.teacher
WHERE s.teacher IS NOT NULL
  AND s.current_grade_level IS NOT NULL
GROUP BY s.current_grade_level, s.teacher, e.id, e.firstname, e.lastname;

--Now create student enrollments for all existing students to the courses created above
INSERT INTO student_enrollments (
  student_id,
  academic_year,
  grade_level,
  teacher_id,
  course_id,
  status,
  enrollment_date
)
SELECT
  s.student_id,
  '2024-2025'::school_year_enum as academic_year,
  s.current_grade_level as grade_level,
  c.teacher_id,
  c.id as course_id,
  'active'::enrollment_status_enum as status,
  '2024-08-01' as enrollment_date
FROM students s 
JOIN employees e ON e.lastname = s.teacher
JOIN courses c ON c.teacher_id = e.id 
  AND c.course_level = s.current_grade_level
  AND c.subject = 'Core Curriculum'
  AND c.academic_year = '2024-2025'::school_year_enum
WHERE s.teacher IS NOT NULL
  AND s.current_grade_level IS NOT NULL
  AND s.student_id IS NOT NULL;
