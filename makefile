.PHONY: build
build:
	cargo b

.PHONY: test
test:
	cargo test

.PHONY: repl
repl:
	cargo r

.PHONY: run
run:
	cargo r -- ./sample/sample.lox

.PHONY: watch
watch:
	cargo watch -x build

.PHONY: act
act:
	act -b --container-architecture linux/amd64