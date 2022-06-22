ALTER TABLE pending_projects ADD COLUMN exceptional_complete_deadline timestamptz CHECK (current_timestamp < exceptional_complete_deadline);
