NAME := rs
COLOR ?= auto # {always, auto, never}
CARGO := cargo --color ${COLOR}

.PHONY: bench build check clean doc install publish run test update

build:
	@${CARGO} build

check:
	@${CARGO} check

release:
	@${CARGO} build --release
	@strip target/release/${NAME}

bench:
	@${CARGO} bench

doc:
	@${CARGO} doc

install: build
	@${CARGO} install

publish:
	@${CARGO} publish

run: build
	@${CARGO} run

test: build
	@${CARGO} test --all

update:
	@${CARGO} update

clean:
	@${CARGO} clean
