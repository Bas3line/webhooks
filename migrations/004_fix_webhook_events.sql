ALTER TABLE webhook_events ADD COLUMN IF NOT EXISTS webhook_id UUID;
ALTER TABLE webhook_events ADD COLUMN IF NOT EXISTS status_code INTEGER DEFAULT 200;
ALTER TABLE webhook_events ADD COLUMN IF NOT EXISTS user_id UUID;

CREATE INDEX IF NOT EXISTS idx_webhook_events_webhook_id ON webhook_events(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_events_user_id ON webhook_events(user_id);