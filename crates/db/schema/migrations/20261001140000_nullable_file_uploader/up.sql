ALTER TABLE files DROP CONSTRAINT files_uploaded_by_fkey;
ALTER TABLE files ALTER COLUMN uploaded_by DROP NOT NULL;
ALTER TABLE files ADD CONSTRAINT files_uploaded_by_fkey FOREIGN KEY (uploaded_by) REFERENCES users(id) ON DELETE SET NULL;
