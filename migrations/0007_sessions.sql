CREATE TABLE IF NOT EXISTS test_sessions (
  session_id VARCHAR(36) PRIMARY KEY,
  test_id UUID NOT NULL,
  student_id INTEGER NOT NULL,
  evaluator_id VARCHAR(36) NOT NULL,
  current_card_index INTEGER NOT NULL DEFAULT 0,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  completed_at TIMESTAMP WITH TIME ZONE,
  FOREIGN KEY (test_id) REFERENCES tests(test_id),
  FOREIGN KEY (student_id) REFERENCES students(student_id)
);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_test_session_updated_at
BEFORE UPDATE ON test_sessions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();


CREATE INDEX idx_test_sessions_test_id ON test_sessions(test_id);
CREATE INDEX idx_test_sessions_student_id ON test_sessions(student_id);
CREATE INDEX idx_test_sessions_active ON test_sessions(is_active);
