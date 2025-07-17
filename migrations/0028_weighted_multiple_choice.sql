-- Alter table to add weighted_multiple_choice column
ALTER TABLE question_table ADD COLUMN weighted_multiple_choice TEXT;

--Alter questiontype_enum to include weighted_multiple_choice
ALTER TYPE questiontype_enum ADD VALUE 'Weighted Multiple Choice';
