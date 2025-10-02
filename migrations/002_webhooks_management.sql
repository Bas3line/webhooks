CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    endpoint TEXT UNIQUE NOT NULL,
    secret TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS webhook_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL,
    headers JSONB NOT NULL,
    body JSONB NOT NULL,
    source_ip TEXT,
    endpoint TEXT NOT NULL,
    event_type TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

ALTER TABLE webhook_events ADD COLUMN IF NOT EXISTS webhook_id UUID;
ALTER TABLE webhook_events ADD COLUMN IF NOT EXISTS status_code INTEGER DEFAULT 200;

CREATE INDEX IF NOT EXISTS idx_webhook_events_webhook_id ON webhook_events(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_events_timestamp ON webhook_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_webhook_events_endpoint ON webhook_events(endpoint);
CREATE INDEX IF NOT EXISTS idx_webhook_events_event_type ON webhook_events(event_type);
CREATE INDEX IF NOT EXISTS idx_webhooks_endpoint ON webhooks(endpoint);
CREATE INDEX IF NOT EXISTS idx_webhooks_active ON webhooks(is_active);