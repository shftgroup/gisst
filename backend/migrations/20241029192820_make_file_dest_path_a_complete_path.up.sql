-- Add up migration script here

-- merge file dest path with file_hash and file_filename

UPDATE file SET file_dest_path=format('%s/%s-%s', file_dest_path, file_hash, file_filename);
