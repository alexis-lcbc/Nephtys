FROM rust:alpine3.21
LABEL org.opencontainers.image.authors="alexis.lecabellec@gmail.com"
LABEL org.opencontainers.image.source https://github.com/alexis-lcbc/Nephtys

# Setup Server
WORKDIR /server
RUN apk add --no-cache ffmpeg
RUN apk add --no-cache opencv
COPY server/ /server/
COPY config.toml /server/config.toml
# Start the backend in a seperate thread (port 8080)
EXPOSE 8080
RUN cargo build --release
CMD [ "cargo", "run" ]


FROM node:22-alpine3.21
LABEL org.opencontainers.image.authors="alexis.lecabellec@gmail.com"
LABEL org.opencontainers.image.source https://github.com/alexis-lcbc/Nephtys

# Setup Web APp
WORKDIR /web
COPY web/ /web/
RUN npm install
RUN npm run build
WORKDIR /web/build
# Start the frontend in a seperate thread (port 3000)
EXPOSE 3000
CMD ["node" "index.js"]