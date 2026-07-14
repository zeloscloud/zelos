package zelos

import (
	"context"
	"encoding/json"
	"errors"
	"net"
	"reflect"
	"testing"
	"time"

	actionspb "github.com/zeloscloud/zelos/go/zeloscloud/actions"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
)

// stubActionsServer is a test-only Actions service. Its Actions handler hands
// the live stream to the test via the streams channel and stays alive until the
// stream context is done, so the test can drive requests and read responses
// directly.
type stubActionsServer struct {
	actionspb.UnimplementedActionsServer
	streams chan actionspb.Actions_ActionsServer
}

func (s *stubActionsServer) Actions(stream actionspb.Actions_ActionsServer) error {
	s.streams <- stream
	<-stream.Context().Done()
	return stream.Context().Err()
}

// newTestClient wires an ActionsClient to an in-memory bufconn server and
// returns the active server-side stream plus a cleanup func.
func newTestClient(t *testing.T, serviceName string, registry *ActionsRegistry) (actionspb.Actions_ActionsServer, func()) {
	t.Helper()

	lis := bufconn.Listen(1024 * 1024)
	grpcServer := grpc.NewServer()
	srv := &stubActionsServer{streams: make(chan actionspb.Actions_ActionsServer, 1)}
	actionspb.RegisterActionsServer(grpcServer, srv)
	go func() { _ = grpcServer.Serve(lis) }()

	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	config := ActionsClientConfig{
		URL:            "grpc://bufnet",
		ReconnectDelay: 100 * time.Millisecond,
		ListInterval:   time.Hour, // avoid interference with assertions
		DialOptions: []grpc.DialOption{
			grpc.WithContextDialer(dialer),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
		},
	}

	ctx, cancel := context.WithCancel(context.Background())
	client := NewActionsClient(ctx, serviceName, registry, config)
	go func() { _ = client.Run() }()

	var stream actionspb.Actions_ActionsServer
	select {
	case stream = <-srv.streams:
	case <-time.After(3 * time.Second):
		cancel()
		grpcServer.Stop()
		t.Fatal("timed out waiting for actions stream")
	}

	cleanup := func() {
		cancel()
		grpcServer.Stop()
	}
	return stream, cleanup
}

// recvWithin reads one message from the stream, failing if it takes too long.
func recvWithin(t *testing.T, stream actionspb.Actions_ActionsServer, d time.Duration) *actionspb.ActionsRequest {
	t.Helper()
	type result struct {
		req *actionspb.ActionsRequest
		err error
	}
	ch := make(chan result, 1)
	go func() {
		req, err := stream.Recv()
		ch <- result{req, err}
	}()
	select {
	case r := <-ch:
		if r.err != nil {
			t.Fatalf("stream.Recv error: %v", r.err)
		}
		return r.req
	case <-time.After(d):
		t.Fatalf("timed out waiting for message after %v", d)
		return nil
	}
}

func TestActionsHandshake(t *testing.T) {
	registry := NewActionsRegistry()
	stream, cleanup := newTestClient(t, "my-service", registry)
	defer cleanup()

	hs := recvWithin(t, stream, 2*time.Second)
	if hs.GetSequenceNumber() != 0 {
		t.Errorf("handshake sequence = %d, want 0", hs.GetSequenceNumber())
	}
	if hs.GetName() != "my-service" {
		t.Errorf("handshake name = %q, want %q", hs.GetName(), "my-service")
	}
	if hs.GetMsg() != nil {
		t.Errorf("handshake msg = %v, want nil", hs.GetMsg())
	}
}

func TestActionsListRequest(t *testing.T) {
	registry := NewActionsRegistry()
	registry.Register("b/action", NewAction("{}", nil))
	registry.Register("a/action", NewAction("{}", nil))
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 7,
		Msg:            &actionspb.ActionsResponse_List{List: &actionspb.ListRequest{}},
	}); err != nil {
		t.Fatalf("send list request: %v", err)
	}

	resp := recvWithin(t, stream, 2*time.Second)
	if resp.GetSequenceNumber() != 7 {
		t.Errorf("list response sequence = %d, want 7", resp.GetSequenceNumber())
	}
	got := resp.GetList().GetActions()
	want := []string{"a/action", "b/action"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("list actions = %v, want %v", got, want)
	}
}

