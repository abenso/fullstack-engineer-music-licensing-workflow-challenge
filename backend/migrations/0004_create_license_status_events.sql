-- Audit trail: one row per license_status transition on a track.
CREATE TABLE license_status_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    track_id UUID NOT NULL REFERENCES tracks (id) ON DELETE CASCADE,
    from_status license_status,
    to_status license_status NOT NULL,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX license_status_events_track_id_idx ON license_status_events (track_id);
