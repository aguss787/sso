FROM codesimple/elm:0.19 AS web_builder

RUN ["mkdir", "-p", "/opt/app"]
WORKDIR /opt/app

COPY . .

RUN ["/opt/app/build-static.sh", "--optimize"]

FROM rust:1.80.0 AS api_builder

RUN ["mkdir", "-p", "/opt/app"]
WORKDIR /opt/app

COPY . .

RUN ["cargo", "build", "--release"]


FROM ubuntu:latest
LABEL authors="agus"

RUN ["apt-get", "update"]
RUN ["apt-get", "install", "-y", "libpq5"]

RUN ["mkdir", "-p", "/opt/app"]
WORKDIR /opt/app

COPY --from=api_builder /opt/app/target/release/sso /opt/app/
COPY --from=api_builder /opt/app/target/release/migrate /opt/app/
COPY --from=web_builder /opt/app/static /opt/app/static

CMD ["/opt/app/sso"]
