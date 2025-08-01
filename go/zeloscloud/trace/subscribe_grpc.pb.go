// Code generated by protoc-gen-go-grpc. DO NOT EDIT.
// versions:
// - protoc-gen-go-grpc v1.5.1
// - protoc             v5.27.0
// source: zeloscloud/trace/subscribe.proto

package trace

import (
	context "context"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
// Requires gRPC-Go v1.64.0 or later.
const _ = grpc.SupportPackageIsVersion9

const (
	TraceSubscribe_Subscribe_FullMethodName = "/zeloscloud.trace.TraceSubscribe/Subscribe"
)

// TraceSubscribeClient is the client API for TraceSubscribe service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://pkg.go.dev/google.golang.org/grpc/?tab=doc#ClientConn.NewStream.
type TraceSubscribeClient interface {
	Subscribe(ctx context.Context, opts ...grpc.CallOption) (grpc.BidiStreamingClient[SubscribeRequest, SubscribeResponse], error)
}

type traceSubscribeClient struct {
	cc grpc.ClientConnInterface
}

func NewTraceSubscribeClient(cc grpc.ClientConnInterface) TraceSubscribeClient {
	return &traceSubscribeClient{cc}
}

func (c *traceSubscribeClient) Subscribe(ctx context.Context, opts ...grpc.CallOption) (grpc.BidiStreamingClient[SubscribeRequest, SubscribeResponse], error) {
	cOpts := append([]grpc.CallOption{grpc.StaticMethod()}, opts...)
	stream, err := c.cc.NewStream(ctx, &TraceSubscribe_ServiceDesc.Streams[0], TraceSubscribe_Subscribe_FullMethodName, cOpts...)
	if err != nil {
		return nil, err
	}
	x := &grpc.GenericClientStream[SubscribeRequest, SubscribeResponse]{ClientStream: stream}
	return x, nil
}

// This type alias is provided for backwards compatibility with existing code that references the prior non-generic stream type by name.
type TraceSubscribe_SubscribeClient = grpc.BidiStreamingClient[SubscribeRequest, SubscribeResponse]

// TraceSubscribeServer is the server API for TraceSubscribe service.
// All implementations must embed UnimplementedTraceSubscribeServer
// for forward compatibility.
type TraceSubscribeServer interface {
	Subscribe(grpc.BidiStreamingServer[SubscribeRequest, SubscribeResponse]) error
	mustEmbedUnimplementedTraceSubscribeServer()
}

// UnimplementedTraceSubscribeServer must be embedded to have
// forward compatible implementations.
//
// NOTE: this should be embedded by value instead of pointer to avoid a nil
// pointer dereference when methods are called.
type UnimplementedTraceSubscribeServer struct{}

func (UnimplementedTraceSubscribeServer) Subscribe(grpc.BidiStreamingServer[SubscribeRequest, SubscribeResponse]) error {
	return status.Errorf(codes.Unimplemented, "method Subscribe not implemented")
}
func (UnimplementedTraceSubscribeServer) mustEmbedUnimplementedTraceSubscribeServer() {}
func (UnimplementedTraceSubscribeServer) testEmbeddedByValue()                        {}

// UnsafeTraceSubscribeServer may be embedded to opt out of forward compatibility for this service.
// Use of this interface is not recommended, as added methods to TraceSubscribeServer will
// result in compilation errors.
type UnsafeTraceSubscribeServer interface {
	mustEmbedUnimplementedTraceSubscribeServer()
}

func RegisterTraceSubscribeServer(s grpc.ServiceRegistrar, srv TraceSubscribeServer) {
	// If the following call pancis, it indicates UnimplementedTraceSubscribeServer was
	// embedded by pointer and is nil.  This will cause panics if an
	// unimplemented method is ever invoked, so we test this at initialization
	// time to prevent it from happening at runtime later due to I/O.
	if t, ok := srv.(interface{ testEmbeddedByValue() }); ok {
		t.testEmbeddedByValue()
	}
	s.RegisterService(&TraceSubscribe_ServiceDesc, srv)
}

func _TraceSubscribe_Subscribe_Handler(srv interface{}, stream grpc.ServerStream) error {
	return srv.(TraceSubscribeServer).Subscribe(&grpc.GenericServerStream[SubscribeRequest, SubscribeResponse]{ServerStream: stream})
}

// This type alias is provided for backwards compatibility with existing code that references the prior non-generic stream type by name.
type TraceSubscribe_SubscribeServer = grpc.BidiStreamingServer[SubscribeRequest, SubscribeResponse]

// TraceSubscribe_ServiceDesc is the grpc.ServiceDesc for TraceSubscribe service.
// It's only intended for direct use with grpc.RegisterService,
// and not to be introspected or modified (even as a copy)
var TraceSubscribe_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "zeloscloud.trace.TraceSubscribe",
	HandlerType: (*TraceSubscribeServer)(nil),
	Methods:     []grpc.MethodDesc{},
	Streams: []grpc.StreamDesc{
		{
			StreamName:    "Subscribe",
			Handler:       _TraceSubscribe_Subscribe_Handler,
			ServerStreams: true,
			ClientStreams: true,
		},
	},
	Metadata: "zeloscloud/trace/subscribe.proto",
}
