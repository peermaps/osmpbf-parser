all: fileformat.proto osmformat.proto

fileformat.proto: proto/fileformat.proto
	mkdir -p src/proto
	pb-rs proto/fileformat.proto -o src/proto/fileformat.rs

osmformat.proto: proto/osmformat.proto
	mkdir -p src/proto
	pb-rs proto/osmformat.proto -o src/proto/osmformat.rs

clean:
	rm src/proto/*.rs
