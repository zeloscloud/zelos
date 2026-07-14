package zelos

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"sync"
	"time"

	actionspb "github.com/zeloscloud/zelos/go/zeloscloud/actions"
	"google.golang.org/grpc"
)

// ActionsClientConfig holds configuration for the actions client.
type ActionsClientConfig struct {
	// URL of the agent, optionally prefixed with "grpc://".
	URL string
	// ReconnectDelay is how long to wait before reconnecting after a stream
	// error or close.
	ReconnectDelay time.Duration
	// ListInterval is how often to unconditionally push the current action list
	// so dynamically registered actions are discovered.
	ListInterval time.Duration
	// DialOptions are appended to DefaultDialOptions() when dialing, so the
	// default max message size and keepalive settings are preserved. This is
	// primarily an injection point for tests (e.g. grpc.WithContextDialer over
	// an in-memory listener); later options take precedence over earlier ones.
	DialOptions []grpc.DialOption
}

// Default reconnect delay and list interval, used both by
// DefaultActionsClientConfig and to fill zero-valued fields in NewActionsClient.
const (
	defaultReconnectDelay = 5 * time.Second
	defaultListInterval   = 5 * time.Second
)

// DefaultActionsClientConfig returns the default configuration. The URL is read
// from the ZELOS_URL environment variable, falling back to
// grpc://127.0.0.1:2300.
func DefaultActionsClientConfig() ActionsClientConfig {
	url := os.Getenv("ZELOS_URL")
	if url == "" {
		url = "grpc://127.0.0.1:2300"
	}
	return ActionsClientConfig{
		URL:            url,
		ReconnectDelay: defaultReconnectDelay,
		ListInterval:   defaultListInterval,
	}
}

// ActionsClient serves provider-side actions to the agent over a bidirectional
// Actions stream, reconnecting automatically if the connection is lost.
type ActionsClient struct {
	config      ActionsClientConfig
	serviceName string
	registry    *ActionsRegistry

	connectionStatus ConnectionStatus
	statusMu         sync.RWMutex

	startTime time.Time

	ctx    context.Context
	cancel context.CancelFunc
}

// NewActionsClient creates a new ActionsClient. Call Run to start serving.
// Zero-valued ReconnectDelay and ListInterval are replaced with sensible
// defaults so there is no time.NewTicker(0) panic or tight reconnect loop;
// explicit positive values are preserved.
func NewActionsClient(ctx context.Context, serviceName string, registry *ActionsRegistry, config ActionsClientConfig) *ActionsClient {
	if registry == nil {
		registry = NewActionsRegistry()
	}
	if config.ReconnectDelay <= 0 {
		config.ReconnectDelay = defaultReconnectDelay
	}
	if config.ListInterval <= 0 {
		config.ListInterval = defaultListInterval
	}
	clientCtx, cancel := context.WithCancel(ctx)
	return &ActionsClient{
		config:           config,
		serviceName:      serviceName,
		registry:         registry,
		connectionStatus: ConnectionStatusDisconnected,
		startTime:        time.Now(),
		ctx:              clientCtx,
		cancel:           cancel,
	}
}

