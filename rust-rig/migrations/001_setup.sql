CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE documents (
    id uuid DEFAULT gen_random_uuid(),
    "key" text NOT NULL,
    first_page int NOT NULL,
    last_page int NOT NULL,
    "text" text NOT NULL,
    -- text-embedding-3-small = 1536 dimensions
    embedding vector(1536)
);

CREATE INDEX IF NOT EXISTS document_embeddings_idx ON documents 
USING hnsw(embedding vector_cosine_ops);