-- Add migration script here
ALTER TABLE forms ADD COLUMN answer_notification_webhook varchar(100);
