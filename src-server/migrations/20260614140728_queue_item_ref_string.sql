-- Add migration script here
DROP TABLE episode_queue_temp_files;
CREATE TABLE episode_queue_temp_files
(
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    episode_queue_id TEXT NOT NULL REFERENCES episode_queue (id),
    file_path        TEXT NOT NULL
);