FROM golang:1.17-alpine AS builder

RUN apk update && apk add --no-cache ca-certificates git make jq

WORKDIR /build/ume
COPY . .
RUN go get
RUN make build

FROM alpine:latest

WORKDIR /app/noel/ume
COPY --from=builder /build/ume/bin/ume                     /app/noel/ume/ume
COPY --from=builder /build/ume/docker/runner.sh            /app/noel/ume/scripts/runner.sh
COPY --from=builder /build/ume/docker/scripts/liblog.sh    /app/noel/ume/scripts/liblog.sh
COPY --from=builder /build/ume/docker/docker-entrypoint.sh /app/noel/ume/scripts/docker-entrypoint.sh

USER 1001
ENTRYPOINT ["/app/noel/ume/scripts/docker-entrypoint.sh"]
CMD ["/app/noel/ume/scripts/runner.sh"]
