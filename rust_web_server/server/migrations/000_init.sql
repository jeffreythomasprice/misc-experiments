CREATE TABLE users(
	name TEXT NOT NULL UNIQUE,
	password TEXT NOT NULL,
	is_admin INTEGER
);

-- sample data
INSERT INTO users (name, password, is_admin) VALUES ("admin", "admin", 1);
