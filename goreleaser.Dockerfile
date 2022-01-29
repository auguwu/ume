FROM alpine:latest

RUN apk update && apk add --no-cache bash musl-dev libc-dev gcompat

WORKDIR /app/noel/ume
COPY docker/docker-entrypoint.sh /app/noel/ume/scripts/docker-entrypoint.sh
COPY docker/scripts/liblog.sh    /app/noel/ume/scripts/liblog.sh
COPY docker/runner.sh            /app/noel/ume/scripts/runner.sh
COPY ume                         /app/noel/ume/ume

RUN chmod +x /app/noel/ume/scripts/docker-entrypoint.sh
RUN chmod +x /app/noel/ume/scripts/runner.sh

USER 1001
ENTRYPOINT ["/app/noel/ume/scripts/docker-entrypoint.sh"]
CMD ["/app/noel/ume/scripts/runner.sh"]
