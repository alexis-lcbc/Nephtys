FROM rust:alpine3.21 as build
LABEL org.opencontainers.image.authors="alexis.lecabellec@gmail.com"
LABEL org.opencontainers.image.source https://github.com/alexis-lcbc/Nephtys

# Setup Server
WORKDIR /server
RUN apk add --no-cache ffmpeg
RUN apk add --no-cache opencv
COPY server/ /server/

RUN cargo build --release


FROM node:22-alpine3.21
LABEL org.opencontainers.image.authors="alexis.lecabellec@gmail.com"
LABEL org.opencontainers.image.source https://github.com/alexis-lcbc/Nephtys

# Prepare backend
RUN apk add --no-cache supervisor
RUN mkdir /server
COPY config.toml /server/config.toml
COPY --from=build /server/target/release/nephtys-server /server/nephtys-server



# Setup Web APp
WORKDIR /web
COPY web/ /web/
RUN npm install
RUN npm run build
WORKDIR /web/build

# We use supervisord to start both the backend & frontend together and stop if one fails
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf
EXPOSE 3000
EXPOSE 8080
CMD ["/usr/bin/supervisord"]