FROM golang:1.17-alpine AS builder

RUN apk update && apk add --no-cache make jq git

WORKDIR /
COPY . .
RUN go get
RUN make build

FROM alpine:latest
WORKDIR /app/ume
COPY --from=builder /build/ume /app/ume/ume
CMD ["/app/ume/ume"]