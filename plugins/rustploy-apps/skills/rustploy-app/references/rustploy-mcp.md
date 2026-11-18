# Operar o rustploy-ms pelo MCP

Servidor: **rustploy-ms.itemize.com.br**, API e MCP na porta **9797**, interface
web (webui) em `https://rustploy-ms.itemize.com.br:9797`. O servidor MCP se chama
`rustploy` e tem as ferramentas `status`, `list_services`, `list_deploys`, `deploy`,
`build_logs`, `ingress_routes`, `ingress_reconcile`, `rpc` e `manual`. Cada agente as
mostra com um prefixo próprio (no Claude Code, `mcp__rustploy__status`; no Codex, como
ferramentas do servidor `rustploy`). O `rpc` executa qualquer `Command` do daemon.

> A fonte da verdade dos comandos é a ferramenta `manual` do próprio MCP. Se um
> JSON daqui for recusado (`400` com a causa do serde), confie no manual e no
> formato de um objeto existente (`ServiceGet`, `JobListAll`), e avise que esta
> página precisa de atualização.

## Conectar

Se as ferramentas do servidor `rustploy` não existem na sessão, a pessoa precisa
conectar o agente ao servidor, uma vez por máquina. O token sai da webui:
`https://rustploy-ms.itemize.com.br:9797` → **Settings → Web Server → "Conectar um
agente (Claude Code)" → Copiar**. Os comandos vão **no terminal**, nunca no chat.

**Claude Code**: colar o comando copiado, que tem a forma
`claude mcp add --transport http rustploy https://…/mcp --header "Authorization: Bearer …"`,
e reiniciar o Claude Code.

**Codex**: o token fica numa variável de ambiente, e o Codex a lê:

```bash
# o token é o texto depois de "Bearer " no comando copiado
echo "export RUSTPLOY_TOKEN='<token>'" >> ~/.bashrc     # ou ~/.zshrc
source ~/.bashrc
codex mcp add rustploy --url https://rustploy-ms.itemize.com.br:9797/mcp --bearer-token-env-var RUSTPLOY_TOKEN
codex mcp list    # rustploy … enabled … Bearer token
```

Depois, abrir uma sessão nova do Codex.

O token é de administrador. **Nunca** o escreva em arquivo do repositório, em
`.mcp.json` ou `.codex/config.toml` versionados, em issue ou no chat. Se ele aparecer
no chat, avise a pessoa que o token precisa ser trocado.

## Codificação do `rpc`

- Comando sem campos: a string nua → `ProjectList`.
- Comando com campos: objeto de uma chave → `{"ServiceGet":{"id":"svc_…"}}`.
- Comando "tupla" leva o valor direto → `{"ServiceCreate":{…spec…}}`.
- Erro do daemon volta como `isError` com `code` e `message`.

## O que você vê e o que não vê

- Valores de env com nome sensível (`*PASSWORD*`, `*SECRET*`, `*TOKEN*`, `*KEY*`…)
  e senhas dentro de URLs chegam como `***`. É de propósito.
- Se você devolver um spec com `***` num `ServiceUpdate`/`ProjectEnvSet`, o daemon
  restaura o valor real. Não tente "consertar" o `***`.
- Se alguma resposta trouxer uma senha em claro (já aconteceu com
  `ManagedDatabaseListAll`, campo `connection_url`), **não a repita**, não a use
  em arquivo e avise a pessoa: é falha da plataforma e a senha deve ser trocada.

## Passo a passo de um app novo

Nos exemplos o app se chama `meuapp`. Troque pelo nome real.

### 1. Conferir o estado

```text
status                       → versão e fila
list_services                → o que já existe (não crie nome repetido)
rpc "ProjectList"            → ids dos projetos
rpc "GitProviderList"        → a conta GitHub conectada (provider_id gp_…)
```

### 2. Projeto

```json
{"ProjectCreate":{"name":"meuapp","description":"<uma linha sobre o app>"}}
```

Guarde o `prj_…` da resposta.

### 3. Database no Postgres compartilhado

O servidor compartilhado é o serviço `pg-central` do projeto `infra`. Ache o
`service_id` dele em `list_services` (hoje `svc_01M3VRSKJG09FKV1HH1TN7TSYF`,
mas confira).

```json
{"ManagedDatabaseCreate":{
  "server_service_id":"<svc do pg-central>",
  "project_id":"<prj do meuapp>",
  "name":"meuapp",
  "env_var":"DATABASE_URL"
}}
```

