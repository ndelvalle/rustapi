# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.79.0
ARG APP_NAME=rustapi

################################################################################
# Create a stage for building the application.

FROM rust:${RUST_VERSION}-alpine AS build

ARG APP_NAME

# Adding necessary packages
RUN apk update
RUN apk add pkgconfig openssl openssl-dev musl-dev

# Build the application.
RUN rustup target add x86_64-unknown-linux-musl

# copy  the source 
WORKDIR /app
COPY . .

RUN cargo build --target=aarch64-unknown-linux-musl --release

# RUN strip ./target/release/aarch64-unknown-linux-musl/$APP_NAME 

# RUN cp ./target/release/aarch64-unknown-linux-musl/$APP_NAME /bin/server

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base
# image from the build stage where the necessary files are copied from the build
# stage.
#
# The example below uses the alpine image as the foundation for running the app.
# By specifying the "3.18" tag, it will use version 3.18 of alpine. If
# reproducability is important, consider using a digest
# (e.g., alpine@sha256:664888ac9cfd28068e062c991ebcff4b4c7307dc8dd4df9e728bedde5c449d91).

FROM alpine:latest AS final

WORKDIR /app 
RUN cd /app
# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/go/dockerfile-user-best-practices/
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /app/target/release/$APP_NAME /bin/server

# Expose the port that the application listens on.
EXPOSE 3000

# What the container should run when it is started.
CMD ["/bin/server"]
