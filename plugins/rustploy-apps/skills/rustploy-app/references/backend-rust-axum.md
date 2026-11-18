# Backend: Rust + Axum + Postgres, em camadas

Ponto de partida: [`templates/backend/`](../templates/backend/). Copie para
`backend/` no repo do app, troque `app-backend` pelo nome do app (no
`Cargo.toml`, no `Dockerfile` e em `migrator/docker-compose.yml`) e gere o
`Cargo.lock` (`cargo generate-lockfile`; ele **é** versionado). O template compila,
aplica migrations, cria o admin e faz login (testado com Postgres 17).

## Estrutura

```text
backend/
├── Cargo.toml  Cargo.lock  Dockerfile  .dockerignore  .env.example
├── migrations/                  20260101000001_create_users.sql, …  (um arquivo por mudança)
├── migrator/docker-compose.yml  o job `migrate` do rustploy
└── src/
    ├── main.rs        subcomandos serve | migrate | migrate-and-seed
    ├── config.rs      leitura do ambiente (segredo ausente = não sobe)
    ├── state.rs       AppState: monta repositories e services uma vez
    ├── error.rs       AppError → status HTTP (detalhe interno só no log)
    ├── security.rs    argon2, JWT e o extrator AuthUser
    ├── seed.rs        usuário inicial idempotente
    ├── models/        entidades (FromRow) e DTOs (Serialize/Deserialize)
    ├── repositories/  SQL, e só SQL
    ├── services/      regra de negócio
    └── routes/        HTTP: extrai, chama o service, responde
```

## As camadas, e o que cada uma pode fazer

| Camada | Pode | Não pode |
|---|---|---|
| `routes` | ler path/query/JSON, exigir `AuthUser`, chamar **um** service, devolver JSON | SQL, regra de negócio, `PgPool` direto (exceto o `/health`) |
| `services` | validar, decidir, combinar repositories, transações de negócio, devolver `AppError` | conhecer HTTP (`StatusCode`, `Json`), escrever SQL |
| `repositories` | SQL parametrizado, mapear linhas para `models` | regra de negócio, saber quem é o usuário logado |
| `models` | structs e conversões (`From<User> for UserResponse`) | lógica |

Uma funcionalidade nova (ex.: "clientes") = `models/customer.rs` +
`repositories/customer_repository.rs` + `services/customer_service.rs` +
`routes/customers.rs` + uma migration + registro no `AppState` e no `router`.
Siga os nomes por extenso, em inglês, no código; mensagens para o usuário em português.

## Regras

- **SQL sempre parametrizado**: `sqlx::query_as::<_, T>("… WHERE id = $1").bind(id)`.
  Nunca `format!` com valor vindo de fora (só com constantes, como `COLUMNS`).
- **Transação** quando uma operação grava em mais de uma tabela:
  `let mut tx = pool.begin().await?; … tx.commit().await?;` (passe `&mut *tx` ao repository).
- **Erros**: handlers devolvem `AppResult<…>`. Nada de `unwrap()` em código de
  requisição. Mensagens do banco nunca vão para o cliente.
- **Autorização**: rota protegida recebe `user: AuthUser`; ação de admin chama
  `user.require_admin()?`. Rota pública é exceção, decidida conscientemente.
- **Senhas**: só `security::hash_password`/`verify_password` (argon2). Nunca logar.
- **Configuração**: tudo por `config.rs`. Segredo é `required` (sem valor padrão);
  só ajuste não sensível usa `optional`.
- **Todas as rotas sob `/api`**: o nginx do frontend repassa `/api/` inteiro.
- **`/api/health`** consulta o banco: é o healthcheck do rustploy.
- **Escutar em `0.0.0.0`** (`BIND_ADDRESS=0.0.0.0:8080`).
- **Listas paginadas** (`LIMIT $1 OFFSET $2`, teto de 100) desde o início.
- **Datas** em `TIMESTAMPTZ` (UTC); converta no frontend.

## Variáveis de ambiente do backend

| Variável | De onde vem no rustploy | Obrigatória |
|---|---|---|
| `DATABASE_URL` | projeto (gravada pelo `ManagedDatabaseCreate`) | sim |
| `JWT_SECRET` | secret do projeto | sim (32+ caracteres) |
| `ADMIN_EMAIL`, `ADMIN_NAME` | serviço `api` (Plain) | e-mail sim |
| `ADMIN_PASSWORD` | secret do projeto | sim no seed (12+ caracteres) |
| `BIND_ADDRESS`, `RUST_LOG`, `APP_ENV`, `JWT_TTL_HOURS` | serviço `api` (Plain) | não |

Variável nova? Acrescente em `config.rs`, no `.env.example` (valor falso), no
`README.md`, e no rustploy (`EnvValueSet` ou spec do serviço). Se for sensível,
peça o secret à pessoa. Se o job `migrate` também precisa dela, liste-a no
`environment:` do `migrator/docker-compose.yml`.

## Migrations

- Arquivo novo em `migrations/` com prefixo de data e hora crescente
  (`YYYYMMDDHHMMSS_descricao.sql`), ou `sqlx migrate add -r descricao` se tiver o
  `sqlx-cli`. **Nunca edite** uma migration que já foi para o servidor.
- Rodam em dois lugares, os dois idempotentes e com lock no banco: no job
  `migrate` (antes do deploy) e na subida do `serve` (rede de segurança).
- Dados de referência fixos e **não sensíveis** (lista de estados, tipos) podem ir
  numa migration. Usuários e qualquer coisa com senha, nunca: vão no seed.
- Mudança que apaga dados: avise a pessoa e prefira duas etapas.

## Seed

`seed.rs` cria o admin só se o e-mail ainda não existe (`ON CONFLICT DO NOTHING`)
e nunca altera usuário existente: trocar `ADMIN_PASSWORD` depois **não** muda a
senha de quem já existe (isso é de propósito). Para dados extras de seed, siga o
mesmo padrão: buscar pela chave natural, criar só se faltar.

## Rodar local

```bash
docker run -d --name app-pg -e POSTGRES_PASSWORD=app -e POSTGRES_USER=app -e POSTGRES_DB=app -p 5432:5432 postgres:17-alpine
cp .env.example .env            # e ajuste; o .env não é commitado
cargo run -- migrate-and-seed
cargo run                       # API em http://localhost:8080/api/health
```

## Build no servidor

O `Dockerfile` do template já resolve as armadilhas do rustploy-ms: construtor
clássico sem BuildKit (nada de `RUN --mount`), sem build args, cache de
dependências pelo esqueleto `fn main(){}`, imagem final slim e usuário sem
privilégio (uid 10001). Se acrescentar binários (`[[bin]]`), crie também o
esqueleto deles na primeira etapa, como o rdo-itemize faz.
