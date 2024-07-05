CREATE TABLE users (
	username CHAR(256) PRIMARY KEY NOT NULL,
	password CHAR(256) NOT NULL
);

INSERT INTO users (username, password) VALUES ("admin", "admin");
