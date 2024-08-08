```
cd client
./watch.sh
```

```
cd server
docker compose up -d
./watch.sh
```

```
diesel migration run
# diesel migration redo
```

```
docker exec -it server-db-1 psql -U user -W experiment
# password
```
