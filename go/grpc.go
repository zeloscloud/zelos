package zelos

import (
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/keepalive"
)

// MaxGrpcMessageSize is the maximum gRPC message size (send and receive) used
// across the SDK. It bounds both call message sizes and connection windows.
const MaxGrpcMessageSize = 100 * 1024 * 1024

// DefaultDialOptions returns the project-wide gRPC dial options: HTTP/2
// keepalive, max message sizes, and insecure transport credentials. Every
// client that dials an endpoint should use these so behavior stays consistent.
func DefaultDialOptions() []grpc.DialOption {
	return []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultCallOptions(
			grpc.MaxCallRecvMsgSize(MaxGrpcMessageSize),
			grpc.MaxCallSendMsgSize(MaxGrpcMessageSize),
		),
		grpc.WithKeepaliveParams(keepalive.ClientParameters{
			Time:                30 * time.Second,
			Timeout:             5 * time.Second,
			PermitWithoutStream: true,
		}),
	}
}
