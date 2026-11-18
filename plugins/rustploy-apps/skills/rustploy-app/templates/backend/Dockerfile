# Backend Rust + Axum. Construído pelo rustploy a partir do Git com
#   contexto de build: backend   Dockerfile: Dockerfile
#
# O servidor usa o construtor clássico (sem BuildKit): nada de `RUN --mount`.
# O cache de dependências é feito compilando um esqueleto antes do código real.
# O rustploy NÃO passa variáveis no build: nenhum ARG/ENV com segredo aqui.

FROM rust:1.90-slim-bookworm AS build
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY migrations ./migrations
COPY src ./src
RUN find src -name '*.rs' -exec touch {} + && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app
COPY --from=build /build/target/release/app-backend /usr/local/bin/app-backend
USER app
ENV BIND_ADDRESS=0.0.0.0:8080 RUST_LOG=info
EXPOSE 8080
CMD ["/usr/local/bin/app-backend", "serve"]
