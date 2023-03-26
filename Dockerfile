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

COPY --from=build /build/target/release/strecken-info-telegram /app/bin/strecken-info-telegram
COPY --from=build /build/target/release/cli /app/bin/strecken-info-telegram-cli
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

ENV SQLITE_PATH=/database/db.sql
VOLUME [ "/database" ]

CMD ["/app/bin/strecken-info-telegram"]