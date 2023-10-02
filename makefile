.PHONY: docker-build
docker-build:
	@ docker build -t r-lox .

.PHONY: build
build: docker-build
	@ docker run \
		--mount type=volume,src=r-lox,target=/app/target \
		-t \
		--rm \
		r-lox \
		cargo b

.PHONY: test
test: docker-build
	@ docker run \
		--mount type=volume,src=r-lox,target=/app/target \
		-t \
		--rm \
		r-lox \
		cargo t

.PHONY: repl
repl: docker-build
	@ docker run \
		--mount type=volume,src=r-lox,target=/app/target \
		-ti \
		--rm \
		r-lox \
		cargo r

.PHONY: run
run: docker-build
	@ docker run \
		--mount type=volume,src=r-lox,target=/app/target \
		-t \
		--rm \
		r-lox \
		cargo r -- ./sample/sample.lox

.PHONY: watch
watch: docker-build
	@ docker run \
		--mount type=volume,src=r-lox,target=/app/target \
		-t \
		--rm \
		r-lox \
		cargo watch -x build

.PHONY: clippy
clippy: docker-build
	@ docker run \
		--mount type=volume,src=r-lox,target=/app/target \
		-t \
		--rm \
		r-lox \
		cargo clippy

.PHONY: act
act:
	act -b --container-architecture linux/amd64