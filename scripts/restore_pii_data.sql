-- Fixed Restore Original Student IDs Script
-- This version addresses the foreign key constraint issues by updating in the correct order

-- Create temporary table to load the CSV mapping data
DROP TABLE IF EXISTS temp_student_id_mapping;
CREATE TEMPORARY TABLE temp_student_id_mapping (
    app_id INTEGER NOT NULL,           -- Current anonymized ID (100000+)
    student_id INTEGER NOT NULL,  -- Original ID to restore to
    firstname VARCHAR(100) NOT NULL,
    lastname VARCHAR(100) NOT NULL,
    pin INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NULL
);

-- Load the CSV file with error handling
DO $$
DECLARE
    file_loaded BOOLEAN := FALSE;
    file_paths TEXT[] := ARRAY[
      '/var/tmp/student_id_mapping.csv',
      '/opt/postgres_files/student_id_mapping.csv',
      '~/student_id_mapping.csv',
      './student_id_mapping.csv',
      '/tmp/student_id_mapping.csv',
      'student_id_mapping.csv',
      '/dahlia/student_id_mapping.csv'
    ];
    file_path TEXT;
    cmd TEXT;
BEGIN
    FOREACH file_path IN ARRAY file_paths LOOP
        BEGIN
            cmd := 'COPY temp_student_id_mapping(app_id, student_id, firstname, lastname, pin, created_at) FROM ''' || file_path || ''' WITH CSV HEADER';
            EXECUTE cmd;
            file_loaded := TRUE;
            RAISE NOTICE 'Successfully loaded mapping file from: %', file_path;
            EXIT;
        EXCEPTION WHEN OTHERS THEN
            RAISE NOTICE 'Failed to load from %, trying next location... Error: %', file_path, SQLERRM;
            CONTINUE;
        END;
    END LOOP;
    
    IF NOT file_loaded THEN
        RAISE EXCEPTION 'Could not load mapping file from any of the expected locations: %. Make sure the CSV file exists and is readable.', file_paths;
    END IF;
END $$;

-- Verify mapping data
DO $$
DECLARE
    mapping_count INTEGER;
    students_to_restore INTEGER;
    scores_to_restore INTEGER;
    enrollments_to_restore INTEGER;
BEGIN
    SELECT COUNT(*) INTO mapping_count FROM temp_student_id_mapping;
    
    -- Count records that need restoration
    SELECT COUNT(*) INTO students_to_restore 
    FROM students s JOIN temp_student_id_mapping m ON s.student_id = m.app_id;
    
    SELECT COUNT(*) INTO scores_to_restore 
    FROM scores sc JOIN temp_student_id_mapping m ON sc.student_id = m.app_id;
    
    SELECT COUNT(*) INTO enrollments_to_restore 
    FROM student_enrollments se JOIN temp_student_id_mapping m ON se.student_id = m.app_id;
    
    RAISE NOTICE '=== PRE-RESTORE VERIFICATION ===';
    RAISE NOTICE 'Total mappings loaded: %', mapping_count;
    RAISE NOTICE 'Students to restore: %', students_to_restore;
    RAISE NOTICE 'Scores to restore: %', scores_to_restore;
    RAISE NOTICE 'Enrollments to restore: %', enrollments_to_restore;
    
    IF students_to_restore = 0 THEN
        RAISE WARNING 'No students found with anonymized IDs matching the mapping file';
    END IF;
END $$;

-- Start transaction for all updates
BEGIN;

-- CRITICAL: Drop ALL foreign key constraints before any updates
ALTER TABLE scores DROP CONSTRAINT IF EXISTS fk_scores_student_id_fkey;
ALTER TABLE scores DROP CONSTRAINT IF EXISTS scores_student_id_fkey;
ALTER TABLE student_enrollments DROP CONSTRAINT IF EXISTS fk_student_enrollments_student_id_fkey;
ALTER TABLE student_enrollments DROP CONSTRAINT IF EXISTS student_enrollments_student_id_fkey;
ALTER TABLE student_enrollments DROP CONSTRAINT IF EXISTS fk_student_enrollments_student;

-- Step 1: Update the STUDENTS table FIRST (most important)
-- This ensures the target IDs exist before updating dependent tables
UPDATE students 
SET student_id = m.student_id
FROM temp_student_id_mapping m 
WHERE students.student_id = m.app_id;

UPDATE students
SET firstname = m.firstname,
    lastname = m.lastname,
    pin = m.pin
FROM temp_student_id_mapping m 
WHERE students.student_id = m.student_id;

-- Verify students update
DO $$
DECLARE
    updated_students INTEGER;
BEGIN
    SELECT COUNT(*) INTO updated_students
    FROM students s 
    JOIN temp_student_id_mapping m ON s.student_id = m.student_id;
    
    RAISE NOTICE 'Updated % student records', updated_students;
    
    IF updated_students = 0 THEN
        RAISE WARNING 'No students were updated - this may indicate data was already restored';
    END IF;
END $$;

-- Step 2: Update SCORES table (now that student IDs exist)
UPDATE scores 
SET student_id = m.student_id
FROM temp_student_id_mapping m 
WHERE scores.student_id = m.app_id;

-- Verify scores update
DO $$
DECLARE
    updated_scores INTEGER;
    orphaned_scores INTEGER;
BEGIN
    SELECT COUNT(*) INTO updated_scores
    FROM scores s 
    JOIN temp_student_id_mapping m ON s.student_id = m.student_id;
    
    -- Check for orphaned scores
    SELECT COUNT(*) INTO orphaned_scores
    FROM scores s
    LEFT JOIN students st ON s.student_id = st.student_id
    WHERE st.student_id IS NULL;
    
    RAISE NOTICE 'Updated % score records', updated_scores;
    
    IF orphaned_scores > 0 THEN
        RAISE WARNING 'Found % orphaned score records after update', orphaned_scores;
    END IF;
END $$;

-- Step 3: Update STUDENT_ENROLLMENTS table (now that student IDs exist)
UPDATE student_enrollments 
SET student_id = m.student_id
FROM temp_student_id_mapping m 
WHERE student_enrollments.student_id = m.app_id;

-- Verify enrollments update
DO $$
DECLARE
    updated_enrollments INTEGER;
    orphaned_enrollments INTEGER;
BEGIN
    SELECT COUNT(*) INTO updated_enrollments
    FROM student_enrollments se 
    JOIN temp_student_id_mapping m ON se.student_id = m.student_id;
    
    -- Check for orphaned enrollments
    SELECT COUNT(*) INTO orphaned_enrollments
    FROM student_enrollments se
    LEFT JOIN students st ON se.student_id = st.student_id
    WHERE st.student_id IS NULL;
    
    RAISE NOTICE 'Updated % enrollment records', updated_enrollments;
    
    IF orphaned_enrollments > 0 THEN
        RAISE WARNING 'Found % orphaned enrollment records after update', orphaned_enrollments;
    END IF;
END $$;

-- Step 4: Re-create foreign key constraints && enforce NOT NULL for firstname, lastname, pin
-- Only add constraints if we have data that would satisfy them
DO $$
DECLARE
    student_count INTEGER;
    score_count INTEGER;
    enrollment_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO student_count FROM students;
    SELECT COUNT(*) INTO score_count FROM scores;
    SELECT COUNT(*) INTO enrollment_count FROM student_enrollments;
    
    IF student_count > 0 AND score_count > 0 THEN
        ALTER TABLE scores ADD CONSTRAINT fk_scores_student_id_fkey
            FOREIGN KEY (student_id) REFERENCES students(student_id) ON DELETE CASCADE;
        RAISE NOTICE 'Added foreign key constraint for scores table';
    END IF;
    
    IF student_count > 0 AND enrollment_count > 0 THEN
        ALTER TABLE student_enrollments ADD CONSTRAINT fk_student_enrollments_student_id_fkey
            FOREIGN KEY (student_id) REFERENCES students(student_id) ON DELETE CASCADE;
        RAISE NOTICE 'Added foreign key constraint for student_enrollments table';
    END IF;
END $$;

ALTER TABLE students 
    ALTER COLUMN firstname SET NOT NULL,
    ALTER COLUMN lastname SET NOT NULL,
    ALTER COLUMN pin SET NOT NULL;

-- Commit all changes
COMMIT;

-- Final comprehensive verification
DO $$
DECLARE
    students_restored INTEGER;
    scores_restored INTEGER;
    enrollments_restored INTEGER;
    students_still_anon INTEGER;
    scores_still_anon INTEGER;
    enrollments_still_anon INTEGER;
    min_student_id INTEGER;
    max_student_id INTEGER;
BEGIN
    -- Count restored records (those with original IDs)
    SELECT COUNT(*) INTO students_restored 
    FROM students s JOIN temp_student_id_mapping m ON s.student_id = m.student_id;
    
    SELECT COUNT(*) INTO scores_restored 
    FROM scores sc JOIN temp_student_id_mapping m ON sc.student_id = m.student_id;
    
    SELECT COUNT(*) INTO enrollments_restored 
    FROM student_enrollments se JOIN temp_student_id_mapping m ON se.student_id = m.student_id;
    
    -- Count still anonymized records
    SELECT COUNT(*) INTO students_still_anon 
    FROM students s JOIN temp_student_id_mapping m ON s.student_id = m.app_id;
    
    SELECT COUNT(*) INTO scores_still_anon 
    FROM scores sc JOIN temp_student_id_mapping m ON sc.student_id = m.app_id;
    
    SELECT COUNT(*) INTO enrollments_still_anon 
    FROM student_enrollments se JOIN temp_student_id_mapping m ON se.student_id = m.app_id;
    
    -- Get ID range
    SELECT MIN(student_id), MAX(student_id) INTO min_student_id, max_student_id FROM students;
    
    RAISE NOTICE '=== FINAL RESTORATION RESULTS ===';
    RAISE NOTICE 'STUDENTS: % restored, % still anonymized', students_restored, students_still_anon;
    RAISE NOTICE 'SCORES: % restored, % still anonymized', scores_restored, scores_still_anon;
    RAISE NOTICE 'ENROLLMENTS: % restored, % still anonymized', enrollments_restored, enrollments_still_anon;
    RAISE NOTICE 'Student ID range: % to %', min_student_id, max_student_id;
    
    -- Status indicators
    IF students_still_anon = 0 AND students_restored > 0 THEN
        RAISE NOTICE '✓ SUCCESS: All student IDs restored';
    ELSIF students_still_anon > 0 THEN
        RAISE WARNING '✗ PARTIAL: % student IDs still anonymized', students_still_anon;
    ELSE
        RAISE WARNING '? NO CHANGE: No student IDs were modified';
    END IF;
    
    IF min_student_id < 100000 THEN
        RAISE NOTICE '✓ Student IDs are in original range (< 100000)';
    ELSE
        RAISE WARNING '? Student IDs still appear to be in anonymized range (100000+)';
    END IF;
END $$;

-- Update global settings
INSERT INTO global_settings (key, value, updated_by) 
VALUES ('student_protections', 'false', 1) 
ON CONFLICT (key) DO UPDATE SET 
    value = 'false',
    updated_at = CURRENT_TIMESTAMP;

-- Clean up
DROP TABLE IF EXISTS temp_student_id_mapping;
DROP TABLE IF EXISTS student_id_mapping;

SELECT 'Student ID restoration process completed. Check the output above for results.' as status;
