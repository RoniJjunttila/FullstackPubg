FROM node:18-alpine AS build
WORKDIR /app

COPY package*.json ./
RUN npm install && npm cache clean --force

COPY public ./public
COPY src ./src
RUN npm run build

FROM nginx:stable-alpine-slim
RUN rm -rf /usr/share/nginx/html/*
COPY --from=build /app/build /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
