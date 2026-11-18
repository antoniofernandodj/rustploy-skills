# rustploy-skills

Skills do [Claude Code](https://claude.com/claude-code) para quem cria e publica apps
no rustploy da Itemize (`rustploy-ms.itemize.com.br`). O repositório é um
**marketplace de plugins**: instalando uma vez, as atualizações chegam pelo próprio
Claude Code.

| Plugin | Skill | Para quê |
|---|---|---|
| `rustploy-apps` | `rustploy-app` | criar um app (Rust + Axum + Postgres, Vue) e colocá-lo no ar com banco, migrations, usuário inicial, secrets, subdomínio e HTTPS |

## Instalar

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

Depois, conecte o Claude Code ao MCP do servidor (uma vez por máquina): na webui do
rustploy, **Settings → Web Server → "Conectar um agente (Claude Code)" → Copiar**, e
cole o comando no terminal. O token é de administrador: não o coloque em arquivo
nenhum de repositório.

A skill entra sozinha quando o pedido for criar, publicar ou mexer num app
("cria um sistema de cadastro de clientes e coloca no ar"). Para chamá-la à força:
`/rustploy-apps:rustploy-app`.

## Atualizar

```text
/plugin marketplace update rustploy-skills
```

Para receber sozinho: `/plugin` → **Marketplaces** → `rustploy-skills` → ligar a
atualização automática.

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

Nada aqui pode conter segredo: o repositório é lido por qualquer pessoa que instale a skill.
