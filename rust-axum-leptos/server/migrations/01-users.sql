CREATE TABLE users (
	username STRING PRIMARY KEY NOT NULL,
	password STRING NOT NULL,
	is_admin BOOL NOT NULL
);

INSERT INTO users (username, password, is_admin) VALUES ("admin", "admin", true);
