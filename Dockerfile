FROM rust AS app-builder
WORKDIR /usr/src/app

COPY . .
RUN cargo install --path .

FROM debian AS app
WORKDIR /root

RUN apt-get update && \
  apt-get install -y ca-certificates && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

COPY --from=app-builder /usr/local/cargo/bin/converge-monitor /usr/local/bin/converge-monitor

CMD [ "converge-monitor" ]