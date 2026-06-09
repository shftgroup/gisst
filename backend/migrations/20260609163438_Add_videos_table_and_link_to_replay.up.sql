CREATE TABLE IF NOT EXISTS video (
    video_id   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id    uuid NOT NULL,
    created_on timestamptz NOT NULL DEFAULT current_timestamp,
    creator_id uuid NOT NULL
);
ALTER TABLE video ADD FOREIGN KEY (file_id) REFERENCES file(file_id);
ALTER TABLE video ADD FOREIGN KEY (creator_id) REFERENCES creator(creator_id);

ALTER TABLE replay ADD COLUMN video_id uuid;
ALTER TABLE replay ADD FOREIGN KEY (video_id) REFERENCES video(video_id);
