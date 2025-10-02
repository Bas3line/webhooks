CREATE TABLE IF NOT EXISTS webhook_events (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    headers JSONB NOT NULL,
    body JSONB NOT NULL,
    source_ip TEXT,
    endpoint TEXT NOT NULL,
    event_type TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhook_events_timestamp ON webhook_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_webhook_events_endpoint ON webhook_events(endpoint);
CREATE INDEX IF NOT EXISTS idx_webhook_events_event_type ON webhook_events(event_type);