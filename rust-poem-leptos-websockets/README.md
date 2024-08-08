```
cd client
npx tailwindcss -i ./index.css -o ./generated/index.css
trunk serve
```

```
cd server
docker compose up -d
cargo watch -x run
```

```
diesel migration run
# diesel migration redo
```

```
docker exec -it server-db-1 psql -U user -W experiment
# password
```
