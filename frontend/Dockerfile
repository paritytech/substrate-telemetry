#### BUILDER IMAGE -- quite big one ####

FROM node:10-alpine as builder
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry frontend build image"

ARG PROFILE=production

WORKDIR /opt/build

RUN apk add --no-cache python make g++

COPY . . 
RUN yarn && yarn build --${PROFILE}}


# MAIN IMAGE FOR PEOPLE TO PULL --- small one#

FROM node:10-alpine
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry frontend build image"

ARG PROFILE=production

WORKDIR /app

COPY --from=builder /opt/build/build/dist  /app

ENTRYPOINT /app/index.js