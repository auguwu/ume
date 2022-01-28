FROM alpine:latest

WORKDIR /app/noel/ume
COPY docker/docker-entrypoint.sh /app/noel/ume/scripts/docker-entrypoint.sh
COPY docker/scripts/liblog.sh    /app/noel/ume/scripts/liblog.sh
COPY docker/runner.sh            /app/noel/ume/scripts/runner.sh
COPY ume                         /app/noel/ume/ume

USER 1001
ENTRYPOINT ["/app/noel/ume/scripts/docker-entrypoint.sh"]
CMD ["/app/noel/ume/scripts/runner.sh"]
