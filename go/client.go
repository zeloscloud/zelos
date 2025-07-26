package zelos

import (
	"context"
	"fmt"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
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

// sendBatch sends a batch of messages (placeholder implementation)
func (c *TracePublishClient) sendBatch(messages []*IpcMessageWithId) error {
	// For now, just log that we're sending messages
	// In a full implementation, this would convert to protobuf and send via gRPC
	fmt.Printf("Sending batch of %d messages\n", len(messages))
	for _, msg := range messages {
		fmt.Printf("  - Message from %s (segment: %s)\n", msg.SourceName, msg.SegmentID.String())
	}
	return nil
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
