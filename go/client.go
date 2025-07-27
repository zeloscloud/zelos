package zelos

import (
	"context"
	"fmt"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	pb "github.com/zeloscloud/zelos/zeloscloud/trace"
)

// ConnectionStatus represents the connection status
type ConnectionStatus int

const (
	ConnectionStatusDisconnected ConnectionStatus = iota
	ConnectionStatusConnecting
	ConnectionStatusConnected
	ConnectionStatusError
)

// String returns the string representation of ConnectionStatus
func (cs ConnectionStatus) String() string {
	switch cs {
	case ConnectionStatusDisconnected:
		return "disconnected"
	case ConnectionStatusConnecting:
		return "connecting"
	case ConnectionStatusConnected:
		return "connected"
	case ConnectionStatusError:
		return "error"
	default:
		return "unknown"
	}
}

// TracePublishClientConfig holds configuration for the publish client
type TracePublishClientConfig struct {
	URL            string
	BatchSize      int
	BatchTimeout   time.Duration
	ReconnectDelay time.Duration
}

// DefaultTracePublishClientConfig returns default configuration
func DefaultTracePublishClientConfig() *TracePublishClientConfig {
	return &TracePublishClientConfig{
		URL:            "grpc://127.0.0.1:2300",
		BatchSize:      1000,
		BatchTimeout:   100 * time.Millisecond,
		ReconnectDelay: 1000 * time.Millisecond,
	}
}

// TracePublishClient represents a client for publishing trace data
type TracePublishClient struct {
	config           *TracePublishClientConfig
	connectionStatus ConnectionStatus
	statusMu         sync.RWMutex
	receiver         Receiver
	ctx              context.Context
	cancel           context.CancelFunc
	conn             *grpc.ClientConn
}

// NewTracePublishClient creates a new TracePublishClient
func NewTracePublishClient(ctx context.Context, receiver Receiver, config *TracePublishClientConfig) *TracePublishClient {
	clientCtx, cancel := context.WithCancel(ctx)

	client := &TracePublishClient{
		config:           config,
		connectionStatus: ConnectionStatusDisconnected,
		receiver:         receiver,
		ctx:              clientCtx,
		cancel:           cancel,
	}

	return client
}

// Run starts the client's main processing loop (similar to Rust's async task)
func (c *TracePublishClient) Run() error {
	for {
		select {
		case <-c.ctx.Done():
			return c.ctx.Err()
		default:
			if err := c.attemptConnection(); err != nil {
				c.setStatus(ConnectionStatusError)
				// Wait before reconnecting
				select {
				case <-c.ctx.Done():
					return c.ctx.Err()
				case <-time.After(c.config.ReconnectDelay):
					continue
				}
			}
		}
	}
}

// attemptConnection attempts to connect and process messages
func (c *TracePublishClient) attemptConnection() error {
	c.setStatus(ConnectionStatusConnecting)

	// Remove the grpc:// prefix if present
	addr := c.config.URL
	if len(addr) > 7 && addr[:7] == "grpc://" {
		addr = addr[7:]
	}

	conn, err := grpc.Dial(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return fmt.Errorf("failed to connect to %s: %w", addr, err)
	}
	defer conn.Close()

	c.setStatus(ConnectionStatusConnected)

	// Process messages until connection is lost or context is cancelled
	return c.processMessages(conn)
}

// processMessages processes incoming messages from the receiver
func (c *TracePublishClient) processMessages(conn *grpc.ClientConn) error {
	// Store connection for sendBatch to use
	c.conn = conn

	batch := make([]*IpcMessageWithId, 0, c.config.BatchSize)
	batchTimer := time.NewTimer(c.config.BatchTimeout)
	defer batchTimer.Stop()

	for {
		select {
		case <-c.ctx.Done():
			// Send any remaining messages in batch
			if len(batch) > 0 {
				c.sendBatch(batch)
			}
			return c.ctx.Err()

		case msg := <-c.receiver:
			batch = append(batch, msg)

			// Send batch if it's full
			if len(batch) >= c.config.BatchSize {
				if err := c.sendBatch(batch); err != nil {
					return err
				}
				batch = batch[:0] // Reset batch
				batchTimer.Reset(c.config.BatchTimeout)
			}

		case <-batchTimer.C:
			// Send batch due to timeout
			if len(batch) > 0 {
				if err := c.sendBatch(batch); err != nil {
					return err
				}
				batch = batch[:0] // Reset batch
			}
			batchTimer.Reset(c.config.BatchTimeout)
		}
	}
}

