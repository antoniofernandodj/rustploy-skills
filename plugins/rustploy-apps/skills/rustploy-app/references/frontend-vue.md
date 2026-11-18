# Frontend: Vue 3 + Vite + TypeScript, servido por nginx

## Criar

```bash
npm create vite@6 frontend -- --template vue-ts
cd frontend && npm install && npm install vue-router@4 pinia@3
```

Depois copie por cima o conteúdo de [`templates/frontend/`](../templates/frontend/)
e apague `src/components/HelloWorld.vue` e `src/style.css`. O resultado compila
(`npm run build`) e, na imagem, foi testado com o backend do template: login,
rotas do Vue e troca do container do backend sem 502.

```text
frontend/
├── Dockerfile  .dockerignore  vite.config.ts  package.json  package-lock.json
├── docker/default.conf.template   nginx: SPA + repasse de /api/ ao backend
└── src/
    ├── api/http.ts       o ÚNICO lugar que faz fetch
    ├── stores/auth.ts    Pinia: login, usuário atual, logout
    ├── router/index.ts   toda rota exige login, salvo meta.public
    └── views/            LoginView.vue, HomeView.vue, …
```

`package-lock.json` é versionado (o `Dockerfile` usa `npm ci`).

## Como o frontend fala com o backend

```text
navegador ──https──▶ meuapp.itemize.com.br (ingress do rustploy, TLS)
                         └──▶ serviço web (nginx :80)
                                ├── /            arquivos do Vue
                                └── /api/…  ──▶ http://rp_api:8080/api/…  (rede interna do projeto)
```

- No código, **sempre** `api('/caminho')`, que vira `fetch('/api/caminho')`.
  Nada de `http://localhost:8080`, nada de domínio da API, nada de `VITE_API_URL`.
- Mesma origem: **sem CORS**, um único certificado, a API não fica exposta com
  domínio próprio.
- Em desenvolvimento o `vite.config.ts` repassa `/api` para `localhost:8080`.
- O endereço interno do backend vem da variável `API_UPSTREAM` do serviço `web`
  (padrão `http://rp_api:8080`). Serviço Application no rustploy = `rp_<nome>`;
  se o backend se chamar `backend`, é `http://rp_backend:8080`.

## nginx: por que o template é assim

- `resolver ${NGINX_LOCAL_RESOLVERS}` com `NGINX_ENTRYPOINT_LOCAL_RESOLVERS=1`:
  usa o DNS que o container recebeu. **Não** escreva `resolver 127.0.0.11`: isso
  só existe no Docker; o rustploy-ms usa Podman e a chamada fica pendurada (é o que
  acontece hoje com `/api` do rdo).
- `set $api_upstream …; proxy_pass $api_upstream$request_uri;`: com variável, o
  nginx resolve o nome a cada requisição; sem ela, guarda o IP do container antigo
  e dá 502 após cada deploy do backend.
- `/healthz` é o healthcheck do serviço `web`.
- `try_files … /index.html` faz o refresh em `/clientes/12` funcionar.
- `index.html` sem cache e `/assets/` com cache longo: deploy novo aparece na hora.

## Regras

- **Nenhum segredo no frontend.** Tudo o que vai para o navegador é público,
  inclusive variáveis `VITE_*`. Chave de API de terceiro → chamada pelo backend.
- Toda chamada passa por `src/api/http.ts` (token, erro e 401 tratados num lugar).
- Token só no `localStorage` com expiração curta (o JWT do backend vence em 8 h);
  nunca guardar a senha.
- Esconder botão não é segurança: o backend precisa checar a permissão.
- Estado compartilhado em Pinia; estado de uma tela fica na tela.
- Telas em português; código (nomes de arquivos, variáveis, funções) em inglês.
- Antes de subir: `npm run build` sem erros de TypeScript.

## Variáveis do serviço `web` no rustploy

| Variável | Valor |
|---|---|
| `API_UPSTREAM` | `http://rp_api:8080` (ajuste ao nome e porta do backend) |

Nada mais. Se precisar de configuração visível no navegador (nome do ambiente,
por exemplo), sirva-a pelo backend (`/api/config`) em vez de compilar no build.
