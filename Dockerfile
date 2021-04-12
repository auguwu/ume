FROM golang:1.16-alpine AS builder
WORKDIR /
COPY . .
RUN go get
RUN go build

FROM alpine:latest
WORKDIR /
COPY --from=builder /ume /app/ume
CMD ["/app/ume"]
