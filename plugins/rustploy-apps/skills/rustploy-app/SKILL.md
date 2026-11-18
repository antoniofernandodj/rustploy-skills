---
name: rustploy-app
description: Cria, organiza e publica apps web da Itemize no servidor rustploy-ms (rustploy-ms.itemize.com.br) pelo MCP do rustploy, com backend Rust + Axum + Postgres (camadas repository e service), frontend Vue separado, banco provisionado pela plataforma, migrations e seed por job, segredos em secrets e subdomínio na Hostinger com HTTPS. Use sempre que o pedido for criar um app ou sistema novo, colocar um app no ar, publicar, fazer deploy, criar banco, usuário inicial, login, subdomínio ou domínio, configurar variáveis ou senhas, ou mexer num app que já roda no rustploy, mesmo que a pessoa não diga "rustploy".
---

# Apps da Itemize no rustploy-ms

Quem pede costuma **não ser desenvolvedor**. Você é o engenheiro responsável: decida
o técnico sozinho, seguindo esta skill, e só pergunte o que é **decisão de negócio**
(nome do app, subdomínio, quem é o primeiro usuário, o que o sistema faz). Senhas,
chaves e configuração técnica são com você: crie tudo sozinho. Explique
em português simples o que está fazendo e por quê, sem jargão desnecessário.

O pedido raramente vai citar banco, seed, secrets, healthcheck ou DNS. **Faça
mesmo assim**: um app que sobe sem usuário inicial, com senha no git ou sem HTTPS
está quebrado, mesmo que "funcione".

## Regras que não se quebram