// sendBatch sends a batch of messages via gRPC
func (c *TracePublishClient) sendBatch(messages []*IpcMessageWithId) error {
	if c.conn == nil {
		return fmt.Errorf("no active connection")
	}

	// Convert internal messages to protobuf
	var protoMessages []*pb.TraceMessage
	for _, msg := range messages {
		protoMsg, err := convertToProto(msg)
		if err != nil {
			return fmt.Errorf("failed to convert message: %w", err)
		}
		protoMessages = append(protoMessages, protoMsg)
	}

	// Create gRPC client
	client := pb.NewTracePublishClient(c.conn)

	// Send via gRPC stream
	stream, err := client.Publish(c.ctx)
	if err != nil {
		return fmt.Errorf("failed to create publish stream: %w", err)
	}

	// Send the batch
	req := &pb.PublishRequest{
		TraceMessages: protoMessages,
	}

	if err := stream.Send(req); err != nil {
		return fmt.Errorf("failed to send batch: %w", err)
	}

	// Close the send side and wait for server response
	if err := stream.CloseSend(); err != nil {
		return fmt.Errorf("failed to close send: %w", err)
	}

	// Handle response (optional - just consume it)
	for {
		_, err := stream.Recv()
		if err != nil {
			// Normal end of stream
			break
		}
	}

	return nil
}

// convertToProto converts an IpcMessageWithId to protobuf TraceMessage
func convertToProto(msg *IpcMessageWithId) (*pb.TraceMessage, error) {
	segmentIdBytes := msg.SegmentID[:]

	traceMsg := &pb.TraceMessage{
		SegmentId:  segmentIdBytes,
		SourceName: msg.SourceName,
	}

	switch m := msg.Message.(type) {
	case *IpcMessageEvent:
		traceMsg.Msg = &pb.TraceMessage_Event{
			Event: convertEventToProto(m.TraceEvent),
		}
	case *IpcMessageSegmentStart:
		traceMsg.Msg = &pb.TraceMessage_SegmentStart{
			SegmentStart: convertSegmentStartToProto(m.TraceSegmentStart),
		}
	case *IpcMessageSegmentEnd:
		traceMsg.Msg = &pb.TraceMessage_SegmentEnd{
			SegmentEnd: convertSegmentEndToProto(m.TraceSegmentEnd),
		}
	case *IpcMessageEventSchema:
		traceMsg.Msg = &pb.TraceMessage_EventSchema{
			EventSchema: convertEventSchemaToProto(m.TraceEventSchema),
		}
	default:
		return nil, fmt.Errorf("unknown message type: %T", m)
	}

	return traceMsg, nil
}

// convertEventToProto converts TraceEvent to protobuf
func convertEventToProto(event *TraceEvent) *pb.TraceEvent {
	var fields []*pb.TraceEventFieldEntry
	for k, v := range event.Fields {
		fields = append(fields, &pb.TraceEventFieldEntry{
			Name:  k,
			Value: convertValueToProto(v),
		})
	}

	return &pb.TraceEvent{
		TimeNs: event.TimeNs,
		Name:   event.Name,
		Fields: fields,
	}
}

// convertSegmentStartToProto converts TraceSegmentStart to protobuf
func convertSegmentStartToProto(start *TraceSegmentStart) *pb.TraceSegmentStart {
	return &pb.TraceSegmentStart{
		TimeNs:     start.TimeNs,
		SourceName: start.SourceName,
	}
}

// convertSegmentEndToProto converts TraceSegmentEnd to protobuf
func convertSegmentEndToProto(end *TraceSegmentEnd) *pb.TraceSegmentEnd {
	return &pb.TraceSegmentEnd{
		TimeNs: end.TimeNs,
	}
}

// convertEventSchemaToProto converts TraceEventSchema to protobuf
func convertEventSchemaToProto(schema *TraceEventSchema) *pb.TraceEventSchema {
	var fields []*pb.TraceEventFieldMetadata
	for _, field := range schema.Fields {
		fields = append(fields, convertFieldMetadataToProto(field))
	}

	return &pb.TraceEventSchema{
		Name:   schema.Name,
		Fields: fields,
	}
}

// convertFieldMetadataToProto converts TraceEventFieldMetadata to protobuf
func convertFieldMetadataToProto(field *TraceEventFieldMetadata) *pb.TraceEventFieldMetadata {
	return &pb.TraceEventFieldMetadata{
		Name:     field.Name,
		DataType: convertDataTypeToProto(field.DataType),
		Unit:     field.Unit,
	}
}

