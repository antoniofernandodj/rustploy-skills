# rustploy-skills

Skills para agentes de código ([Claude Code](https://claude.com/claude-code) e
[Codex](https://developers.openai.com/codex)) usados por quem cria e publica apps no
rustploy da Itemize (`rustploy-ms.itemize.com.br`). O repositório é um **marketplace de
plugins** que os dois agentes leem: instalando uma vez, as atualizações chegam pelo
próprio agente.

| Plugin | Skill | Para quê |
|---|---|---|
| `rustploy-apps` | `rustploy-app` | criar um app (Rust + Axum + Postgres, Vue) e colocá-lo no ar com banco, migrations, usuário inicial, secrets, subdomínio e HTTPS |

## Claude Code

### Instalar

Dentro do Claude Code:

```text
/plugin marketplace add antoniofernandodj/rustploy-skills
/plugin install rustploy-apps@rustploy-skills
```

Ou pelo terminal:

```bash
claude plugin marketplace add antoniofernandodj/rustploy-skills
claude plugin install rustploy-apps@rustploy-skills
```

### Conectar ao servidor (uma vez por máquina)

Na webui do rustploy (`https://rustploy-ms.itemize.com.br:9797`), **Settings → Web Server →
"Conectar um agente (Claude Code)" → Copiar**, cole o comando no terminal e reinicie o Claude
Code. Ele tem a forma:

```bash
claude mcp add --transport http rustploy https://rustploy-ms.itemize.com.br:9797/mcp \
  --header "Authorization: Bearer <token>"
```

### Usar e atualizar

A skill entra sozinha quando o pedido for criar, publicar ou mexer num app ("cria um sistema
de cadastro de clientes e coloca no ar"). Para chamá-la à força: `/rustploy-apps:rustploy-app`.

```text
/plugin marketplace update rustploy-skills
```

Para receber sozinho: `/plugin` → **Marketplaces** → `rustploy-skills` → ligar a atualização
automática.

## Codex

Testado no Codex CLI 0.154.

### Instalar

```bash
codex plugin marketplace add antoniofernandodj/rustploy-skills
codex plugin add rustploy-apps@rustploy-skills
codex plugin list        # rustploy-apps@rustploy-skills  installed, enabled
```

(Também dá para instalar pelo navegador de plugins: `/plugins` dentro do Codex.)

### Conectar ao servidor (uma vez por máquina)

O Codex lê o token de uma variável de ambiente. Pegue o token na webui do rustploy
(`https://rustploy-ms.itemize.com.br:9797`), em **Settings → Web Server → "Conectar um agente
(Claude Code)" → Copiar**: é o texto depois de `Bearer ` no comando copiado. No terminal:

```bash
echo "export RUSTPLOY_TOKEN='<token>'" >> ~/.bashrc     # ou ~/.zshrc
source ~/.bashrc
codex mcp add rustploy --url https://rustploy-ms.itemize.com.br:9797/mcp --bearer-token-env-var RUSTPLOY_TOKEN
codex mcp list           # rustploy  …  enabled  Bearer token
```

Abra uma sessão nova do Codex depois de instalar o plugin e de conectar o MCP.

### Usar e atualizar

A skill entra sozinha pelos mesmos pedidos. Para chamá-la à força, digite `$` e escolha
`rustploy-apps:rustploy-app` (ou `/skills`).

```bash
codex plugin marketplace upgrade rustploy-skills
```

## O token é de administrador

Nos dois agentes, ele vai **só** na configuração feita no terminal. Nunca em arquivo de
repositório (`.mcp.json`, `.codex/config.toml`), issue ou chat. Vazou? Peça para trocar.

## Manter

```text
plugins/rustploy-apps/
├── .claude-plugin/plugin.json       nome e versão do plugin
└── skills/rustploy-app/
    ├── SKILL.md                     regras e o caminho do zero ao ar (curto de propósito)
    ├── references/                  lido sob demanda: MCP, DNS, backend, frontend, erros
    └── templates/                   código inicial testado (backend, frontend, .gitignore, README)
```

1. Mude o que precisa. O `SKILL.md` fica curto; detalhe vai em `references/`.
2. Erro novo que aconteceu de verdade? Uma linha em `references/erros-comuns.md`.
3. Mexeu em template? Compile e teste (`cargo build`, `npm run build`,
   `DOCKER_BUILDKIT=0 docker build`).
4. Suba a versão em `plugin.json` **e** em `.claude-plugin/marketplace.json`, e anote no
   `CHANGELOG.md`.
5. Commit e push aqui e, no agregador `rustploy`, commit do novo ponteiro do submódulo.

Skill nova (por exemplo, para outro tipo de app)? Uma pasta nova em
`plugins/rustploy-apps/skills/<nome>/SKILL.md`, ou um plugin novo listado no
`marketplace.json`.

O texto da skill vale para os dois agentes: não cite nomes de ferramenta de um só (`mcp__…`),
e sim as ferramentas do servidor `rustploy` (`status`, `deploy`, `rpc`…). O Codex lê o mesmo
`.claude-plugin/marketplace.json` e o `plugin.json` daqui; depois de mudar, confira nos dois
(`claude plugin validate .` e `codex plugin marketplace add ./` num `CODEX_HOME` temporário).

Nada aqui pode conter segredo: o repositório é lido por qualquer pessoa que instale a skill.
