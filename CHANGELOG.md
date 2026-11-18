# Changelog

Toda mudança numa skill entra aqui e sobe a `version` em
`plugins/<plugin>/.claude-plugin/plugin.json` **e** em `.claude-plugin/marketplace.json`.
Sem o bump, quem já instalou não recebe a atualização.

## rustploy-apps 0.2.0 — 2026-11-18

- Funciona também no **Codex** (testado no Codex CLI 0.154): ele lê o mesmo marketplace e o mesmo
  plugin. `codex plugin marketplace add antoniofernandodj/rustploy-skills` +
  `codex plugin add rustploy-apps@rustploy-skills`; MCP com
  `codex mcp add rustploy --url … --bearer-token-env-var RUSTPLOY_TOKEN`.
- O texto da skill ficou neutro: as ferramentas são citadas pelo nome do servidor `rustploy`, não
  pelo prefixo do Claude Code, e a seção "Conectar" traz os dois agentes.

## rustploy-apps 0.1.0 — 2026-11-18

- Primeira versão da skill `rustploy-app`: regras (segredos, senhas, seed, migrations,
  banco da plataforma, confirmação antes de destruir, teste da URL pública), caminho do
  zero ao ar pelo MCP, subdomínio na Hostinger, mapa de erros comuns.
- Templates testados: backend Rust/Axum/SQLx (migrations, seed idempotente, login JWT,
  argon2, healthcheck com banco) e frontend Vue 3 + nginx com repasse de `/api/`
  (resolver do próprio container, compatível com Podman). As duas imagens foram
  construídas sem BuildKit.
- Senhas e chaves nascem com `SecretGenerate` (o daemon sorteia; o agente não vê o valor) e
  a pessoa lê pelo 👁 da webui. Requer o daemon 0.1.66 ou mais novo (release v0.1.79); num daemon antigo a skill
  cai no caminho manual (a pessoa cria o secret).
- Ainda **não** validado num deploy real no rustploy-ms: o primeiro app vai confirmar os
  JSONs de `ServiceCreate`/`JobCreate` e o resolver do nginx no Podman.
