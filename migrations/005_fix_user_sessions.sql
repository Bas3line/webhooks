-- Add user_id column to user_sessions table if it doesn't exist
ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE;

-- Create index for user_id if it doesn't exist
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);