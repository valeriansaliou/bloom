FROM rustlang/rust:nightly

WORKDIR /usr/src/bloom

RUN cargo install bloom-server
CMD [ "bloom", "-c", "/etc/bloom.cfg" ]

EXPOSE 8080 8811
