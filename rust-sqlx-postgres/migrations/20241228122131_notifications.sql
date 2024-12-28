CREATE TABLE notifications(
	id SERIAL PRIMARY KEY,
	message TEXT
);

CREATE OR REPLACE FUNCTION notifcations_trigger_function() RETURNS trigger AS $$
BEGIN
	PERFORM pg_notify(
		'table_update',
		json_build_object(
			'table', TG_TABLE_NAME,
			'op', TG_OP,
			'old', OLD,
			'new', NEW
		)::text
	);
	RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- TODO should be able to do AFTER UPDATE OR INSERT OR DELETE

DROP TRIGGER IF EXISTS notifications_notify_update ON notifications;
CREATE TRIGGER notifications_notify_update AFTER UPDATE ON notifications FOR EACH ROW EXECUTE PROCEDURE notifcations_trigger_function();

DROP TRIGGER IF EXISTS notifications_notify_insert ON notifications;
CREATE TRIGGER notifications_notify_insert AFTER INSERT ON notifications FOR EACH ROW EXECUTE PROCEDURE notifcations_trigger_function();

DROP TRIGGER IF EXISTS notifications_notify_delete ON notifications;
CREATE TRIGGER notifications_notify_delete AFTER DELETE ON notifications FOR EACH ROW EXECUTE PROCEDURE notifcations_trigger_function();
