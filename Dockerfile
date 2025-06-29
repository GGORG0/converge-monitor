FROM rust AS app-builder
WORKDIR /usr/src/app

COPY . .
RUN cargo install --path .

FROM debian AS app
WORKDIR /root

COPY --from=app-builder /usr/local/cargo/bin/converge-monitor /usr/local/bin/converge-monitor

EXPOSE 8080

CMD [ "converge-monitor" ]