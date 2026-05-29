-- Add up migration script here

CREATE TABLE IF NOT EXISTS rdb_work (
  platform text NOT NULL,
  name text NOT NULL,
  serial text,
  sha1 text,
  crc text,
  md5 text,
  size bigint,
  description text,

  releaseyear integer,
  releasemonth integer,
  releaseday integer,

  genre text,
  analog boolean,
  famitsu_rating integer,
  franchise text,
  publisher text,
  rom_name text,
  users integer,
  esrb_rating text,
  edge_issue integer,
  rumble boolean,
  origin text,
  enhancement_hw text,
  elspa_rating text,
  edge_rating integer,
  region text,
  developer text,
  PRIMARY KEY(platform, name)
);