// Run serves actions until the context is cancelled, reconnecting after
// ReconnectDelay whenever the stream errors or closes.
func (c *ActionsClient) Run() error {
	for {
		select {
		case <-c.ctx.Done():
			return c.ctx.Err()
		default:
			if err := c.attemptConnection(); err != nil {
				c.setStatus(ConnectionStatusError)
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

// attemptConnection dials the agent and serves a single actions stream.
func (c *ActionsClient) attemptConnection() error {
	c.setStatus(ConnectionStatusConnecting)

	// Remove the grpc:// prefix if present (matches TracePublishClient).
	addr := c.config.URL
	if len(addr) > 7 && addr[:7] == "grpc://" {
		addr = addr[7:]
	}

	// Start from the project-wide defaults (max message size, keepalive) and
	// append any custom options so callers extend rather than replace them.
	dialOpts := DefaultDialOptions()
	dialOpts = append(dialOpts, c.config.DialOptions...)

	conn, err := grpc.Dial(addr, dialOpts...)
	if err != nil {
		return fmt.Errorf("failed to connect to %s: %w", addr, err)
	}
	defer conn.Close()

	return c.processStream(conn)
}

// processStream opens the bidirectional stream, sends the handshake, and serves
// server requests until the stream errors or closes.
func (c *ActionsClient) processStream(conn *grpc.ClientConn) error {
	client := actionspb.NewActionsClient(conn)

	// streamCtx bounds the lifetime of the stream and its send/list-ticker
	// goroutines. The stream (and therefore stream.Recv) is derived from it, so
	// cancelling streamCtx when a Send fails unblocks Recv and returns the loop
	// to attempt reconnection.
	streamCtx, streamCancel := context.WithCancel(c.ctx)
	defer streamCancel()

	stream, err := client.Actions(streamCtx)
	if err != nil {
		return fmt.Errorf("failed to start actions stream: %w", err)
	}

	c.setStatus(ConnectionStatusConnected)

	// gRPC client streams are not safe for concurrent Send, so a single
	// goroutine owns stream.Send, fed by sendCh (mirrors the Rust mpsc design).
	sendCh := make(chan *actionspb.ActionsRequest, 64)
	go func() {
		for {
			select {
			case <-streamCtx.Done():
				return
			case req := <-sendCh:
				if err := stream.Send(req); err != nil {
					streamCancel()
					return
				}
			}
		}
	}()

	// Handshake must be the first message on the stream.
	c.send(streamCtx, sendCh, &actionspb.ActionsRequest{
		SequenceNumber: 0,
		Name:           c.serviceName,
	})

	// Unconditionally push the action list on an interval so dynamically
	// registered actions are discovered.
	go func() {
		ticker := time.NewTicker(c.config.ListInterval)
		defer ticker.Stop()
		for {
			select {
			case <-streamCtx.Done():
				return
			case <-ticker.C:
				c.send(streamCtx, sendCh, c.listResponse(0))
			}
		}
	}()

	for {
		resp, err := stream.Recv()
		if err != nil {
			return fmt.Errorf("actions stream closed: %w", err)
		}
		c.handleServerMessage(streamCtx, sendCh, resp)
	}
}

// send delivers req to the send goroutine, giving up if the stream is gone.
func (c *ActionsClient) send(ctx context.Context, sendCh chan<- *actionspb.ActionsRequest, req *actionspb.ActionsRequest) {
	select {
	case sendCh <- req:
	case <-ctx.Done():
	}
}

// handleServerMessage dispatches a single server request. Execute requests run
// in their own goroutine so the stream loop stays responsive.
func (c *ActionsClient) handleServerMessage(ctx context.Context, sendCh chan<- *actionspb.ActionsRequest, resp *actionspb.ActionsResponse) {
	seq := resp.GetSequenceNumber()
	switch msg := resp.GetMsg().(type) {
	case *actionspb.ActionsResponse_List:
		c.send(ctx, sendCh, c.listResponse(seq))
	case *actionspb.ActionsResponse_Execute:
		go c.handleExecute(ctx, sendCh, seq, msg.Execute)
	case *actionspb.ActionsResponse_Status:
		c.send(ctx, sendCh, c.statusResponse(seq))
	case *actionspb.ActionsResponse_Schema:
		c.send(ctx, sendCh, c.schemaResponse(seq, msg.Schema))
	default:
		// Empty or unknown message; nothing to do.
	}
}

// listResponse builds a ListResponse echoing the given sequence number.
func (c *ActionsClient) listResponse(seq uint64) *actionspb.ActionsRequest {
	return &actionspb.ActionsRequest{
		SequenceNumber: seq,
		Name:           "client",
		Msg: &actionspb.ActionsRequest_List{
			List: &actionspb.ListResponse{Actions: c.registry.List()},
		},
	}
}

// statusResponse builds a StatusResponse echoing the given sequence number.
// uptime_seconds is the elapsed time since the client was created.
func (c *ActionsClient) statusResponse(seq uint64) *actionspb.ActionsRequest {
	statusJSON, _ := json.Marshal(struct {
		UptimeSeconds int64 `json:"uptime_seconds"`
		ActionsCount  int   `json:"actions_count"`
	}{
		UptimeSeconds: int64(time.Since(c.startTime).Seconds()),
		ActionsCount:  len(c.registry.List()),
	})
	status := string(statusJSON)
	return &actionspb.ActionsRequest{
		SequenceNumber: seq,
		Name:           c.serviceName,
		Msg: &actionspb.ActionsRequest_Status{
			Status: &actionspb.StatusResponse{Json: status},
		},
	}
}

// schemaResponse builds a SchemaResponse for the requested action, echoing the
// given sequence number. Schema strings are passed through verbatim.
func (c *ActionsClient) schemaResponse(seq uint64, req *actionspb.SchemaRequest) *actionspb.ActionsRequest {
	resp := &actionspb.SchemaResponse{Action: req.GetAction()}

	action, ok := c.registry.Get(req.GetAction())
	if !ok {
		resp.ActionSchema = errorJSON("action not found: " + req.GetAction())
		resp.UiSchema = "{}"
	} else {
		var currentValues json.RawMessage
		if req.GetCurrentValues() != "" {
			currentValues = json.RawMessage(req.GetCurrentValues())
		}
		actionSchema, uiSchema, err := action.Schema(currentValues)
		if err != nil {
			resp.ActionSchema = errorJSON(err.Error())
			resp.UiSchema = "{}"
		} else {
			resp.ActionSchema = actionSchema
			resp.UiSchema = uiSchema
			if d := action.DefaultTimeout(); d > 0 {
				ms := uint32(d.Milliseconds())
				resp.DefaultTimeoutMs = &ms
			}
		}
	}

	return &actionspb.ActionsRequest{
		SequenceNumber: seq,
		Name:           "client",
		Msg:            &actionspb.ActionsRequest_Schema{Schema: resp},
	}
}

// handleExecute runs an action, applying the resolved timeout, and sends the
// response. It runs in its own goroutine.
func (c *ActionsClient) handleExecute(ctx context.Context, sendCh chan<- *actionspb.ActionsRequest, seq uint64, req *actionspb.ExecuteRequest) {
	action, ok := c.registry.Get(req.GetAction())
	if !ok {
		c.send(ctx, sendCh, executeError(seq, req.GetAction(), "action not found: "+req.GetAction()))
		return
	}

	// Resolve timeout: request timeout_ms > action default > none. A zero
	// timeout_ms is treated as absent (matching the Rust SDK), falling back to
	// the action's default timeout.
	var timeout time.Duration
	if req.TimeoutMs != nil && req.GetTimeoutMs() > 0 {
		timeout = time.Duration(req.GetTimeoutMs()) * time.Millisecond
	} else {
		timeout = action.DefaultTimeout()
	}

	execCtx := ctx
	if timeout > 0 {
		var cancel context.CancelFunc
		execCtx, cancel = context.WithTimeout(ctx, timeout)
		defer cancel()
	}

	params := json.RawMessage(req.GetParams())
	resultCh := make(chan executeOutcome, 1)
	go func() {
		res, err := action.Execute(execCtx, params)
		resultCh <- executeOutcome{result: res, err: err}
	}()

	if timeout > 0 {
		select {
		case <-execCtx.Done():
			if execCtx.Err() == context.DeadlineExceeded {
				msg := errorJSON(fmt.Sprintf("Action timed out after %dms", timeout.Milliseconds()))
				c.send(ctx, sendCh, executeResult(seq, req.GetAction(), msg, ExecuteStatusError))
			}
			return
		case outcome := <-resultCh:
			if execCtx.Err() == context.DeadlineExceeded {
				msg := errorJSON(fmt.Sprintf("Action timed out after %dms", timeout.Milliseconds()))
				c.send(ctx, sendCh, executeResult(seq, req.GetAction(), msg, ExecuteStatusError))
			} else {
				c.sendExecuteOutcome(ctx, sendCh, seq, req.GetAction(), outcome)
			}
			return
		}
	}

	select {
	case outcome := <-resultCh:
		c.sendExecuteOutcome(ctx, sendCh, seq, req.GetAction(), outcome)
	case <-ctx.Done():
	}
}

// executeOutcome bundles the result and error of an action execution.
type executeOutcome struct {
	result *ActionResult
	err    error
}

// sendExecuteOutcome marshals an execution outcome into an ExecuteResponse and
// sends it.
func (c *ActionsClient) sendExecuteOutcome(ctx context.Context, sendCh chan<- *actionspb.ActionsRequest, seq uint64, action string, outcome executeOutcome) {
	if outcome.err != nil {
		c.send(ctx, sendCh, executeError(seq, action, outcome.err.Error()))
		return
	}
	if outcome.result == nil {
		c.send(ctx, sendCh, executeError(seq, action, "action returned nil result"))
		return
	}
	valueJSON, err := json.Marshal(outcome.result.Value)
	if err != nil {
		c.send(ctx, sendCh, executeError(seq, action, "failed to serialize result: "+err.Error()))
		return
	}
	c.send(ctx, sendCh, executeResult(seq, action, string(valueJSON), outcome.result.Status))
}

// executeResult builds an ExecuteResponse with the given result JSON and status.
func executeResult(seq uint64, action, result string, status ExecuteStatus) *actionspb.ActionsRequest {
	st := status.proto()
	return &actionspb.ActionsRequest{
		SequenceNumber: seq,
		Name:           "client",
		Msg: &actionspb.ActionsRequest_Execute{
			Execute: &actionspb.ExecuteResponse{
				Action: action,
				Result: result,
				Status: &st,
			},
		},
	}
}

// executeError builds an ExecuteResponse carrying an error result and status
// ERROR.
func executeError(seq uint64, action, errMsg string) *actionspb.ActionsRequest {
	return executeResult(seq, action, errorJSON(errMsg), ExecuteStatusError)
}

// errorJSON builds a JSON object of the form {"error": <msg>} using encoding/json
// so quotes, backslashes, and newlines in msg cannot break the payload.
func errorJSON(msg string) string {
	b, _ := json.Marshal(struct {
		Error string `json:"error"`
	}{Error: msg})
	return string(b)
}

// setStatus sets the connection status thread-safely.
func (c *ActionsClient) setStatus(status ConnectionStatus) {
	c.statusMu.Lock()
	defer c.statusMu.Unlock()
	c.connectionStatus = status
}

// GetConnectionStatus returns the current connection status.
func (c *ActionsClient) GetConnectionStatus() ConnectionStatus {
	c.statusMu.RLock()
	defer c.statusMu.RUnlock()
	return c.connectionStatus
}

// WaitUntilConnected waits until the client is connected or timeout expires.
func (c *ActionsClient) WaitUntilConnected(timeout time.Duration) error {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		if c.GetConnectionStatus() == ConnectionStatusConnected {
			return nil
		}
		time.Sleep(100 * time.Millisecond)
	}
	return fmt.Errorf("timeout waiting for connection after %v", timeout)
}

// IsConnected returns whether the client is currently connected.
func (c *ActionsClient) IsConnected() bool {
	return c.GetConnectionStatus() == ConnectionStatusConnected
}

// Close cancels the client and stops Run.
func (c *ActionsClient) Close() error {
	c.cancel()
	return nil
}
