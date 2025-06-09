-- Create Mapping table to preserve relationships
CREATE TABLE IF NOT EXISTS student_id_mapping (
    old_student_id INTEGER NOT NULL,
    new_student_id INTEGER NOT NULL,
    firstname VARCHAR(100) NOT NULL,
    lastname VARCHAR(100) NOT NULL,
    pin INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (old_student_id)
);

-- Clear any previous mapping data
DELETE FROM student_id_mapping;

-- Step 1: Create the mapping first (without updating anything yet)
DO $$
DECLARE
    current_student RECORD;
    new_id INTEGER := 100000;
    max_existing_id INTEGER;
BEGIN
    RAISE NOTICE 'Creating ID mappings...';
    
    -- Check if any IDs in our target range already exist
    SELECT COALESCE(MAX(student_id), 0) INTO max_existing_id FROM students WHERE student_id >= 100000;
    IF max_existing_id >= 100000 THEN
        new_id := max_existing_id + 1000; -- Start well above existing IDs
        RAISE NOTICE 'Found existing IDs >= 100000, starting from %', new_id;
    END IF;
    
    FOR current_student IN (SELECT student_id, firstname, lastname, pin FROM students ORDER BY student_id) LOOP
        INSERT INTO student_id_mapping (old_student_id, new_student_id, firstname, lastname, pin) 
        VALUES (current_student.student_id, new_id, current_student.firstname, current_student.lastname, current_student.pin);
        new_id := new_id + 1;
    END LOOP;
    
    RAISE NOTICE 'Created mappings for % students', (SELECT COUNT(*) FROM student_id_mapping);
END $$;

-- Step 2: Verify mapping was created
DO $$
DECLARE
    mapping_count INTEGER;
    student_count INTEGER;
    sample_mapping RECORD;
BEGIN
    SELECT COUNT(*) INTO mapping_count FROM student_id_mapping;
    SELECT COUNT(*) INTO student_count FROM students;
    
    -- Show a sample mapping for verification
    SELECT * INTO sample_mapping FROM student_id_mapping ORDER BY old_student_id LIMIT 1;
    RAISE NOTICE 'Sample mapping: old_id=%, new_id=%', sample_mapping.old_student_id, sample_mapping.new_student_id;
    
    IF mapping_count = 0 THEN
        RAISE EXCEPTION 'ERROR: No mappings were created! Check if students table has data.';
    END IF;
    
    IF mapping_count != student_count THEN
        RAISE EXCEPTION 'Mapping count (%) does not match student count (%)', mapping_count, student_count;
    END IF;
    
    RAISE NOTICE 'Successfully created % student ID mappings', mapping_count;
END $$;

-- Step 3: Show current student IDs before update
DO $$
DECLARE
    sample_student RECORD;
BEGIN
    RAISE NOTICE 'BEFORE UPDATE - Sample current student IDs:';
    FOR sample_student IN (SELECT student_id FROM students ORDER BY student_id LIMIT 3) LOOP
        RAISE NOTICE 'Current student_id: %', sample_student.student_id;
    END LOOP;
END $$;

-- Step 4: Start transaction for all updates
BEGIN;

-- Step 5: Drop foreign key constraints to allow updates && change pin, firstname, lastname to be nullable
ALTER TABLE scores DROP CONSTRAINT IF EXISTS fk_scores_student_id_fkey;
ALTER TABLE scores DROP CONSTRAINT IF EXISTS scores_student_id_fkey;
ALTER TABLE student_enrollments DROP CONSTRAINT IF EXISTS fk_student_enrollments_student;
ALTER TABLE student_enrollments DROP CONSTRAINT IF EXISTS fk_student_enrollments_student_id_fkey;
ALTER TABLE student_enrollments DROP CONSTRAINT IF EXISTS student_enrollments_student_id_fkey;

ALTER TABLE students ALTER COLUMN pin DROP NOT NULL;
ALTER TABLE students ALTER COLUMN firstname DROP NOT NULL;
ALTER TABLE students ALTER COLUMN lastname DROP NOT NULL;

-- Step 6: Update students table FIRST (parent table)
UPDATE students 
SET student_id = (
    SELECT m.new_student_id 
    FROM student_id_mapping m 
    WHERE m.old_student_id = students.student_id
)
WHERE EXISTS (
    SELECT 1 FROM student_id_mapping m 
    WHERE m.old_student_id = students.student_id
);
-- clear pii data from table
UPDATE students
SET pin = NULL, firstname = NULL, lastname = NULL
WHERE EXISTS (
    SELECT 1 FROM student_id_mapping m 
    WHERE m.new_student_id = students.student_id
);

-- Step 7: Update child tables using the mapping
-- Update scores table
UPDATE scores 
SET student_id = (
    SELECT m.new_student_id 
    FROM student_id_mapping m 
    WHERE m.old_student_id = scores.student_id
)
WHERE EXISTS (
    SELECT 1 FROM student_id_mapping m 
    WHERE m.old_student_id = scores.student_id
);

