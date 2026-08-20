CREATE TABLE songs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    rights_holder TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
