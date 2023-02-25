CREATE TABLE users(
	name TEXT NOT NULL UNIQUE,
	password TEXT NOT NULL,
	is_admin INTEGER
);
