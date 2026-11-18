# Changelog

Toda mudança numa skill entra aqui e sobe a `version` em
`plugins/<plugin>/.claude-plugin/plugin.json` **e** em `.claude-plugin/marketplace.json`.
Sem o bump, quem já instalou não recebe a atualização.

## rustploy-apps 0.1.0 — 2026-11-18

- Primeira versão da skill `rustploy-app`: regras (segredos, senhas, seed, migrations,
  banco da plataforma, confirmação antes de destruir, teste da URL pública), caminho do
  zero ao ar pelo MCP, subdomínio na Hostinger, mapa de erros comuns.
- Templates testados: backend Rust/Axum/SQLx (migrations, seed idempotente, login JWT,
  argon2, healthcheck com banco) e frontend Vue 3 + nginx com repasse de `/api/`
  (resolver do próprio container, compatível com Podman). As duas imagens foram
  construídas sem BuildKit.
- Senhas e chaves nascem com `SecretGenerate` (o daemon sorteia; o agente não vê o valor) e
  a pessoa lê pelo 👁 da webui. Requer o daemon com `SecretGenerate`; num daemon antigo a skill
  cai no caminho manual (a pessoa cria o secret).
- Ainda **não** validado num deploy real no rustploy-ms: o primeiro app vai confirmar os
  JSONs de `ServiceCreate`/`JobCreate` e o resolver do nginx no Podman.
