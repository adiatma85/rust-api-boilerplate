FROM alpine:3.23

ARG SERVICE_VERSION
ENV SERVICE_VERSION=$SERVICE_VERSION

WORKDIR /project

# Dependencies:
RUN apk add --no-cache \
    libstdc++ \
    tzdata \
    libx11 \
    libxrender \
    libxext \
    libssl3 \
    ca-certificates

# Create logs directory
RUN mkdir -p ./logs

# Ensure this matches the output location in the Makefile
COPY ./build/app ./app
COPY ./etc/cfg etc/cfg

CMD [ "./app" ]
