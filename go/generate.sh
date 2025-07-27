#!/bin/bash
set -e

echo "Generating Go protobuf files..."

# Add Go bin directory to PATH
export PATH="$PATH:$(go env GOPATH)/bin"

# Check if protoc tools are installed
if ! command -v protoc-gen-go &> /dev/null; then
    echo "Installing protoc-gen-go..."
    go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
fi

if ! command -v protoc-gen-go-grpc &> /dev/null; then
    echo "Installing protoc-gen-go-grpc..."
    go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
fi

# Verify tools are available
if ! command -v protoc-gen-go &> /dev/null; then
    echo "Error: protoc-gen-go not found in PATH after installation"
    echo "PATH: $PATH"
    echo "GOPATH: $(go env GOPATH)"
    exit 1
fi

if ! command -v protoc-gen-go-grpc &> /dev/null; then
    echo "Error: protoc-gen-go-grpc not found in PATH after installation"
    exit 1
fi

# Generate protobuf files
protoc --go_out=. --go_opt=paths=source_relative \
       --go-grpc_out=. --go-grpc_opt=paths=source_relative \
       --proto_path=proto \
       proto/zeloscloud/trace/*.proto

echo "Protobuf generation complete!"
echo "Generated files:"
find . -name "*.pb.go" -o -name "*_grpc.pb.go"
