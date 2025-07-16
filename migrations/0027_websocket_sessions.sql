--Add teacher_id column to websocket_sessions table
ALTER TABLE websocket_sessions
ADD COLUMN teacher_id INTEGER REFERENCES users(id);

--Add index for better performance when querying by teacher
CREATE INDEX idx_websocket_sessions_teacher_id ON websocket_sessions(teacher_id);

--ADD index for teacher and teacher_id queries
CREATE INDEX idx_websocket_sessions_teacher_test ON websocket_sessions(teacher_id, test_id);