func TestActionsExecutePass(t *testing.T) {
	registry := NewActionsRegistry()
	registry.Register("echo", NewAction("{}", func(_ context.Context, params json.RawMessage) (*ActionResult, error) {
		return ActionResultPass(map[string]any{"ok": true}), nil
	}))
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 3,
		Msg: &actionspb.ActionsResponse_Execute{
			Execute: &actionspb.ExecuteRequest{Action: "echo", Params: "{}"},
		},
	}); err != nil {
		t.Fatalf("send execute request: %v", err)
	}

	resp := recvWithin(t, stream, 2*time.Second)
	if resp.GetSequenceNumber() != 3 {
		t.Errorf("execute response sequence = %d, want 3", resp.GetSequenceNumber())
	}
	exec := resp.GetExecute()
	if exec.GetStatus() != actionspb.ExecuteStatus_PASS {
		t.Errorf("status = %v, want PASS", exec.GetStatus())
	}
	if exec.GetResult() != `{"ok":true}` {
		t.Errorf("result = %q, want %q", exec.GetResult(), `{"ok":true}`)
	}
}

func TestActionsExecuteFailure(t *testing.T) {
	registry := NewActionsRegistry()
	registry.Register("boom", NewAction("{}", func(context.Context, json.RawMessage) (*ActionResult, error) {
		return nil, errors.New("kaboom")
	}))
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 9,
		Msg: &actionspb.ActionsResponse_Execute{
			Execute: &actionspb.ExecuteRequest{Action: "boom", Params: "{}"},
		},
	}); err != nil {
		t.Fatalf("send execute request: %v", err)
	}

	resp := recvWithin(t, stream, 2*time.Second)
	exec := resp.GetExecute()
	if exec.GetStatus() != actionspb.ExecuteStatus_ERROR {
		t.Errorf("status = %v, want ERROR", exec.GetStatus())
	}
	if exec.GetResult() != `{"error":"kaboom"}` {
		t.Errorf("result = %q, want %q", exec.GetResult(), `{"error":"kaboom"}`)
	}
}

func TestActionsExecuteTimeout(t *testing.T) {
	registry := NewActionsRegistry()
	workerDone := make(chan struct{})
	registry.Register("slow", NewAction("{}", func(ctx context.Context, _ json.RawMessage) (*ActionResult, error) {
		defer close(workerDone)
		<-ctx.Done()
		return nil, ctx.Err()
	}))
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	timeoutMs := uint32(50)
	start := time.Now()
	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 5,
		Msg: &actionspb.ActionsResponse_Execute{
			Execute: &actionspb.ExecuteRequest{Action: "slow", Params: "{}", TimeoutMs: &timeoutMs},
		},
	}); err != nil {
		t.Fatalf("send execute request: %v", err)
	}

	resp := recvWithin(t, stream, 2*time.Second)
	if elapsed := time.Since(start); elapsed > 2*time.Second {
		t.Errorf("timeout response took %v, expected prompt", elapsed)
	}
	exec := resp.GetExecute()
	if exec.GetStatus() != actionspb.ExecuteStatus_ERROR {
		t.Errorf("status = %v, want ERROR", exec.GetStatus())
	}
	if exec.GetResult() != `{"error":"Action timed out after 50ms"}` {
		t.Errorf("result = %q, want timeout message", exec.GetResult())
	}
	select {
	case <-workerDone:
	case <-time.After(time.Second):
		t.Fatal("action did not stop after its context was cancelled")
	}
}

func TestActionsConcurrentExecute(t *testing.T) {
	registry := NewActionsRegistry()
	registry.Register("slow", NewAction("{}", func(context.Context, json.RawMessage) (*ActionResult, error) {
		time.Sleep(500 * time.Millisecond)
		return ActionResultPass("slow"), nil
	}))
	registry.Register("fast", NewAction("{}", func(context.Context, json.RawMessage) (*ActionResult, error) {
		return ActionResultPass("fast"), nil
	}))
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	// Send slow first, then fast. The stream must stay responsive so fast's
	// response arrives before slow's.
	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 1,
		Msg:            &actionspb.ActionsResponse_Execute{Execute: &actionspb.ExecuteRequest{Action: "slow", Params: "{}"}},
	}); err != nil {
		t.Fatalf("send slow: %v", err)
	}
	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 2,
		Msg:            &actionspb.ActionsResponse_Execute{Execute: &actionspb.ExecuteRequest{Action: "fast", Params: "{}"}},
	}); err != nil {
		t.Fatalf("send fast: %v", err)
	}

	first := recvWithin(t, stream, 2*time.Second)
	if first.GetSequenceNumber() != 2 {
		t.Errorf("first response sequence = %d, want 2 (fast should finish first)", first.GetSequenceNumber())
	}
	second := recvWithin(t, stream, 2*time.Second)
	if second.GetSequenceNumber() != 1 {
		t.Errorf("second response sequence = %d, want 1 (slow)", second.GetSequenceNumber())
	}
}

