-- Add down migration script here

-- split file dest path into path/hash-filename

UPDATE file SET file_dest_path=left(substring(file_dest_path from '.*/'),-1)
