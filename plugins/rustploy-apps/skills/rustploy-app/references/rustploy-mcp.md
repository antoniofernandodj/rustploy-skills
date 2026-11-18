# Operar o rustploy-ms pelo MCP

Servidor: **rustploy-ms.itemize.com.br**, API e MCP na porta **9797**, interface
web (webui) em `https://rustploy-ms.itemize.com.br:9797`. As ferramentas aparecem
como `mcp__rustploy__*` (`status`, `list_services`, `deploy`, `build_logs`,
`ingress_routes`, `rpc`…). O `rpc` executa qualquer `Command` do daemon.

> A fonte da verdade dos comandos é a ferramenta `manual` do próprio MCP. Se um
> JSON daqui for recusado (`400` com a causa do serde), confie no manual e no
> formato de um objeto existente (`ServiceGet`, `JobListAll`), e avise que esta
> página precisa de atualização.

## Conectar

Se as ferramentas `mcp__rustploy__*` não existem na sessão, a pessoa precisa
conectar o Claude Code ao servidor. Oriente:

1. Abrir `https://rustploy-ms.itemize.com.br:9797` e entrar.
2. **Settings → Web Server → "Conectar um agente (Claude Code)" → Copiar**.
3. Colar o comando copiado **no terminal** (não no chat). Ele tem a forma
   `claude mcp add --transport http rustploy https://…/mcp --header "Authorization: Bearer …"`.
4. Reiniciar o Claude Code.

O token é de administrador. **Nunca** o escreva em arquivo do repositório, em
`.mcp.json` versionado, em issue ou no chat. Se ele aparecer no chat, avise a
pessoa que o token precisa ser trocado.

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

Secrets que todo app com login precisa (no **projeto**):

| Secret | Para quê | Como a pessoa gera |
|---|---|---|
| `JWT_SECRET` | assinar os tokens de login | 64+ caracteres aleatórios: `openssl rand -base64 48` no terminal dela |
| `ADMIN_PASSWORD` | senha do primeiro usuário | ela escolhe uma senha forte (12+ caracteres) |

Mensagem para a pessoa (adapte):

> Preciso de duas senhas guardadas no cofre do rustploy. Eu não posso ver nem
> escolher essas senhas. Abra https://rustploy-ms.itemize.com.br:9797, entre no
> projeto **meuapp**, aba **Secrets**, e crie:
> 1. nome `JWT_SECRET`, valor: o resultado de `openssl rand -base64 48`
>    (rode no seu terminal e cole o resultado);
> 2. nome `ADMIN_PASSWORD`, valor: a senha que você quer usar no primeiro login.
> Não cole esses valores aqui no chat. Me avise quando terminar.

Depois confira só os nomes:

```json
{"SecretList":{"project_id":"<prj>"}}
```

Secret com nome errado vira **texto vazio sem erro**. Confira letra por letra.

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
| trocar uma senha | a pessoa troca o secret na webui → **deploy** dos serviços que usam |
| renomear variável sem ver o valor | `EnvKeyRename` / `SecretRename` |
| ver log do app | `LogsGet{service_id, tail:200}`; antes do último deploy: `LogsHistory` |
| voltar uma versão | `DeployRollback{service_id}` mostra o deploy anterior; confirme com a pessoa |
| novo domínio | DNS primeiro, depois `ServiceGet` → acrescenta em `domains` → `ServiceUpdate` → deploy |

## Proibido ao agente

`SecretReveal`, `SecretMove` com `SecretToEnv`, `ServiceExport` com valores,
`RegistryTokenCreate`, `RecoveryKeyGenerate`, `BackupDestinationSet`,
`ServiceBackupRestore`, `StateGitResolve`, `ApplyUpdate`. Não use `SecretSet` com
um valor que você inventou: senha é coisa da pessoa.
