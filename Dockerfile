# There may be a vuln in rust:bookworm for now but we don't care BECAUSE it's a build step
FROM rust:trixie as build
LABEL org.opencontainers.image.authors="alexis.lecabellec@gmail.com"
LABEL org.opencontainers.image.source https://github.com/alexis-lcbc/Nephtys

# Setup Server
WORKDIR /server
RUN apt-get update -y
RUN apt-get install libopencv-dev clang libclang-dev -y

COPY server/ /server/

RUN --mount=type=cache,id=rust-nephtys-backend,target=/src/target cargo build --release


FROM node:trixie-slim
LABEL org.opencontainers.image.authors="alexis.lecabellec@gmail.com"
LABEL org.opencontainers.image.source https://github.com/alexis-lcbc/Nephtys

# Prepare backend
RUN apt-get update
RUN apt-get install supervisor -y
RUN apt-get install ffmpeg -y
RUN apt-get install libopencv-core410 libopencv-imgproc410 libopencv-videoio410 -y


# Setup Web APp
WORKDIR /web
COPY web/ /web/
RUN npm install
RUN npm run build

# Finalize backend prep
RUN mkdir /server
# COPY config.toml /server/config.toml ## Now done by mounting a volume
COPY --from=build /server/target/release/nephtys-server /server/nephtys-server

# We use supervisord to start both the backend & frontend together and stop if one fails
COPY supervisord.conf /etc/supervisord.conf
EXPOSE 3000
EXPOSE 8080
CMD ["/usr/bin/supervisord"]
# ENTRYPOINT ["/usr/bin/env"]