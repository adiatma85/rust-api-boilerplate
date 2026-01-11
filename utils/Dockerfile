FROM alpine:3.23

ARG SERVICE_VERSION
ENV SERVICE_VERSION=$SERVICE_VERSION

WORKDIR /project

# Dependencies:
# Rust binaries built with musl are usually static, but if you use OpenSSL,
# you might need libssl1.1 (or libssl3 on newer alpine).
# Keeping your existing list as it covers most bases.
RUN apk add --no-cache \
    libstdc++ \
    tzdata \
    libx11 \
    libxrender \
    libxext \
    libssl1.1 \
    ca-certificates;

# Ensure this matches the output location in the Makefile
COPY ./build/app ./app
# COPY ./docs docs
COPY ./etc/cfg etc/cfg

CMD [ "./app" ]
