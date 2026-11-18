# Erros que acontecem (e como evitar)

Cada linha é um erro que alguém que **não é desenvolvedor**, guiado por um agente,
tende a cometer, ou que já aconteceu neste servidor. Antes de dar uma tarefa por
concluída, passe pela lista da sua área.

## Checagem antes do commit

Rode da raiz do repo, **antes de todo commit**. Qualquer saída é motivo para parar.

```bash
# 1. arquivos que nunca podem ser versionados
git ls-files | grep -Ei '(^|/)\.env($|\.)|\.pem$|\.key$|id_rsa|credentials|\.sqlite3?$|\.db$' | grep -v '\.env\.example$'

# 2. segredos escritos no código (inclui o que está em stage)
git grep -nIE '(password|passwd|senha|secret|token|api[_-]?key)\s*[:=]\s*["'"'"'][^"'"'"'$<{]{6,}' -- . ':!*.lock' ':!*.md'
git grep -nIE 'postgres(ql)?://[^:/ ]+:[^@/ $<{]+@' -- . ':!*.md'
git grep -nIE 'BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY|ghp_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}'

# 3. o .gitignore cobre o básico
grep -qxF '.env' .gitignore && grep -qE '^(/)?target/?$|backend/target' .gitignore && grep -q node_modules .gitignore || echo "FALTA .gitignore"
```

Se `gitleaks` estiver instalado, rode também `gitleaks detect --no-banner`.

Um segredo **já commitado** não some com outro commit: ele está no histórico e
já foi para o GitHub. Avise a pessoa, peça para **trocar** o valor (novo secret) e
só então limpe o histórico, se ela quiser.

## Segredos e configuração

| Erro | Por que é ruim | Como fazer certo |
|---|---|---|
| senha, token ou `DATABASE_URL` com senha no código, `Dockerfile`, `docker-compose`, `README` | qualquer um com acesso ao repo (ou a um fork, ou à imagem) lê | variável de ambiente; valor sensível em **secret** do rustploy |
| `.env` commitado | o mesmo | `.env` no `.gitignore`; versionar só `.env.example` com valores falsos |
| valor padrão de segredo no código (`unwrap_or("dev-secret")`) | em produção, sem a variável, o app sobe com a senha pública | segredo **obrigatório**: sem a variável, o app **não sobe** e diz qual falta |
| `ARG`/`ENV` com segredo no `Dockerfile` | fica gravado na imagem | o rustploy nem passa variáveis no build; leia em tempo de execução |
| `VITE_API_KEY` ou qualquer segredo no frontend | tudo que vai para o navegador é público | segredo só no backend; o frontend chama o backend |
| agente escolhe a senha e grava com `SecretSet`, ou pede para colar a senha no chat | a senha passou pelo agente e fica no histórico da conversa | a pessoa cria o secret na webui; o agente só usa o **nome** |
| nome do secret digitado diferente na variável (`JWT_SECRT`) | a variável chega **vazia, sem erro** | conferir com `SecretList`; o app deve recusar subir sem a variável |
| mudar variável/secret e só dar `Reload` | o container antigo mantém o ambiente velho | **deploy** depois de mudar variável ou secret |
| `ServiceUpdate` com só o campo novo | apaga o resto do spec | `ServiceGet` → muda um campo → manda o spec inteiro |
| token do MCP salvo em arquivo do repo ou mostrado no chat | é token de administrador do servidor inteiro | só no `claude mcp add` no terminal; vazou → trocar |

## Banco, migrations e seed

| Erro | Consequência | Como fazer certo |
|---|---|---|
| app sem usuário inicial | ninguém consegue entrar; alguém cria usuário à mão no banco | seed idempotente do admin, lendo `ADMIN_EMAIL`/`ADMIN_PASSWORD` |
| admin criado numa migration, com hash fixo | a senha fica no git para sempre e é igual em todo ambiente | seed separado, senha do ambiente, hash `argon2` na hora |
| seed que duplica a cada deploy, ou que **reseta a senha** do admin | dados repetidos; o admin perde a senha que trocou | `INSERT … ON CONFLICT (email) DO NOTHING`; só cria se não existe |
| `CREATE TABLE` à mão no banco de produção | o código e o banco divergem; o próximo ambiente quebra | toda mudança é um arquivo novo em `backend/migrations/` |
| editar uma migration que já rodou | o `sqlx` recusa (checksum) e o deploy para | criar **outra** migration que corrige |
| migration destrutiva (`DROP COLUMN`, `DROP TABLE`) sem aviso | perda de dados | perguntar antes; preferir em duas etapas (deixar de usar, depois remover) |
| job de migration criado mas fora do `pre_deploy_job_ids` | o código novo sobe antes das tabelas | ligar o job ao `api` em `pre_deploy_job_ids` |
| Postgres dentro do container do app, ou SQLite | dado some a cada deploy; sem backup | database no `pg-central` (`ManagedDatabaseCreate`) |
| porta do banco exposta (`tcp_ports`) para "testar de casa" | banco aberto para a internet | não expor; para inspecionar, um job com `psql` ou acesso que a pessoa autorize |
| usar o superusuário `postgres` no app | o app pode apagar qualquer banco do servidor | o usuário próprio que o `ManagedDatabaseCreate` cria |
| SQL montado com `format!` e dado do usuário | SQL injection | sempre parâmetros (`$1, $2` + `.bind(...)`), nunca concatenar |

