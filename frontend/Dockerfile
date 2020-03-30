#### BUILDER IMAGE -- quite big one ####

FROM node:13-alpine as builder
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry frontend build image"

ARG PROFILE=production

WORKDIR /opt/build

RUN apk add --no-cache python make g++

COPY . . 
RUN yarn && yarn build --${PROFILE}}


# MAIN IMAGE FOR PEOPLE TO PULL --- small one#

FROM node:13-alpine
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry frontend build image"

ARG PROFILE=production

EXPOSE 3000

WORKDIR /app

COPY --from=builder /opt/build/build  /app

RUN npm install -g serve

CMD [ "serve","-s","/app" ]