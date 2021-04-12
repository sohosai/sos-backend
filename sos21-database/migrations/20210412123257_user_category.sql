CREATE TYPE user_category AS ENUM ('undergraduate_student', 'graduate_student', 'academic_staff');

ALTER TABLE users ADD COLUMN category user_category;
UPDATE users SET category = 'undergraduate_student';
ALTER TABLE users ALTER COLUMN category SET NOT NULL;
