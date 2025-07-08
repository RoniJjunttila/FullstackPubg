# nginx/Dockerfile

FROM nginx:stable-alpine-slim
COPY nginx/default.config /etc/nginx/conf.d/default.conf
COPY --from=frontend-build /app/build /usr/share/nginx/html
