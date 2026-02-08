```
# .env
OPENAI_API_KEY='...'
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/experiment
```

```
docker compose up -d
```

```
docker exec -it rust-rig-postgres-1 psql -U postgres -d experiment
\dx
\dt
\d documents
select count(*) from documents;
select id, path, key, first_page, last_page from documents;
```

```
cargo run -- insert-document --path ~/scratch/games/source_material/free_or_stolen/World\ of\ Darkness\ \(Classic\)/v20\ Vampire\ The\ Masquerade\ -\ 20th\ Anniversary\ Edition.pdf --chunk-page-count 3
```

```
cargo run -- search-documents --query "thaumaturgy 4 dot power rules" --count 5
```

```
cargo run -- chat
```

TODO if this works delete rust-llm, that's just a different shittier way of doing this?