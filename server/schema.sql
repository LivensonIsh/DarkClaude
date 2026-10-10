CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  username TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  plan TEXT NOT NULL DEFAULT 'free',
  premium_until TIMESTAMPTZ,
  cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
  is_admin BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS sessions (
  token_hash TEXT PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS payments (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  plan TEXT NOT NULL,
  method TEXT NOT NULL,
  reference TEXT NOT NULL,
  amount_htg BIGINT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  processed_at TIMESTAMPTZ
);
CREATE TABLE IF NOT EXISTS contact_messages (
  id BIGSERIAL PRIMARY KEY,
  name TEXT, email TEXT NOT NULL, message TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
CREATE TABLE IF NOT EXISTS messages (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_messages_user ON messages(user_id, created_at);
CREATE TABLE IF NOT EXISTS conversations (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title TEXT NOT NULL DEFAULT 'Conversation',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_conversations_user ON conversations(user_id, created_at);
ALTER TABLE messages ADD COLUMN IF NOT EXISTS conversation_id BIGINT REFERENCES conversations(id) ON DELETE CASCADE;
DO $$
DECLARE r RECORD; cid BIGINT;
BEGIN
  FOR r IN SELECT DISTINCT user_id FROM messages WHERE conversation_id IS NULL LOOP
    INSERT INTO conversations(user_id, title) VALUES (r.user_id, 'Conversation') RETURNING id INTO cid;
    UPDATE messages SET conversation_id = cid WHERE user_id = r.user_id AND conversation_id IS NULL;
  END LOOP;
END $$;
CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id, created_at);
ALTER TABLE payments ADD COLUMN IF NOT EXISTS proof_data_url TEXT;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS sender_number TEXT;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS sender_is_agent BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS notes TEXT;
