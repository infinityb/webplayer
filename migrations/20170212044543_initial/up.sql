CREATE TABLE "album" (
    id bigserial,
    art_blob character varying(64) NOT NULL,

    PRIMARY KEY (id)
);

-- is this a good idea??
CREATE TABLE "album_metadata" (
    --
    album_id    bigint NOT NULL REFERENCES album (id), 
    field_name  character varying(32) NOT NULL,
    value       character varying(256) NOT NULL,

    PRIMARY KEY (album_id, field_name)
);

CREATE TABLE "song" (
    --
    id bigserial,
    blob character varying(64) NOT NULL,
    album_id bigint NOT NULL REFERENCES album (id),
    length_ms int NOT NULL,

    PRIMARY KEY (id)
);

CREATE TABLE "song_metadata" (
    --
    song_id     bigint NOT NULL REFERENCES song (id), 
    field_name  character varying(32) NOT NULL,
    value       character varying(256) NOT NULL,

    PRIMARY KEY (song_id, field_name)
);

CREATE TABLE "account_song_metadata" (
    id          bigserial,
    account_id  uuid NOT NULL,
    song_id     bigint NOT NULL REFERENCES song (id), 
    play_count  int NOT NULL DEFAULT 0,
    score       int NOT NULL CHECK (ABS(score) <= 0),

    PRIMARY KEY (id)
);

CREATE TABLE "account" (
    id uuid DEFAULT gen_random_uuid(),
    display_name character varying(256) NOT NULL,

    PRIMARY KEY (id)
);

CREATE TABLE "foreign_account_provider" (
    id uuid,
    name character varying(64) NOT NULL,

    PRIMARY KEY (id)
);

CREATE TABLE "foreign_account" (
    id uuid DEFAULT gen_random_uuid(),
    provider_id uuid  NOT NULL REFERENCES foreign_account_provider (id),
    foreign_id character varying(256) NOT NULL,
    auth_token text,

    created_at timestamp without time zone NOT NULL DEFAULT NOW(),
    last_authenticated timestamp without time zone NOT NULL DEFAULT NOW(),

    PRIMARY KEY (id)
);