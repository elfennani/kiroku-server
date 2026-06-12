-- Add migration script here
DELETE FROM sessions;
ALTER TABLE sessions ADD COLUMN user_id INT NOT NULL default -1