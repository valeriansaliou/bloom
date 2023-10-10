FROM rust:1.73-slim-buster AS build

WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc

WORKDIR /usr/src/bloom

COPY --from=build /app/target/release/bloom /usr/local/bin/bloom

CMD [ "bloom", "-c", "/etc/bloom.cfg" ]

EXPOSE 8080 8811
