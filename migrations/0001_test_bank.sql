/*Defining all types so they can be used called in the tables that follow */
CREATE TYPE testarea_enum AS ENUM ('Reading', 'Math');
CREATE TYPE questiontype_enum AS ENUM ('Multiple choice', 'Written', 'Selection', 'True False');
CREATE TYPE gender_enum AS ENUM ('Male', 'Female', 'Non-binary');
CREATE TYPE esl_enum AS ENUM ('Not Applicable', 'Spanish', 'Arabic', 'Mandarin', 'Cantonese', 'Vietnamese', 'Nepali', 'French', 'Russian', 'Somali', 'Amharic', 'Hindi', 'Telugu', 'Tamil', 'Other');
CREATE TYPE grade_enum AS ENUM ('Kindergarten', '1st Grade', '2nd Grade', '3rd Grade', '4th Grade', '5th Grade', '6th Grade', '7th Grade', '8th Grade', '9th Grade', '10th Grade', '11th Grade', '12th Grade');
/*We define the tables after so that they can utilize the Types */
CREATE TABLE IF NOT EXISTS tests (
  name VARCHAR(100) NOT NULL,
  score INT NOT NULL,
  comments TEXT NOT NULL,
  testarea testarea_enum NOT NULL,
  test_id UUID DEFAULT gen_random_uuid(),
  PRIMARY KEY(test_id)
  );

CREATE TABLE IF NOT EXISTS question_table (
  word_problem TEXT,
  point_value INT,
  question_type questiontype_enum NOT NULL,
  options TEXT [],
  correct_answer TEXT,
  qnumber SERIAL PRIMARY KEY,
  testlinker UUID NOT NULL,
  CONSTRAINT fk_testlinker
    FOREIGN KEY (testlinker)
    REFERENCES tests (test_id)
    ON DELETE CASCADE,
  CHECK (correct_answer = ANY(options))
);

CREATE TABLE IF NOT EXISTS students (
  firstname VARCHAR(100) NOT NULL,
  lastname VARCHAR(100) NOT NUll,
  gender gender_enum NOT NULL,
  date_of_birth DATE NOT NULL,
  student_id INT PRIMARY KEY NOT NULL,
  esl esl_enum DEFAULT 'Not Applicable', 
  grade grade_enum NOT NULL,
  teacher TEXT NOT NULL,
  iep BOOLEAN NOT NULL DEFAULT FALSE,
  student_504 BOOLEAN NOT NULL DEFAULT FALSE,
  readplan BOOLEAN NOT NULL DEFAULT FALSE,
  gt BOOLEAN NOT NULL DEFAULT FALSE,
  intervention BOOLEAN NOT NULL DEFAULT FALSE,
  eye_glasses BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS scores (
  student_id INT NOT NULL,
  date_administered TIMESTAMP DEFAULT now(),
  test_id UUID NOT NULL,
  test_scores INT [],
  comments TEXT [],
  test_variant INT NOT NULL,
  evaluator TEXT NOT NULL,
  CONSTRAINT fk_testlinker
    FOREIGN KEY (test_id)
    REFERENCES tests (test_id)
    ON DELETE CASCADE
);

CREATE TYPE status_enum AS ENUM (
  'Active',
  'On Leave',
  'Part-time',
  'Not Applicable'
);

CREATE TYPE employee_role AS ENUM (
  'Teacher',
  'Assistant Principal',
  'Principal',
  'Interventionist',
  'Integrated Services',
  'Speech',
  'O/T',
  'Psychologist',
  'Para-Professional',
  'Assessment Coordinator',
  'Other'
);

CREATE TABLE employees (
  id SERIAL PRIMARY KEY,
  firstname VARCHAR(255) NOT NULL,
  lastname VARCHAR(255) NOT NULL,
  status status_enum NOT NULL,
  role employee_role NOT NULL,
  grade grade_enum
);
/*CREATE EXTENSION IF NOT EXISTS pg_trgm;*/

/*CREATE TABLE IF NOT EXISTS students (
  uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  grade TEXT NOT NULL,
  student_id INT NOT NULL
);


INSERT INTO students (name, grade, student_id) VALUES ('Thien Le', '1st grade', 52884);
INSERT INTO tests (name, score, comments, testarea) VALUES ('The Huckleberry Games', 50, '', 'Math');*/
/*INSERT INTO question_table (word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker)*/
