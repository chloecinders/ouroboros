DROP TABLE IF EXISTS ocr_image_hashes;

CREATE TABLE ocr_image_hashes (
    image_hash CHAR(64) NOT NULL,
    rule_hash CHAR(64) NOT NULL,
    is_match BOOLEAN NOT NULL,
    PRIMARY KEY (image_hash, rule_hash)
);

CREATE INDEX ocr_image_hashes_lookup_idx ON ocr_image_hashes (image_hash);
