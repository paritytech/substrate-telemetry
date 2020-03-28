FROM node:10-alpine

WORKDIR /app

RUN apk add --no-cache python make g++

COPY ./scripts ./scripts
COPY ./packages ./packages
COPY ./package.json ./yarn.lock ./tsconfig.json ./

RUN yarn && yarn cache clean
