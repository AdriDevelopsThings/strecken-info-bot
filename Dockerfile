FROM rust:alpine as build
WORKDIR /build
ARG GIT_SHA

RUN apk add musl-dev ca-certificates

COPY ./Cargo.lock ./Cargo.toml ./
COPY ./src ./src

RUN cargo build --release

FROM scratch
WORKDIR /app

ENV PATH="$PATH:/app/bin"

COPY --from=build /build/target/release/strecken-info-bot /app/bin/strecken-info-bot
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

ENV SQLITE_PATH=/database/db.sql
VOLUME [ "/database" ]

ENV METRICS_LISTEN_ADDRESS=0.0.0.0:80
EXPOSE 80

CMD ["/app/bin/strecken-info-bot"]