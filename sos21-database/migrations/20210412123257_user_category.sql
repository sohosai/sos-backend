CREATE TYPE user_category AS ENUM ('undergraduate', 'graduate_student', 'academic_staff');

ALTER TABLE users ADD COLUMN category user_category;
UPDATE users SET category = 'undergraduate';
ALTER TABLE users ALTER COLUMN category SET NOT NULL;
