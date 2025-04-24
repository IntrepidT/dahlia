CREATE TYPE session_type_enum AS ENUM ('chat', 'test');

--Add sesion type column and test_specific columns to sessions table
ALTER TABLE websocket_sessions
  ADD COLUMN session_type session_type_enum NOT NULL DEFAULT 'chat',
  ADD COLUMN test_id VARCHAR(255),
  ADD COLUMN start_time TIMESTAMP WITH TIME ZONE,
  ADD COLUMN end_time TIMESTAMP WITH TIME ZONE;


CREATE INDEX sessions_session_type_idx ON websocket_sessions(session_type);
CREATE INDEX sessions_test_id_idx ON websocket_sessions(test_id) WHERE test_id IS NOT NULL;

