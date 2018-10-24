all:
	cargo run -- -a example.s example.o
	cargo run -- -i example.o 20