func TestActionsSchemaPassthrough(t *testing.T) {
	// Deliberately non-alphabetical key order to prove no reordering.
	const schema = `{"zebra":1,"apple":2,"properties":{"z":{},"a":{}}}`
	const uiSchema = `{"ui:order":["z","a"]}`

	registry := NewActionsRegistry()
	registry.Register("shaped", schemaAction{schema: schema, uiSchema: uiSchema})
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 11,
		Msg:            &actionspb.ActionsResponse_Schema{Schema: &actionspb.SchemaRequest{Action: "shaped"}},
	}); err != nil {
		t.Fatalf("send schema request: %v", err)
	}

	resp := recvWithin(t, stream, 2*time.Second)
	if resp.GetSequenceNumber() != 11 {
		t.Errorf("schema response sequence = %d, want 11", resp.GetSequenceNumber())
	}
	sc := resp.GetSchema()
	if sc.GetActionSchema() != schema {
		t.Errorf("action schema = %q, want verbatim %q", sc.GetActionSchema(), schema)
	}
	if sc.GetUiSchema() != uiSchema {
		t.Errorf("ui schema = %q, want verbatim %q", sc.GetUiSchema(), uiSchema)
	}
}

// schemaAction is a test Action with fixed schema strings.
type schemaAction struct {
	schema   string
	uiSchema string
}

func (a schemaAction) Execute(context.Context, json.RawMessage) (*ActionResult, error) {
	return ActionResultDone(nil), nil
}

func (a schemaAction) Schema(json.RawMessage) (string, string, error) {
	return a.schema, a.uiSchema, nil
}

func (a schemaAction) DefaultTimeout() time.Duration { return 0 }

func TestActionsRegistry(t *testing.T) {
	registry := NewActionsRegistry()
	a1 := NewAction("{}", nil)
	a2 := NewAction("{}", nil)
	registry.Register("charlie", a1)
	registry.Register("alpha", a2)
	registry.Register("bravo", a1)

	if _, ok := registry.Get("alpha"); !ok {
		t.Error("expected to find alpha")
	}
	if _, ok := registry.Get("missing"); ok {
		t.Error("did not expect to find missing")
	}
	got := registry.List()
	want := []string{"alpha", "bravo", "charlie"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("List() = %v, want %v", got, want)
	}
}

// countingActionsServer records how many times a stream is opened and
// terminates each stream immediately, forcing the client to reconnect.
type countingActionsServer struct {
	actionspb.UnimplementedActionsServer
	opened chan struct{}
}

func (s *countingActionsServer) Actions(actionspb.Actions_ActionsServer) error {
	s.opened <- struct{}{}
	return nil // terminate the stream server-side
}

func TestActionsReconnectsAfterStreamTermination(t *testing.T) {
	lis := bufconn.Listen(1024 * 1024)
	grpcServer := grpc.NewServer()
	srv := &countingActionsServer{opened: make(chan struct{}, 16)}
	actionspb.RegisterActionsServer(grpcServer, srv)
	go func() { _ = grpcServer.Serve(lis) }()
	defer grpcServer.Stop()

	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	config := ActionsClientConfig{
		URL:            "grpc://bufnet",
		ReconnectDelay: 50 * time.Millisecond,
		ListInterval:   time.Hour,
		DialOptions: []grpc.DialOption{
			grpc.WithContextDialer(dialer),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
		},
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	client := NewActionsClient(ctx, "svc", NewActionsRegistry(), config)
	go func() { _ = client.Run() }()

	// Each terminated stream must be followed by a fresh connection attempt.
	for i := 0; i < 2; i++ {
		select {
		case <-srv.opened:
		case <-time.After(3 * time.Second):
			t.Fatalf("expected stream open #%d (reconnection did not happen)", i+1)
		}
	}
}

func TestExecuteErrorJSONHostileStrings(t *testing.T) {
	hostile := "boom \"quote\" \\backslash\\ \nnewline\ttab"
	got := errorJSON(hostile)
	if !json.Valid([]byte(got)) {
		t.Fatalf("errorJSON produced invalid JSON: %s", got)
	}
	var decoded struct {
		Error string `json:"error"`
	}
	if err := json.Unmarshal([]byte(got), &decoded); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}
	if decoded.Error != hostile {
		t.Errorf("round-trip mismatch: got %q, want %q", decoded.Error, hostile)
	}
}

