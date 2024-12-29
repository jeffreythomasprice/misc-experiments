CREATE TABLE messages(
	id SERIAL PRIMARY KEY,
	timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
	sender CHAR(256) NOT NULL,
	message TEXT NOT NULL
);

CREATE OR REPLACE FUNCTION messages_trigger_function() RETURNS trigger AS $$
BEGIN
	PERFORM pg_notify(
		'table_update',
		json_build_object(
			'table', TG_TABLE_NAME,
			'op', TG_OP,
			'old_id', OLD.id,
			'new_id', NEW.id
		)::text
	);
	RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS messages_notify ON messages;
CREATE TRIGGER messages_notify AFTER INSERT OR UPDATE OR DELETE ON messages FOR EACH ROW EXECUTE PROCEDURE messages_trigger_function();
