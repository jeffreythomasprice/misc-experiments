# Database

Common to everything else in this section:
```
cd server
```

Init config files:
```
diesel setup
```

Create a migration:
```
diesel migration generate create_users
```

View status:
```
diesel migration list
```

Run all migrations:
```
diesel migration run
```

Tear down migrations:
```
diesel migration revert
```

# Server

Common to everything else in this section:
```
cd server
```

```
cargo watch -x run
```

# Client

Common to everything else in this section:
```
cd client
```

```
trunk serve
```