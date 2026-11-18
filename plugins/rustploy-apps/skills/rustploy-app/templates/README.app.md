# <Nome do app>

<Uma frase: o que o sistema faz e para quem.>

- Produção: https://<app>.itemize.com.br
- Servidor: rustploy-ms (projeto `<app>`: serviços `api` e `web`, job `migrate`)
- Banco: database `<app>` no Postgres compartilhado `pg-central`

## Estrutura

- `backend/`: Rust + Axum + SQLx (routes → services → repositories), migrations em `backend/migrations/`
- `frontend/`: Vue 3 + Vite, servido por nginx, que repassa `/api/` ao backend

## Rodar local

```bash
docker run -d --name <app>-pg -e POSTGRES_PASSWORD=app -e POSTGRES_USER=app -e POSTGRES_DB=app -p 5432:5432 postgres:17-alpine
cd backend && cp .env.example .env && cargo run -- migrate-and-seed && cargo run
cd frontend && npm install && npm run dev      # http://localhost:5173
```

## Configuração (produção)

| Variável | Onde | Observação |
|---|---|---|
| `DATABASE_URL` | projeto (automática) | criada com o database |
| `JWT_SECRET` | secret do projeto | 32+ caracteres |
| `ADMIN_PASSWORD` | secret do projeto | senha do primeiro usuário |
| `ADMIN_EMAIL` | serviço `api` | e-mail do primeiro usuário |
| `API_UPSTREAM` | serviço `web` | `http://rp_api:8080` |

Nenhum segredo fica neste repositório.

## Publicar

`git push` na `main` e deploy do serviço no rustploy (o job `migrate` roda antes,
sozinho). Mudou variável ou secret? Novo deploy.