func TestActionsExecuteHostileErrorMessage(t *testing.T) {
	hostile := "kaboom \"q\" \\b\\ \nline"
	registry := NewActionsRegistry()
	registry.Register("boom", NewAction("{}", func(context.Context, json.RawMessage) (*ActionResult, error) {
		return nil, errors.New(hostile)
	}))
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 1,
		Msg:            &actionspb.ActionsResponse_Execute{Execute: &actionspb.ExecuteRequest{Action: "boom", Params: "{}"}},
	}); err != nil {
		t.Fatalf("send execute request: %v", err)
	}

	resp := recvWithin(t, stream, 2*time.Second)
	result := resp.GetExecute().GetResult()
	if !json.Valid([]byte(result)) {
		t.Fatalf("execute error result is not valid JSON: %s", result)
	}
	var decoded struct {
		Error string `json:"error"`
	}
	if err := json.Unmarshal([]byte(result), &decoded); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}
	if decoded.Error != hostile {
		t.Errorf("error message mismatch: got %q, want %q", decoded.Error, hostile)
	}
}

// erroringSchemaAction returns an error from Schema for testing the schema
// error JSON path.
type erroringSchemaAction struct{ err error }

func (a erroringSchemaAction) Execute(context.Context, json.RawMessage) (*ActionResult, error) {
	return ActionResultDone(nil), nil
}
func (a erroringSchemaAction) Schema(json.RawMessage) (string, string, error) {
	return "", "", a.err
}
func (a erroringSchemaAction) DefaultTimeout() time.Duration { return 0 }

func TestActionsSchemaHostileErrorMessage(t *testing.T) {
	hostile := "bad schema \"x\" \\ \nnext"
	registry := NewActionsRegistry()
	registry.Register("shaped", erroringSchemaAction{err: errors.New(hostile)})
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 1,
		Msg:            &actionspb.ActionsResponse_Schema{Schema: &actionspb.SchemaRequest{Action: "shaped"}},
	}); err != nil {
		t.Fatalf("send schema request: %v", err)
	}

	resp := recvWithin(t, stream, 2*time.Second)
	schema := resp.GetSchema().GetActionSchema()
	if !json.Valid([]byte(schema)) {
		t.Fatalf("schema error payload is not valid JSON: %s", schema)
	}
	var decoded struct {
		Error string `json:"error"`
	}
	if err := json.Unmarshal([]byte(schema), &decoded); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}
	if decoded.Error != hostile {
		t.Errorf("schema error message mismatch: got %q, want %q", decoded.Error, hostile)
	}
}

func TestActionsClientDefaultsZeroValues(t *testing.T) {
	zero := NewActionsClient(context.Background(), "svc", NewActionsRegistry(), ActionsClientConfig{})
	if zero.config.ReconnectDelay != defaultReconnectDelay {
		t.Errorf("ReconnectDelay = %v, want %v", zero.config.ReconnectDelay, defaultReconnectDelay)
	}
	if zero.config.ListInterval != defaultListInterval {
		t.Errorf("ListInterval = %v, want %v", zero.config.ListInterval, defaultListInterval)
	}

	explicit := NewActionsClient(context.Background(), "svc", NewActionsRegistry(), ActionsClientConfig{
		ReconnectDelay: 1 * time.Second,
		ListInterval:   2 * time.Second,
	})
	if explicit.config.ReconnectDelay != 1*time.Second {
		t.Errorf("explicit ReconnectDelay = %v, want 1s", explicit.config.ReconnectDelay)
	}
	if explicit.config.ListInterval != 2*time.Second {
		t.Errorf("explicit ListInterval = %v, want 2s", explicit.config.ListInterval)
	}
}