## Backend

| Erro | Como fazer certo |
|---|---|
| SQL dentro do handler HTTP | handler → service → repository; SQL só no repository |
| `unwrap()`/`expect()` em código de requisição | erro tipado (`AppError`) que vira status HTTP; `expect` só na subida |
| senha guardada em texto, MD5 ou SHA | `argon2` (hash + verificação) |
| JWT sem expiração, ou segredo curto | expiração (ex.: 8 h); `JWT_SECRET` com 32+ bytes, obrigatório |
| CORS `*` com credenciais | não precisa de CORS: o frontend chama `/api` na mesma origem |
| erro interno devolvido ao cliente (mensagem do banco, stack) | log detalhado no servidor; cliente recebe mensagem genérica |
| escutar em `127.0.0.1` | o container precisa escutar em `0.0.0.0:<porta>` |
| sem `/api/health` | o deploy não sabe se subiu; o healthcheck depende dele |
| log de senha, token ou corpo de login | nunca logar segredos nem `Authorization` |
| rota sem checar permissão | middleware de autenticação em tudo que não for público |

## Docker e build

| Erro | Como fazer certo |
|---|---|
| `RUN --mount=type=cache` no `Dockerfile` | o servidor usa o construtor clássico, **sem BuildKit**: isso falha. Use o truque do esqueleto (template) |
| macros `sqlx::query!` no build tentando conectar ao banco | o template usa consultas com `.bind()` (não precisa de banco no build). Se adotar as macros: `SQLX_OFFLINE=true` e `.sqlx/` versionada (`cargo sqlx prepare`) |
| `ARG` esperando variável do rustploy no build | não há build args em serviço Git; tudo em tempo de execução |
| imagem final `rust:` gigante, rodando como root | multi-stage, final `debian:bookworm-slim`, usuário sem privilégio |
| sem `.dockerignore` | `target/`, `node_modules/` e `.env` vão para o contexto de build |
| `Dockerfile` fora do `build_context` | caminho do Dockerfile é relativo ao contexto (`backend/Dockerfile`) |

## Frontend

| Erro | Como fazer certo |
|---|---|
| URL da API fixa (`http://localhost:8080`, `https://x-api…`) no código | chamar **`/api/…`** relativo; o nginx repassa |
| `VITE_API_URL` esperando valor do rustploy no build | não há variável no build; o relativo resolve |
| nginx com `resolver 127.0.0.11` | esse DNS é do Docker; este servidor usa **Podman** e a chamada pendura até dar timeout (aconteceu no rdo). Use `NGINX_ENTRYPOINT_LOCAL_RESOLVERS` (template) |
| `proxy_pass http://rp_api:8080;` sem variável | o nginx guarda o IP do container antigo e dá 502 depois de cada deploy do backend |
| refresh numa rota do Vue dá 404 | `try_files $uri $uri/ /index.html;` |
| token guardado em `localStorage` sem expiração | aceitável com expiração curta; nunca guardar a senha |

## Rustploy e publicação

| Erro | Como fazer certo |
|---|---|
| deploy do frontend antes de o DNS apontar | criar o registro, esperar resolver, depois deploy (senão o certificado não sai) |
| domínio no `api` e no `web` sem necessidade | só o `web` tem domínio; `api` é interno |
| `tls: false` em domínio público | `tls: true` |
| criar serviço/projeto com nome já usado | conferir `list_services` antes |
| mexer em projeto de outra pessoa ("aproveitando") | só o projeto do app em questão; o resto, perguntar |
| apagar algo para "começar de novo" | perguntar; `ManagedDatabaseDelete` apaga os dados sem volta |
| dar por pronto com o deploy `Live` | testar a URL pública e o login |
| repo GitHub público | repositório **privado** na organização `itemizebi` |
| branch errada no serviço | o serviço clona só a branch configurada (`main` por padrão) |

## Diagnóstico rápido

| Sintoma | Onde olhar |
|---|---|
| `deploy` com `ok:false` | `error` da resposta; `build_logs` com o `deployment_id` |
| healthcheck falhou | `LogsHistory` do serviço: o app caiu na subida? falta variável? |
| `502` | `ingress_routes`: domínio ausente (deploy não promoveu), `backends` vazio, ou porta errada. Depois `ingress_reconcile` |
| `/api` pendura ou dá 502 e o site abre | o nginx do `web` não alcança `rp_api`: nome do serviço, porta, resolver |
| login falha só em produção | `JWT_SECRET`/`ADMIN_*` vazios (secret com nome errado), seed não rodou (`JobRunHistory` do `migrate`) |
| job `migrate` falhou | `GetJobLogs`; `DATABASE_URL` chegou? migration editada? |
