```
sudo apt install pdftk-java
```

Create `.env` and fill in API key:
```
PG_HOST=127.0.0.1
PG_PORT=5432
PG_USERNAME=postgres
PG_PASSWORD=password
PG_DATABASE=experiment
OPENAI_API_KEY=...
```

```
docker compose up -d
```

```
docker exec -it pgvector-db psql -U postgres -d experiment
\dx
\dt
\d document_chunk
select id, key, first_page, last_page from document_chunk;
```

```
# or just let systemctl do it
ollama serve

# pull all the models we're going to use
ollama pull ...
```

```
journalctl -u ollama --no-pager --follow --pager-end
```

```
cargo run
```