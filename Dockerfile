FROM rust:1.81-slim-bookworm AS build
ARG CPU_TARGET
WORKDIR /app
COPY . /app
RUN if [ "$CPU_TARGET" = "neoverse" ]; then \
      export RUSTFLAGS="-Ctarget-feature=+lse -Ctarget-cpu=neoverse-n1"; \
    fi
RUN cargo build --release

FROM gcr.io/distroless/cc

WORKDIR /usr/src/bloom

COPY --from=build /app/target/release/bloom /usr/local/bin/bloom

CMD [ "bloom", "-c", "/etc/bloom.cfg" ]

EXPOSE 8080 8811