Isso cria o database e um usuário só dele, libera a rede entre o projeto e o
`pg-central` e grava `DATABASE_URL` nas variáveis **do projeto**. Nome do
database: `a-z`, `0-9`, `_`, até 32 caracteres (troque `-` por `_`).
Não use `overwrite_env` sem entender o que já existe.

### 4. Secrets

Quem sorteia é o **daemon**, com `SecretGenerate`. O valor nunca chega até você:

```json
{"SecretGenerate":{"project_id":"<prj>","name":"JWT_SECRET","kind":"Hex","on_exists":"Keep"}}
{"SecretGenerate":{"project_id":"<prj>","name":"ADMIN_PASSWORD","kind":"Password","length":20,"on_exists":"Keep"}}
```

Resposta: `{"SecretGenerated":{"name":"…","created":true,"length":20,"fingerprint":"a1b2c3d4"}}`.
`created: false` = já existia e foi mantido (repetir é seguro com `Keep`).

| Secret | `kind` | Para quê |
|---|---|---|
| `JWT_SECRET` | `Hex` (64) | assinar os tokens de login |
| `ADMIN_PASSWORD` | `Password`, `length` 20 | senha do primeiro usuário |
| outras chaves internas do app | `Hex` ou `Base64Url` | tokens de integração entre serviços do próprio app |

Regras:

- **Nunca** `on_exists: "Replace"` num secret que já está em uso sem perguntar: o app
  passa a ter outro valor no próximo deploy (`JWT_SECRET` novo desloga todo mundo;
  `ADMIN_PASSWORD` novo **não** muda a senha de quem já existe, porque o seed não
  sobrescreve).
- Chave de API **de terceiro** (gateway de pagamento, e-mail, Google…) não se sorteia:
  ela vem do painel do fornecedor. Peça para a pessoa criar o secret na webui
  (projeto → **Secrets**), com o nome que você indicar, e confira com `SecretList`.
  Nunca peça o valor no chat.
- Secret com nome errado vira **texto vazio sem erro**: o nome do `SecretGenerate` e o
  `{"Secret":"…"}` da variável têm de ser idênticos.
- Daemon antigo (antes do 0.1.66, release v0.1.79) responde `unknown variant SecretGenerate`:
  o rustploy-ms precisa ser atualizado. Até lá, peça à pessoa para criar os secrets na webui
  (valor de `JWT_SECRET`: `openssl rand -hex 32` no terminal dela) e avise quem cuida do
  servidor.

**Como a pessoa lê uma senha gerada** (por exemplo, para o primeiro login). Mensagem
pronta:

> A senha do primeiro acesso foi gerada pelo próprio rustploy e eu não tenho acesso a
> ela. Para ver: abra https://rustploy-ms.itemize.com.br:9797, entre no projeto
> **meuapp**, aba **Secrets**, e clique no 👁 da linha `ADMIN_PASSWORD`. Entre em
> https://meuapp.itemize.com.br com o e-mail `<e-mail>` e essa senha.

Confira só os nomes:

```json
{"SecretList":{"project_id":"<prj>"}}
```

### 5. Serviço `api` (backend)

Antes, leia um serviço existente como molde (`ServiceGet` de um backend Git que
esteja `Running`) e mantenha os campos que ele tem. Forma testada:

```json
{"ServiceCreate":{
  "project_id":"<prj>",
  "name":"api",
  "source":{"Git":{
    "url":"https://github.com/itemizebi/meuapp.git",
    "branch":"main",
    "provider_id":"<gp_… da conta GitHub>",
    "build_context":"backend",
    "dockerfile_path":"Dockerfile",
    "root_path":".",
    "build_stage":null, "credentials":null, "username":null,
    "submodules":false, "watch_paths":[]
  }},
  "port":8080,
  "replicas":1,
  "autostart":true,
  "domain":null, "domains":[], "tls_enabled":false, "host_port":null,
  "env_vars":[
    {"key":"BIND_ADDRESS","value":{"Plain":"0.0.0.0:8080"}},
    {"key":"RUST_LOG","value":{"Plain":"info,tower_http=info"}},
    {"key":"APP_ENV","value":{"Plain":"production"}},
    {"key":"JWT_SECRET","value":{"Secret":"JWT_SECRET"}},
    {"key":"ADMIN_EMAIL","value":{"Plain":"<e-mail combinado>"}},
    {"key":"ADMIN_NAME","value":{"Plain":"Administrador"}},
    {"key":"ADMIN_PASSWORD","value":{"Secret":"ADMIN_PASSWORD"}}
  ],
  "env_comments":[],
  "healthcheck":{"interval_secs":5,"timeout_secs":3,"retries":30,"start_period_secs":15,
                 "kind":{"Http":{"path":"/api/health","expected_status":200}}},
  "depends_on":[],
  "pre_deploy_job_id":null, "pre_deploy_job_ids":[],
  "product":null, "registry_auto_deploy":false,
  "resources":{"cpu_shares":0,"mem_limit_bytes":0},
  "run_args":[], "run_command":null, "volumes":[]
}}
```

