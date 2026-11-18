# Frontend Vue 3 + Vite servido por nginx. Construído pelo rustploy a partir do Git com
#   contexto de build: frontend   Dockerfile: Dockerfile
# Sem variáveis no build: a URL da API é sempre o caminho relativo `/api`.

FROM node:22-alpine AS build
WORKDIR /build
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:1.27-alpine
RUN rm -f /etc/nginx/conf.d/default.conf
COPY docker/default.conf.template /etc/nginx/templates/default.conf.template
COPY --from=build /build/dist /usr/share/nginx/html
# NGINX_ENTRYPOINT_LOCAL_RESOLVERS: a imagem lê o DNS do /etc/resolv.conf do container
# e expõe em ${NGINX_LOCAL_RESOLVERS}. Funciona em Docker e em Podman (o rustploy-ms usa
# Podman: o 127.0.0.11 do Docker NÃO existe lá).
# API_UPSTREAM: o backend pela rede interna do projeto (rp_<nome do serviço>:<porta>).
ENV NGINX_ENTRYPOINT_LOCAL_RESOLVERS=1 \
    API_UPSTREAM=http://rp_api:8080
EXPOSE 80
