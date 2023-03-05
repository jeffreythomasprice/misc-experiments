CREATE TABLE config(
	key TEXT NOT NULL UNIQUE,
	value TEXT NOT NULL,
	created TEXT NOT NULL,
	updated TEXT NOT NULL
);

CREATE TABLE users(
	name TEXT NOT NULL UNIQUE,
	password TEXT NOT NULL,
	is_admin INTEGER
);

-- default admin user
INSERT INTO users (name, password, is_admin) VALUES ("admin", "admin", 1);
