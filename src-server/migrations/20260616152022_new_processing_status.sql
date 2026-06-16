DELETE
FROM episode_queue_temp_files;

DROP TABLE episode_queue;

CREATE TABLE episode_queue
(
    id             TEXT PRIMARY KEY,
    media_id       INT  NOT NULL,
    episode_number REAL NOT NULL,
    file_path      TEXT,
    output_path    TEXT NOT NULL,
    step           TEXT NOT NULL CHECK (
        step IN ('IN_QUEUE',
                 'PROCESSING_1080P',
                 'PROCESSING_720P',
                 'PROCESSING_AUDIO',
                 'PROCESSING_SUBTITLES',
                 'PACKAGING',
                 'DONE',
                 'CANCELLED')
        ),
    progress       REAL,
    created_at     INT  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     INT  NOT NULL DEFAULT CURRENT_TIMESTAMP
);
