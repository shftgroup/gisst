-- Add down migration script here
CREATE TABLE IF NOT EXISTS image (
    image_id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id             uuid NOT NULL,
    image_description   text,
    image_config        jsonb,
    created_on  timestamptz DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS environmentImage (
    environment_id              uuid,
    image_id                    uuid,
    environment_image_config    jsonb,
    PRIMARY KEY (environment_id, image_id)
);
--COPY image FROM 'cleanup_crud.image.sql';
--COPY environmentImage FROM 'cleanup_crud.environmentImage.sql';