// defaultTimeoutAction is a test Action with a configurable default timeout.
type defaultTimeoutAction struct {
	d  time.Duration
	fn func(context.Context, json.RawMessage) (*ActionResult, error)
}

func (a defaultTimeoutAction) Execute(ctx context.Context, params json.RawMessage) (*ActionResult, error) {
	return a.fn(ctx, params)
}
func (a defaultTimeoutAction) Schema(json.RawMessage) (string, string, error) { return "{}", "{}", nil }
func (a defaultTimeoutAction) DefaultTimeout() time.Duration                  { return a.d }

func TestActionsExecuteZeroTimeoutUsesDefault(t *testing.T) {
	registry := NewActionsRegistry()
	registry.Register("slow", defaultTimeoutAction{
		d: 50 * time.Millisecond,
		fn: func(ctx context.Context, _ json.RawMessage) (*ActionResult, error) {
			<-ctx.Done()
			return nil, ctx.Err()
		},
	})
	stream, cleanup := newTestClient(t, "svc", registry)
	defer cleanup()

	recvWithin(t, stream, 2*time.Second) // handshake

	// timeout_ms == 0 must be treated as absent and fall back to the action's
	// 50ms default (not "no timeout").
	zero := uint32(0)
	if err := stream.Send(&actionspb.ActionsResponse{
		SequenceNumber: 1,
		Msg:            &actionspb.ActionsResponse_Execute{Execute: &actionspb.ExecuteRequest{Action: "slow", Params: "{}", TimeoutMs: &zero}},
	}); err != nil {
		t.Fatalf("send execute request: %v", err)
	}

	resp := recvWithin(t, stream, 2*time.Second)
	exec := resp.GetExecute()
	if exec.GetStatus() != actionspb.ExecuteStatus_ERROR {
		t.Errorf("status = %v, want ERROR", exec.GetStatus())
	}
	if exec.GetResult() != `{"error":"Action timed out after 50ms"}` {
		t.Errorf("result = %q, want default-timeout message", exec.GetResult())
	}
}

func TestNewActionNilFuncReturnsError(t *testing.T) {
	fromString := NewAction("{}", nil)
	if _, err := fromString.Execute(context.Background(), json.RawMessage(`{}`)); err == nil {
		t.Error("NewAction with nil fn: expected error, got nil")
	}

	fromSchema := NewActionFromSchema(NewActionSchema("T", "D"), nil)
	if _, err := fromSchema.Execute(context.Background(), json.RawMessage(`{}`)); err == nil {
		t.Error("NewActionFromSchema with nil fn: expected error, got nil")
	}
}

func TestNilInputsAreHandledSafely(t *testing.T) {
	registry := NewActionsRegistry()
	if err := registry.Register("nil", nil); err == nil {
		t.Fatal("Register(nil): expected error")
	}
	if got := registry.List(); len(got) != 0 {
		t.Fatalf("Register(nil) added an action: %v", got)
	}

	action := NewActionFromSchema(nil, func(context.Context, json.RawMessage) (*ActionResult, error) {
		return ActionResultDone(nil), nil
	})
	if _, _, err := action.Schema(nil); err == nil {
		t.Fatal("NewActionFromSchema(nil): expected schema error")
	}

	client := NewActionsClient(context.Background(), "svc", nil, ActionsClientConfig{})
	if client.registry == nil {
		t.Fatal("NewActionsClient(nil registry) left registry nil")
	}
}

func TestActionsRunStopsOnCancel(t *testing.T) {
	registry := NewActionsRegistry()
	ctx, cancel := context.WithCancel(context.Background())
	client := NewActionsClient(ctx, "svc", registry, ActionsClientConfig{
		URL:            "grpc://127.0.0.1:1", // unreachable; Run should still stop on cancel
		ReconnectDelay: 50 * time.Millisecond,
		ListInterval:   time.Hour,
	})

	done := make(chan error, 1)
	go func() { done <- client.Run() }()

	cancel()

	select {
	case <-done:
	case <-time.After(2 * time.Second):
		t.Fatal("Run did not return after context cancellation")
	}
}
