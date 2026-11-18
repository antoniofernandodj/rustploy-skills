# Subdomínio na Hostinger e HTTPS no rustploy

O DNS de `itemize.com.br` é da **Hostinger** (servidores `ns1/ns2.dns-parking.com`).
O rustploy **não mexe em DNS**: ele só recebe o tráfego e emite o certificado
(Let's Encrypt) depois que o nome já aponta para o servidor. **Não existe registro
curinga** (`*.itemize.com.br`): cada subdomínio novo precisa do seu registro.

## Precisa de subdomínio?

| Situação | Subdomínio? |
|---|---|
| app que pessoas abrem no navegador | **sim, um** (o do frontend) |
| backend/API do mesmo app | **não**: o frontend repassa `/api/` pela rede interna |
| API que **outro sistema de fora** vai chamar (integração, app mobile, webhook) | sim, um para a API (`<app>-api.itemize.com.br`) |
| banco, job, worker, fila | **nunca** |
| app só de teste que ninguém de fora vai abrir | não; use quando for mostrar a alguém |
| domínio que não é `itemize.com.br` (ex.: de cliente) | pergunte quem controla o DNS desse domínio; o passo é o mesmo, em outro painel |

Padrão de nome: `<app>.itemize.com.br` (minúsculas, hífen, sem acento). Não
reutilize um nome que já está em uso: confira em `ingress_routes` e com `dig`.

## 1. Já resolve?

```bash
dig +short meuapp.itemize.com.br
dig +short rustploy-ms.itemize.com.br   # o IP atual do servidor
```

Se os dois devolvem o mesmo IP, pule para o passo 3.

## 2. Criar o registro (feito pela pessoa, no painel)

Você não tem acesso à Hostinger. Passe esta instrução, com o nome preenchido:

> Precisamos criar o endereço **meuapp.itemize.com.br**. Na Hostinger:
> 1. Entre em **hpanel.hostinger.com** → **Domínios** → **itemize.com.br** →
>    **DNS / Nameservers** (ou "Gerenciar registros DNS").
> 2. Em **Adicionar registro**, escolha o tipo **CNAME**.
> 3. **Nome**: `meuapp` (só a parte antes de `.itemize.com.br`).
> 4. **Aponta para / Destino**: `rustploy-ms.itemize.com.br`.
> 5. **TTL**: deixe o padrão (ou 300). Salve.
> Me avise quando salvar.

Por que CNAME: se o IP do servidor mudar, só o registro `rustploy-ms` precisa ser
atualizado. Se o painel recusar CNAME para esse nome (já existe outro registro
com o mesmo nome), use um registro **A** com o IP que o
`dig +short rustploy-ms.itemize.com.br` mostrou, e avise que existia algo lá.

Não peça para apagar nem editar registros existentes (e-mail `MX`, `TXT` de
verificação, o `www`…). Se parecer necessário, explique o motivo e deixe a
decisão com a pessoa.

## 3. Esperar propagar

```bash
until [ "$(dig +short meuapp.itemize.com.br @1.1.1.1 | tail -1)" = "$(dig +short rustploy-ms.itemize.com.br @1.1.1.1 | tail -1)" ]; do sleep 20; done
```

Normalmente leva de 1 a 15 minutos. Use um limite de tempo (por exemplo 30 minutos)
e, se não resolver, confira com a pessoa se o nome foi digitado certo.

## 4. Ligar o domínio no rustploy

No serviço **web** (o frontend), campo `domains`:

```json
[{"domain":"meuapp.itemize.com.br","port":null,"tls":true}]
```

`port: null` = a porta do serviço (80 no nginx). `tls: true` sempre, salvo pedido
explícito. Se o serviço já existe: `ServiceGet` → acrescenta o domínio → spec
inteiro no `ServiceUpdate` → `deploy`. O certificado é pedido quando o deploy chega
a `Live` e é renovado sozinho.

## 5. Conferir

```bash
curl -sI https://meuapp.itemize.com.br/ | head -1     # HTTP/2 200 (ou 1.1 200)
curl -s  https://meuapp.itemize.com.br/api/health      # {"status":"ok"}
```

## Problemas comuns

| Sintoma | Causa provável | O que fazer |
|---|---|---|
| erro de certificado logo após o primeiro deploy | o certificado ainda está saindo | espere 1–2 min e tente de novo |
| certificado nunca sai | DNS não apontava no momento do deploy; ou o registro está errado | confira o `dig`; faça novo `deploy` do web depois que resolver |
| `502` | rota sem backend ou container parado | `ingress_routes`; ver a seção 502 em [erros-comuns.md](erros-comuns.md) |
| o domínio abre outro site | registro aponta para outro IP (hospedagem antiga da Hostinger) | corrigir o registro para `rustploy-ms.itemize.com.br` |
| funciona no celular e não no PC | cache de DNS local | esperar o TTL ou `sudo resolvectl flush-caches` |
