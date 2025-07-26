package zelos

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestValueTypes(t *testing.T) {
	// Test int8
	val := NewInt8Value(42)
	if dt := val.DataType(); dt != DataTypeInt8 {
		t.Errorf("Expected DataTypeInt8, got %v", dt)
	}
	if v, ok := val.AsInt8(); !ok || v != 42 {
		t.Errorf("Expected 42, got %v, ok=%v", v, ok)
	}

	// Test string
	strVal := NewStringValue("hello")
	if dt := strVal.DataType(); dt != DataTypeString {
		t.Errorf("Expected DataTypeString, got %v", dt)
	}
	if v, ok := strVal.AsString(); !ok || v != "hello" {
		t.Errorf("Expected 'hello', got %v, ok=%v", v, ok)
	}

	// Test boolean
	boolVal := NewBooleanValue(true)
	if dt := boolVal.DataType(); dt != DataTypeBoolean {
		t.Errorf("Expected DataTypeBoolean, got %v", dt)
	}
	if v, ok := boolVal.AsBoolean(); !ok || v != true {
		t.Errorf("Expected true, got %v, ok=%v", v, ok)
	}
}

func TestTraceSource(t *testing.T) {
	ctx := context.Background()
	_, sender, _ := NewTraceRouter(ctx)

	source, err := NewTraceSource("test-source", sender)
	if err != nil {
		t.Fatalf("Failed to create trace source: %v", err)
	}
	defer source.Close()

	if source.SourceName != "test-source" {
		t.Errorf("Expected 'test-source', got %s", source.SourceName)
	}

	// Check that ID was generated
	if source.ID == uuid.Nil {
		t.Error("Expected non-nil UUID for source ID")
	}
}

func TestTraceSourceEventBuilder(t *testing.T) {
	ctx := context.Background()
	_, sender, _ := NewTraceRouter(ctx)

	source, err := NewTraceSource("test-source", sender)
	if err != nil {
		t.Fatalf("Failed to create trace source: %v", err)
	}
	defer source.Close()

	event, err := source.BuildEvent("test-event").
		AddStringField("message", nil).
		AddUint64Field("count", nil).
		Build()
	if err != nil {
		t.Fatalf("Failed to build event: %v", err)
	}

	if event.Name != "test-event" {
		t.Errorf("Expected 'test-event', got %s", event.Name)
	}

	if len(event.Schema) != 2 {
		t.Errorf("Expected 2 fields in schema, got %d", len(event.Schema))
	}
}

func TestEventBuilderValidation(t *testing.T) {
	ctx := context.Background()
	_, sender, _ := NewTraceRouter(ctx)

	source, err := NewTraceSource("test-source", sender)
	if err != nil {
		t.Fatalf("Failed to create trace source: %v", err)
	}
	defer source.Close()

	event, err := source.BuildEvent("test-event").
		AddStringField("message", nil).
		AddUint64Field("count", nil).
		Build()
	if err != nil {
		t.Fatalf("Failed to build event: %v", err)
	}

	builder := event.Build()

	// Test valid insertion
	_, err = builder.TryInsertString("message", "hello")
	if err != nil {
		t.Errorf("Valid insertion failed: %v", err)
	}

	// Test type mismatch
	_, err = builder.TryInsertString("count", "not a number")
	if err == nil {
		t.Error("Expected type mismatch error")
	}

	// Test unknown field
	_, err = builder.TryInsertString("unknown", "value")
	if err == nil {
		t.Error("Expected unknown field error")
	}
}

func TestNowTimeNs(t *testing.T) {
	before := time.Now().UnixNano()
	now := NowTimeNs()
	after := time.Now().UnixNano()

	if now < before || now > after {
		t.Errorf("NowTimeNs() returned %d, should be between %d and %d", now, before, after)
	}
}

func TestDataTypeString(t *testing.T) {
	testCases := []struct {
		dt     DataType
		expect string
	}{
		{DataTypeInt8, "int8"},
		{DataTypeString, "string"},
		{DataTypeBoolean, "boolean"},
		{DataTypeFloat64, "float64"},
	}

	for _, tc := range testCases {
		if got := tc.dt.String(); got != tc.expect {
			t.Errorf("DataType(%d).String() = %s, want %s", tc.dt, got, tc.expect)
		}
	}
}

func TestTraceEventFieldMetadata(t *testing.T) {
	unit := "ms"
	metadata := NewTraceEventFieldMetadata("duration", DataTypeUint64, &unit)

	if metadata.Name != "duration" {
		t.Errorf("Expected 'duration', got %s", metadata.Name)
	}

	if metadata.DataType != DataTypeUint64 {
		t.Errorf("Expected DataTypeUint64, got %v", metadata.DataType)
	}

	if metadata.Unit == nil || *metadata.Unit != "ms" {
		t.Errorf("Expected unit 'ms', got %v", metadata.Unit)
	}
}

func TestConnectionStatus(t *testing.T) {
	testCases := []struct {
		status ConnectionStatus
		expect string
	}{
		{ConnectionStatusDisconnected, "disconnected"},
		{ConnectionStatusConnecting, "connecting"},
		{ConnectionStatusConnected, "connected"},
		{ConnectionStatusError, "error"},
	}

	for _, tc := range testCases {
		if got := tc.status.String(); got != tc.expect {
			t.Errorf("ConnectionStatus(%d).String() = %s, want %s", tc.status, got, tc.expect)
		}
	}
}

func TestIpcMessageTypes(t *testing.T) {
	// Test that all IPC message types implement the interface
	var _ IpcMessage = &IpcMessageSegmentStart{}
	var _ IpcMessage = &IpcMessageSegmentEnd{}
	var _ IpcMessage = &IpcMessageEventSchema{}
	var _ IpcMessage = &IpcMessageEventFieldNamedValues{}
	var _ IpcMessage = &IpcMessageEvent{}
}

func TestChannelCreation(t *testing.T) {
	sender, receiver := NewChannel(10)

	// Test that we can send and receive
	testMsg := &IpcMessageWithId{
		SegmentID:  uuid.New(),
		SourceName: "test",
		Message:    &IpcMessageSegmentStart{&TraceSegmentStart{TimeNs: 123, SourceName: "test"}},
	}

	// Send should not block
	sender <- testMsg

	// Receive should get the same message
	received := <-receiver
	if received.SegmentID != testMsg.SegmentID {
		t.Error("Received message does not match sent message")
	}
}