-- Update student_enrollments table
UPDATE student_enrollments 
SET student_id = (
    SELECT m.new_student_id 
    FROM student_id_mapping m 
    WHERE m.old_student_id = student_enrollments.student_id
)
WHERE EXISTS (
    SELECT 1 FROM student_id_mapping m 
    WHERE m.old_student_id = student_enrollments.student_id
);

-- Step 8: Re-enable foreign key constraints
ALTER TABLE scores ADD CONSTRAINT scores_student_id_fkey
    FOREIGN KEY (student_id) REFERENCES students(student_id) ON DELETE CASCADE;

ALTER TABLE student_enrollments ADD CONSTRAINT student_enrollments_student_id_fkey
    FOREIGN KEY (student_id) REFERENCES students(student_id) ON DELETE CASCADE;

-- Commit the transaction
COMMIT;

-- Step 9: Verify the changes actually took place
DO $$
DECLARE
    min_new_id INTEGER;
    max_new_id INTEGER;
    student_count INTEGER;
    old_id_exists BOOLEAN := False;
    sample_student RECORD;
BEGIN
    SELECT MIN(student_id), MAX(student_id), COUNT(*) 
    INTO min_new_id, max_new_id, student_count 
    FROM students;
    
    RAISE NOTICE 'Student table verification:';
    RAISE NOTICE '- Count: %', student_count;
    RAISE NOTICE '- ID range: % to %', min_new_id, max_new_id;
    
    -- Show sample of new IDs
    RAISE NOTICE 'AFTER UPDATE - Sample new student IDs:';
    FOR sample_student IN (SELECT student_id FROM students ORDER BY student_id LIMIT 5) LOOP
        RAISE NOTICE 'New student_id: %', sample_student.student_id;
    END LOOP;
    
    -- Check if any old IDs still exist (they shouldn't if update worked)
    SELECT EXISTS(
        SELECT 1 FROM students s 
        JOIN student_id_mapping m ON s.student_id = m.old_student_id
    ) INTO old_id_exists;
    
    IF old_id_exists THEN
        RAISE WARNING 'WARNING: Some old student IDs still exist in students table!';
    ELSE
        RAISE NOTICE 'SUCCESS: All student IDs have been updated to new values';
    END IF;
    
    -- Verify new IDs are in expected range
    IF min_new_id >= 100000 THEN
        RAISE NOTICE 'SUCCESS: New IDs are in expected range (100000+)';
    ELSE
        RAISE WARNING 'WARNING: Some student IDs are not in expected range (100000+)';
    END IF;
END $$;

-- Step 10: Comprehensive verification that ALL tables have new IDs
DO $$
DECLARE
    students_updated INTEGER;
    scores_updated INTEGER;
    enrollments_updated INTEGER;
    students_total INTEGER;
    scores_total INTEGER;
    enrollments_total INTEGER;
    students_old_ids INTEGER;
    scores_old_ids INTEGER;
    enrollments_old_ids INTEGER;
BEGIN
    -- Count total records
    SELECT COUNT(*) INTO students_total FROM students;
    SELECT COUNT(*) INTO scores_total FROM scores;
    SELECT COUNT(*) INTO enrollments_total FROM student_enrollments;
    
    -- Count records that now have new IDs (exist in mapping as new_student_id)
    SELECT COUNT(*) INTO students_updated 
    FROM students s
    INNER JOIN student_id_mapping m ON s.student_id = m.new_student_id;
    
    SELECT COUNT(*) INTO scores_updated 
    FROM scores sc
    INNER JOIN student_id_mapping m ON sc.student_id = m.new_student_id;
    
    SELECT COUNT(*) INTO enrollments_updated 
    FROM student_enrollments se
    INNER JOIN student_id_mapping m ON se.student_id = m.new_student_id;
    
    -- Count records that still have old IDs (exist in mapping as old_student_id)
    SELECT COUNT(*) INTO students_old_ids 
    FROM students s
    INNER JOIN student_id_mapping m ON s.student_id = m.old_student_id;
    
    SELECT COUNT(*) INTO scores_old_ids 
    FROM scores sc
    INNER JOIN student_id_mapping m ON sc.student_id = m.old_student_id;
    
    SELECT COUNT(*) INTO enrollments_old_ids 
    FROM student_enrollments se
    INNER JOIN student_id_mapping m ON se.student_id = m.old_student_id;
    
    RAISE NOTICE '=== ID CONVERSION VERIFICATION ===';
    RAISE NOTICE 'STUDENTS table:';
    RAISE NOTICE '  Total records: %', students_total;
    RAISE NOTICE '  Records with NEW IDs: %', students_updated;
    RAISE NOTICE '  Records with OLD IDs: %', students_old_ids;
    
    RAISE NOTICE 'SCORES table:';
    RAISE NOTICE '  Total records: %', scores_total;
    RAISE NOTICE '  Records with NEW IDs: %', scores_updated;
    RAISE NOTICE '  Records with OLD IDs: %', scores_old_ids;
    
    RAISE NOTICE 'STUDENT_ENROLLMENTS table:';
    RAISE NOTICE '  Total records: %', enrollments_total;
    RAISE NOTICE '  Records with NEW IDs: %', enrollments_updated;
    RAISE NOTICE '  Records with OLD IDs: %', enrollments_old_ids;
    
    -- Check for success
    IF students_updated = students_total AND students_old_ids = 0 THEN
        RAISE NOTICE '✓ STUDENTS: All IDs successfully converted';
    ELSE
        RAISE WARNING '✗ STUDENTS: ID conversion incomplete';
    END IF;
    
    IF scores_updated = scores_total AND scores_old_ids = 0 THEN
        RAISE NOTICE '✓ SCORES: All IDs successfully converted';
    ELSE
        RAISE WARNING '✗ SCORES: ID conversion incomplete';
    END IF;
    
    IF enrollments_updated = enrollments_total AND enrollments_old_ids = 0 THEN
        RAISE NOTICE '✓ ENROLLMENTS: All IDs successfully converted';
    ELSE
        RAISE WARNING '✗ ENROLLMENTS: ID conversion incomplete';
    END IF;
    
    RAISE NOTICE '==================================';
END $$;

-- Step 11: Verify data integrity
DO $$
DECLARE
    orphaned_scores INTEGER;
    orphaned_enrollments INTEGER;
    total_scores INTEGER;
    total_enrollments INTEGER;
BEGIN
    -- Count total records
    SELECT COUNT(*) INTO total_scores FROM scores;
    SELECT COUNT(*) INTO total_enrollments FROM student_enrollments;
    
    -- Check for orphaned scores
    SELECT COUNT(*) INTO orphaned_scores
    FROM scores s
    LEFT JOIN students st ON s.student_id = st.student_id
    WHERE st.student_id IS NULL;
    
    -- Check for orphaned enrollments
    SELECT COUNT(*) INTO orphaned_enrollments
    FROM student_enrollments se
    LEFT JOIN students st ON se.student_id = st.student_id
    WHERE st.student_id IS NULL;
    
    RAISE NOTICE 'Data integrity verification:';
    RAISE NOTICE '- Total scores: %, orphaned: %', total_scores, orphaned_scores;
    RAISE NOTICE '- Total enrollments: %, orphaned: %', total_enrollments, orphaned_enrollments;
    
    IF orphaned_scores > 0 THEN
        RAISE EXCEPTION 'Found % orphaned score records out of %', orphaned_scores, total_scores;
    END IF;
    
    IF orphaned_enrollments > 0 THEN
        RAISE EXCEPTION 'Found % orphaned enrollment records out of %', orphaned_enrollments, total_enrollments;
    END IF;
    
    RAISE NOTICE 'SUCCESS: Data integrity check passed - no orphaned records found';
END $$;

-- Step 12: Export mapping relationship to CSV
\copy (SELECT new_student_id as app_id, old_student_id as original_student_id, firstname as firstname, lastname as lastname, pin as pin,  created_at FROM student_id_mapping ORDER BY new_student_id) TO '/var/tmp/student_id_mapping.csv' WITH CSV HEADER;

-- Step 13: Insert/Update global setting to track protection status
UPDATE global_settings SET value = 'true' WHERE key = 'student_protections';
-- Final summary with detailed verification
DO $$
DECLARE
    total_students INTEGER;
    total_mappings INTEGER;
    min_old_id INTEGER;
    max_old_id INTEGER;
    min_new_id INTEGER;
    max_new_id INTEGER;
    verification_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_students FROM students;
    SELECT COUNT(*) INTO total_mappings FROM student_id_mapping;
    
    -- Get ID ranges for verification
    SELECT MIN(old_student_id), MAX(old_student_id) INTO min_old_id, max_old_id FROM student_id_mapping;
    SELECT MIN(new_student_id), MAX(new_student_id) INTO min_new_id, max_new_id FROM student_id_mapping;
    
    -- Verify that all students now have IDs that exist in the mapping as new IDs
    SELECT COUNT(*) INTO verification_count 
    FROM students s 
    JOIN student_id_mapping m ON s.student_id = m.new_student_id;
    
    RAISE NOTICE '=== STUDENT ID REPLACEMENT COMPLETE ===';
    RAISE NOTICE 'Total students processed: %', total_students;
    RAISE NOTICE 'Total ID mappings created: %', total_mappings;
    RAISE NOTICE 'Students with new IDs: %', verification_count;
    RAISE NOTICE 'Original ID range: % to %', min_old_id, max_old_id;
    RAISE NOTICE 'New ID range: % to %', min_new_id, max_new_id;
    RAISE NOTICE 'CSV mapping file: student_id_mapping.csv';
    
    IF verification_count = total_students THEN
        RAISE NOTICE 'STATUS: ✓ SUCCESS - All student IDs successfully replaced';
    ELSE
        RAISE WARNING 'STATUS: ✗ PARTIAL - Only %/% students have new IDs', verification_count, total_students;
    END IF;
    RAISE NOTICE '==========================================';
END $$;
