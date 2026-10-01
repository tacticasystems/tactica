-- Refuse rollback if deleted-account attribution cannot satisfy the original schema.
ALTER TABLE files ALTER COLUMN uploaded_by SET NOT NULL;
ALTER TABLE files DROP CONSTRAINT files_uploaded_by_fkey;
ALTER TABLE files ADD CONSTRAINT files_uploaded_by_fkey FOREIGN KEY (uploaded_by) REFERENCES users(id) ON DELETE RESTRICT;
