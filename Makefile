GIT_COMMIT=$(shell git rev-parse HEAD --short)
VERSION=$(shell cat ./version.json | jq '.version' | tr -d '"')
BUILD_DATE=$(shell date +"%D")

.PHONY: build
build:
	go build -ldflags "-s -w -X main.version=${VERSION} -X main.commit=${GIT_COMMIT} -X main.buildDate=${BUILD_DATE}" -o ./build/ume

build-docker:
	docker build . --no-cache -t auguwu/ume:latest
	docker build . --no-cache -t auguwu/ume:$(VERSION)

publish-docker:
	docker push auguwu/ume:latest
	docker push auguwu/ume:$(VERSION)

goreleaser:
	docker pull goreleaser/goreleaser
	docker run --rm --privileged \
		-v $$PWD:/go/src/floofy.dev/ume \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-w /go/src/floofy.dev/ume \
		-e GITHUB_TOKEN=$(GITHUB_TOKEN) \
		goreleaser/goreleaser

goreleaser-test:
	docker pull goreleaser/goreleaser
	docker run --rm --privileged \
		-v $$PWD:/go/src/floofy.dev/ume \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-w /go/src/floofy.dev/ume \
		-e GITHUB_TOKEN=$(GITHUB_TOKEN) \
		goreleaser/goreleaser --rm-dist --skip-publish --snapshot
