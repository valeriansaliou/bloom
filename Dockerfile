FROM rustlang/rust:nightly-slim AS build

RUN cargo install bloom-server

FROM debian:stretch-slim

WORKDIR /usr/src/bloom

COPY --from=build /usr/local/cargo/bin/bloom /usr/local/bin/bloom

CMD [ "bloom", "-c", "/etc/bloom.cfg" ]

EXPOSE 8080 8811