1. **Nenhum segredo no repositório.** Senha, token, chave JWT, URL de banco com
   senha, chave de API: nada disso vai para código, `Dockerfile`, `docker-compose`,
   migration, seed, `README` ou commit. Tudo vem de **variável de ambiente**, e o
   valor sensível mora num **secret** do rustploy. Antes de **todo** commit, rode a
   checagem de [references/erros-comuns.md](references/erros-comuns.md#checagem-antes-do-commit).
2. **Senha quem sorteia é o rustploy, nunca você.** Toda senha ou chave nasce com
   `SecretGenerate` (o daemon sorteia e grava cifrado; você recebe só o tamanho e uma
   impressão digital). Nunca invente um valor para o `SecretSet`, nunca escreva senha em
   arquivo, nunca imprima senha no chat e nunca peça para a pessoa colar uma senha na
   conversa. Quando a pessoa precisa saber uma senha (o primeiro login), diga onde ler:
   webui → projeto → **Secrets** → 👁 (passo a passo em
   [references/rustploy-mcp.md](references/rustploy-mcp.md#secrets)).
3. **Todo app com login nasce com usuário inicial**, criado por um **seed
   idempotente** que lê e-mail e senha do ambiente. Nunca usuário/senha fixos no
   código, nunca "admin/admin", nunca uma migration com hash de senha.
4. **O schema muda só por migration versionada** (`sqlx migrate`), rodada por um
   **job de pré-deploy**. Nunca altere tabela à mão no banco de produção.
5. **Banco é da plataforma.** Use o Postgres compartilhado `pg-central` (projeto
   `infra`) com um database próprio do app. Não suba Postgres dentro da imagem do
   app, não use SQLite em produção, não exponha porta de banco à internet.
6. **Confirme antes de destruir.** `ServiceDelete`, `ProjectDelete`,
   `ManagedDatabaseDelete`, `Prune*`, `Remove*`, `StopAllManaged`, `DeployAbort`
   e qualquer mudança em app de **outra pessoa**: pergunte antes, dizendo o que se
   perde. Outros projetos do servidor não são seus; não mexa neles.
7. **Só diga que está no ar depois de testar a URL pública** (`curl` no HTTPS do
   frontend e do `/api/health`). Deploy `Live` não é prova de que o login funciona.

## Pilha fixa (não troque sem pedido explícito)

| Parte | Escolha |
|---|---|
| Backend | Rust estável, **Axum**, **SQLx** (Postgres, `migrate`), Tokio, `tracing`, `argon2` para senha, JWT |
| Arquitetura do backend | `routes` (HTTP) → `services` (regra de negócio) → `repositories` (SQL). Detalhes: [references/backend-rust-axum.md](references/backend-rust-axum.md) |
| Frontend | **Vue 3 + Vite + TypeScript**, Vue Router, Pinia, servido por **nginx** num serviço separado. Detalhes: [references/frontend-vue.md](references/frontend-vue.md) |
| Banco | Postgres `pg-central`, database próprio por app |
| Ligação front → back | o nginx do frontend repassa `/api/` ao backend **pela rede interna** do projeto; o navegador nunca chama outro domínio (sem CORS, sem segundo certificado) |
| Repositório | um repo GitHub na organização `itemizebi`, com pastas `backend/` e `frontend/` |

## O caminho, do zero ao ar

Siga na ordem e marque cada item numa lista de tarefas. Detalhes e JSON exato de
cada chamada em [references/rustploy-mcp.md](references/rustploy-mcp.md).

1. **Conferir o MCP.** Chame a ferramenta `status` do servidor `rustploy`. Ela
   tem de responder do **rustploy-ms.itemize.com.br**. Sem a ferramenta, pare e
   oriente a conexão (seção "Conectar" do rustploy-mcp.md). Leia o `manual` do
   MCP uma vez por sessão.
2. **Combinar o essencial com a pessoa**, em uma pergunta só: nome curto do app
   (vira projeto, database e repo: minúsculas, `-`), o **subdomínio** (padrão
   `<app>.itemize.com.br`), o **e-mail do primeiro usuário** e o que o sistema
   precisa fazer. O resto você decide.
3. **Criar o código** a partir de [templates/](templates/) (copie e adapte, não
   reescreva do zero): backend com `/api/health`, migrations, seed do admin;
   frontend com tela de login; `templates/gitignore` vira o `.gitignore` da raiz; `README.app.md` vira o `README.md`.
   Compile e rode os testes localmente antes de subir (`cargo build`,
   `cargo test`, `npm run build`).
4. **Checagem de segredos** e commit/push para o GitHub (repo **privado**).
5. **Projeto e banco no rustploy**: `ProjectCreate`; `ManagedDatabaseCreate` no
   `pg-central` (grava `DATABASE_URL` no projeto sozinho).
6. **Secrets**: `SecretGenerate` de `JWT_SECRET` (`Hex`) e `ADMIN_PASSWORD`
   (`Password`, 20) no projeto, com `on_exists: Keep`. Confira com `SecretList`.
7. **Serviços**: `api` (Git, `backend/`, porta 8080, healthcheck `/api/health`,
   envs referenciando secrets) e `web` (Git, `frontend/`, porta 80, domínio com
   TLS). Copie a forma do spec de um serviço existente com `ServiceGet`.
8. **Job `migrate`** (fonte Git, `backend/migrator/docker-compose.yml`), gatilho
   `api`, e coloque-o em `pre_deploy_job_ids` do `api`. Ele roda migrations **e**
   o seed antes de cada deploy.
9. **Subdomínio**: se o nome ainda não resolve para `179.198.122.159`, a pessoa
   cria o registro A na Hostinger ([references/dns-hostinger.md](references/dns-hostinger.md)).
   Espere resolver **antes** do deploy do `web`, senão o certificado não sai.
10. **Deploy** com a ferramenta `deploy` (`wait: true`): primeiro `api`, depois
    `web`. Falhou? Leia o `error` e o `build_logs`, corrija, repita.
11. **Verificar**: `curl -I https://<sub>/`, `curl https://<sub>/api/health`, e peça à
    pessoa para entrar com o primeiro usuário (ela lê a senha no 👁 de `ADMIN_PASSWORD`
    e digita no navegador; você não vê).
12. **Entregar**: URL, e-mail do primeiro usuário, onde ficam as senhas (secrets
    do projeto no rustploy), e um `README.md` no repo com como rodar local e como
    publicar. Recomende trocar a senha inicial no primeiro acesso.

## App que já existe

Antes de mudar qualquer coisa: `list_services`, `ServiceGet` do serviço e o
`README.md` do repo. Mudou o banco → **nova** migration (nunca edite uma que já
rodou). Mudou variável ou secret → **deploy** de novo (`ServiceReload` não relê
variáveis). Mudou só código → `git push` e `deploy`. O `ServiceUpdate` substitui o
spec inteiro: sempre `ServiceGet` → altera um campo → devolve tudo.

## Quando algo dá errado

Antes de tentar de novo, consulte [references/erros-comuns.md](references/erros-comuns.md):
lá estão os erros que já aconteceram neste servidor, com a causa e a correção
(502, certificado que não sai, variável vazia, build quebrando, migrations).
Não "conserte" desligando segurança (TLS, validação de token, senha fraca, CORS
aberto com `*`).

## Manter esta skill

Se você descobrir um erro novo, um comando que mudou ou um passo que faltou,
diga isso à pessoa no fim e sugira registrar no repositório da skill
(`antoniofernandodj/rustploy-skills`, submódulo `rustploy-skills/` do agregador
`rustploy`): na seção correspondente e no `CHANGELOG.md`.
