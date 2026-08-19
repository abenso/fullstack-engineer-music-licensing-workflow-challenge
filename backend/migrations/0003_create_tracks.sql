-- Mirrors the Rust `LicenseStatus` enum in src/domain/license.rs — keep both in sync.
CREATE TYPE license_status AS ENUM (
    'draft',
    'requested',
    'in_negotiation',
    'approved',
    'licensed',
    'rejected'
);

CREATE TABLE tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scene_id UUID NOT NULL REFERENCES scenes (id) ON DELETE CASCADE,
    song_id UUID NOT NULL REFERENCES songs (id),
    start_time_ms INTEGER NOT NULL,
    end_time_ms INTEGER NOT NULL,
    license_status license_status NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT tracks_end_after_start CHECK (end_time_ms > start_time_ms)
);

CREATE INDEX tracks_scene_id_idx ON tracks (scene_id);
CREATE INDEX tracks_song_id_idx ON tracks (song_id);