- **O `api` não tem domínio.** Ele só é alcançado pela rede interna como
  `rp_api:8080` (Application = `rp_<nome do serviço>`).
- `DATABASE_URL` não entra aqui: vem do projeto.
- O rustploy **não injeta `PORT`** nem nada além dessa lista.

### 6. Job `migrate` (migrations + seed antes de cada deploy)

```json
{"JobCreate":{
  "project_id":"<prj>",
  "trigger_service_id":"<svc do api>",
  "name":"migrate",
  "compose":"",
  "git_source":{
    "provider_id":"<gp_…>",
    "url":"https://github.com/itemizebi/meuapp.git",
    "branch":"main",
    "compose_path":"backend/migrator/docker-compose.yml"
  },
  "main_service":"migrate",
  "env_vars":[],
  "recurrence":null
}}
```

O gatilho `api` faz o job herdar `DATABASE_URL` (projeto) e `ADMIN_*` (serviço).
Agora ligue o job ao deploy do `api`: `ServiceGet` do `api` → troque
`pre_deploy_job_ids` para `["<job_…>"]` → `ServiceUpdate{id, spec}` com o spec
**inteiro**. Rode uma vez à mão para validar e aquecer o cache:

```json
{"JobRunNow":{"id":"<job_…>"}}
{"JobRunHistory":{"job_id":"<job_…>","limit":1}}
{"GetJobLogs":{"job_run_id":"<jrun_…>"}}
```

### 7. Serviço `web` (frontend)

Igual ao `api`, com: `"name":"web"`, `build_context":"frontend"`, `"port":80`,
`env_vars` com `{"key":"API_UPSTREAM","value":{"Plain":"http://rp_api:8080"}}`,
healthcheck `Http` em `/healthz`, e o domínio:

```json
"domains":[{"domain":"meuapp.itemize.com.br","port":null,"tls":true}]
```

Coloque `"depends_on":["<svc do api>"]` para religar na ordem certa.
Crie o registro DNS **antes** deste deploy ([dns-hostinger.md](dns-hostinger.md)).

### 8. Deploy e verificação

```json
{"name":"deploy","arguments":{"service_id":"<svc do api>","wait":true}}
{"name":"deploy","arguments":{"service_id":"<svc do web>","wait":true}}
```

- `ok: true` = Live. `ok: false` → leia `error` e `log_tail`; mais log com
  `build_logs` (`after = log_cursor`). `ok: null` com `Stopped`/`Pruning` não é falha.
- Depois: `ingress_routes` deve listar o domínio com um backend `ip:80`.
- `curl -sI https://meuapp.itemize.com.br/` → 200; `curl -s https://meuapp.itemize.com.br/api/health` → `{"status":"ok"}`.
  O primeiro HTTPS pode falhar por alguns segundos enquanto o certificado sai.

## Mudanças do dia a dia

| Quero… | Faça |
|---|---|
| publicar código novo | `git push` → `deploy` do serviço (o job de pré-deploy roda sozinho) |
| mudar uma variável | `EnvValueSet{project_id, service_id?, key, value}` → **deploy** |
| trocar uma chave do app | `SecretGenerate` com `on_exists: Replace` (pergunte antes) → **deploy** dos serviços que usam |
| renomear variável sem ver o valor | `EnvKeyRename` / `SecretRename` |
| ver log do app | `LogsGet{service_id, tail:200}`; antes do último deploy: `LogsHistory` |
| voltar uma versão | `DeployRollback{service_id}` mostra o deploy anterior; confirme com a pessoa |
| novo domínio | DNS primeiro, depois `ServiceGet` → acrescenta em `domains` → `ServiceUpdate` → deploy |

## Proibido ao agente

`SecretReveal`, `SecretMove` com `SecretToEnv`, `ServiceExport` com valores,
`RegistryTokenCreate`, `RecoveryKeyGenerate`, `BackupDestinationSet`,
`ServiceBackupRestore`, `StateGitResolve`, `ApplyUpdate`. Não use `SecretSet` com
um valor que você inventou: para sortear, `SecretGenerate`.