// convertDataTypeToProto converts DataType to protobuf
func convertDataTypeToProto(dt DataType) pb.DataType {
	switch dt {
	case DataTypeInt8:
		return pb.DataType_DATA_TYPE_INT8
	case DataTypeInt16:
		return pb.DataType_DATA_TYPE_INT16
	case DataTypeInt32:
		return pb.DataType_DATA_TYPE_INT32
	case DataTypeInt64:
		return pb.DataType_DATA_TYPE_INT64
	case DataTypeUint8:
		return pb.DataType_DATA_TYPE_UINT8
	case DataTypeUint16:
		return pb.DataType_DATA_TYPE_UINT16
	case DataTypeUint32:
		return pb.DataType_DATA_TYPE_UINT32
	case DataTypeUint64:
		return pb.DataType_DATA_TYPE_UINT64
	case DataTypeFloat32:
		return pb.DataType_DATA_TYPE_FLOAT32
	case DataTypeFloat64:
		return pb.DataType_DATA_TYPE_FLOAT64
	case DataTypeString:
		return pb.DataType_DATA_TYPE_STRING
	case DataTypeBinary:
		return pb.DataType_DATA_TYPE_BINARY
	case DataTypeBoolean:
		return pb.DataType_DATA_TYPE_BOOL
	case DataTypeTimestampNs:
		return pb.DataType_DATA_TYPE_TIMESTAMP_NS
	default:
		return pb.DataType_DATA_TYPE_UNSPECIFIED
	}
}

// convertValueToProto converts Value to protobuf
func convertValueToProto(v *Value) *pb.Value {
	switch val := v.value.(type) {
	case int8:
		return &pb.Value{Value: &pb.Value_Int8{Int8: int64(val)}}
	case int16:
		return &pb.Value{Value: &pb.Value_Int16{Int16: int64(val)}}
	case int32:
		return &pb.Value{Value: &pb.Value_Int32{Int32: int64(val)}}
	case int64:
		return &pb.Value{Value: &pb.Value_Int64{Int64: val}}
	case uint8:
		return &pb.Value{Value: &pb.Value_Uint8{Uint8: uint64(val)}}
	case uint16:
		return &pb.Value{Value: &pb.Value_Uint16{Uint16: uint64(val)}}
	case uint32:
		return &pb.Value{Value: &pb.Value_Uint32{Uint32: uint64(val)}}
	case uint64:
		return &pb.Value{Value: &pb.Value_Uint64{Uint64: val}}
	case float32:
		return &pb.Value{Value: &pb.Value_Float32{Float32: val}}
	case float64:
		return &pb.Value{Value: &pb.Value_Float64{Float64: val}}
	case string:
		return &pb.Value{Value: &pb.Value_String_{String_: val}}
	case []byte:
		return &pb.Value{Value: &pb.Value_Binary{Binary: val}}
	case bool:
		return &pb.Value{Value: &pb.Value_Bool{Bool: val}}
	default:
		// Handle timestamp as int64
		if v.DataType() == DataTypeTimestampNs {
			if tsVal, ok := val.(int64); ok {
				return &pb.Value{Value: &pb.Value_TimestampNs{TimestampNs: tsVal}}
			}
		}
		// Return empty value for unknown types
		return &pb.Value{}
	}
}

// setStatus sets the connection status thread-safely
func (c *TracePublishClient) setStatus(status ConnectionStatus) {
	c.statusMu.Lock()
	defer c.statusMu.Unlock()
	c.connectionStatus = status
}

// GetConnectionStatus returns the current connection status
func (c *TracePublishClient) GetConnectionStatus() ConnectionStatus {
	c.statusMu.RLock()
	defer c.statusMu.RUnlock()
	return c.connectionStatus
}

// WaitUntilConnected waits until the client is connected or timeout expires
func (c *TracePublishClient) WaitUntilConnected(timeout time.Duration) error {
	deadline := time.Now().Add(timeout)

	for time.Now().Before(deadline) {
		if c.GetConnectionStatus() == ConnectionStatusConnected {
			return nil
		}
		time.Sleep(100 * time.Millisecond)
	}

	return fmt.Errorf("timeout waiting for connection after %v", timeout)
}

// IsConnected returns whether the client is currently connected
func (c *TracePublishClient) IsConnected() bool {
	return c.GetConnectionStatus() == ConnectionStatusConnected
}

// Close closes the client and cancels all operations
func (c *TracePublishClient) Close() error {
	c.cancel()
	return nil
}
