FROM node:10-alpine

WORKDIR /app

RUN apk add --no-cache python make g++

COPY ./scripts ./scripts
COPY ./backend ./backend
COPY ./packages ./packages
COPY ./package.json ./yarn.lock ./tsconfig.json ./

RUN yarn 


# Frontend is exposing 3000
# Backend is exposing 8080
# No need for expose, if using docker-compose & docker run -p 3000:3000
