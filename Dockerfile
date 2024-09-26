FROM rust:1.81 AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . . 
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/gathering_surf gathering_surf 
COPY config config
COPY assets assets
COPY templates templates
ENV APP_ENVIRONMENT=production 
ENTRYPOINT ["./gathering_surf"]
