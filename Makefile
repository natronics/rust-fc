all: rust-fc

rust-fc: src/main.rs
	rustc $^ -o rust-fc
