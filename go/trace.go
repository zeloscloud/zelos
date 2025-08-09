package zelos

import (
	"context"
	"fmt"
	"sync"

	"github.com/google/uuid"
)

// Sender represents a channel for sending IpcMessageWithId
type Sender chan *IpcMessageWithId

// Receiver represents a channel for receiving IpcMessageWithId
type Receiver chan *IpcMessageWithId

// NewChannel creates a new sender/receiver pair with the given buffer size
func NewChannel(bufferSize int) (Sender, Receiver) {
	ch := make(chan *IpcMessageWithId, bufferSize)
	return ch, ch
}

// TraceSourceEvent represents an event that belongs to a TraceSource
type TraceSourceEvent struct {
	id         uuid.UUID
	sourceName string
	sender     Sender
	Name       string
	Schema     []*TraceEventFieldMetadata
}

// NewTraceSourceEvent creates a new TraceSourceEvent
func NewTraceSourceEvent(id uuid.UUID, sourceName, name string, sender Sender, schema []*TraceEventFieldMetadata) *TraceSourceEvent {
	return &TraceSourceEvent{
		id:         id,
		sourceName: sourceName,
		sender:     sender,
		Name:       name,
		Schema:     schema,
	}
}

// Build creates a new EventBuilder for this event
func (e *TraceSourceEvent) Build() *EventBuilder {
	return NewEventBuilder(e)
}

// EventBuilder helps build and emit trace events
type EventBuilder struct {
	parent *TraceSourceEvent
	data   map[string]*Value
}

// NewEventBuilder creates a new EventBuilder
func NewEventBuilder(parent *TraceSourceEvent) *EventBuilder {
	return &EventBuilder{
		parent: parent,
		data:   make(map[string]*Value),
	}
}

// TryInsert attempts to insert a value, validating it matches the schema
func (b *EventBuilder) TryInsert(name string, value *Value) error {
	// Find the field in the schema
	var field *TraceEventFieldMetadata
	for _, f := range b.parent.Schema {
		if f.Name == name {
			field = f
			break
		}
	}

	if field == nil {
		return fmt.Errorf("field '%s' not found in schema", name)
	}

	// Check if value type matches schema
	if field.DataType != value.DataType() {
		return fmt.Errorf("type mismatch for field '%s': expected %s, got %s",
			name, field.DataType.String(), value.DataType().String())
	}

	b.data[name] = value
	return nil
}

// TryInsertInt8 inserts an int8 value
func (b *EventBuilder) TryInsertInt8(name string, value int8) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewInt8Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertInt16 inserts an int16 value
func (b *EventBuilder) TryInsertInt16(name string, value int16) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewInt16Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertInt32 inserts an int32 value
func (b *EventBuilder) TryInsertInt32(name string, value int32) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewInt32Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertInt64 inserts an int64 value
func (b *EventBuilder) TryInsertInt64(name string, value int64) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewInt64Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertUint8 inserts a uint8 value
func (b *EventBuilder) TryInsertUint8(name string, value uint8) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewUint8Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertUint16 inserts a uint16 value
func (b *EventBuilder) TryInsertUint16(name string, value uint16) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewUint16Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertUint32 inserts a uint32 value
func (b *EventBuilder) TryInsertUint32(name string, value uint32) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewUint32Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertUint64 inserts a uint64 value
func (b *EventBuilder) TryInsertUint64(name string, value uint64) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewUint64Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertFloat32 inserts a float32 value
func (b *EventBuilder) TryInsertFloat32(name string, value float32) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewFloat32Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertFloat64 inserts a float64 value
func (b *EventBuilder) TryInsertFloat64(name string, value float64) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewFloat64Value(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertString inserts a string value
func (b *EventBuilder) TryInsertString(name string, value string) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewStringValue(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertBoolean inserts a boolean value
func (b *EventBuilder) TryInsertBoolean(name string, value bool) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewBooleanValue(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertBinary inserts a binary value
func (b *EventBuilder) TryInsertBinary(name string, value []byte) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewBinaryValue(value)); err != nil {
		return b, err
	}
	return b, nil
}

// TryInsertTimestampNs inserts a timestamp value
func (b *EventBuilder) TryInsertTimestampNs(name string, value int64) (*EventBuilder, error) {
	if err := b.TryInsert(name, NewTimestampNsValue(value)); err != nil {
		return b, err
	}
	return b, nil
}

// Emit emits the event
func (b *EventBuilder) Emit() error {
	event := &TraceEvent{
		TimeNs: NowTimeNs(),
		Name:   b.parent.Name,
		Fields: b.data,
	}

	msg := &IpcMessageWithId{
		SegmentID:  b.parent.id,
		SourceName: b.parent.sourceName,
		Message:    &IpcMessageEvent{event},
	}

	select {
	case b.parent.sender <- msg:
		return nil
	default:
		return fmt.Errorf("failed to send event: channel full")
	}
}

// EmitAt emits the event at the specified timestamp
func (b *EventBuilder) EmitAt(timeNs int64) error {
    event := &TraceEvent{
        TimeNs: timeNs,
        Name:   b.parent.Name,
        Fields: b.data,
    }

    msg := &IpcMessageWithId{
        SegmentID:  b.parent.id,
        SourceName: b.parent.sourceName,
        Message:    &IpcMessageEvent{event},
    }

    select {
    case b.parent.sender <- msg:
        return nil
    default:
        return fmt.Errorf("failed to send event: channel full")
    }
}

// TraceSourceEventBuilder helps build event schemas
type TraceSourceEventBuilder struct {
	source *TraceSource
	name   string
	schema map[string]*TraceEventFieldMetadata
}

// NewTraceSourceEventBuilder creates a new TraceSourceEventBuilder
func NewTraceSourceEventBuilder(source *TraceSource, name string) *TraceSourceEventBuilder {
	return &TraceSourceEventBuilder{
		source: source,
		name:   name,
		schema: make(map[string]*TraceEventFieldMetadata),
	}
}

// AddField adds a field to the event schema
func (b *TraceSourceEventBuilder) AddField(name string, dataType DataType, unit *string) *TraceSourceEventBuilder {
	b.schema[name] = NewTraceEventFieldMetadata(name, dataType, unit)
	return b
}

// AddInt8Field adds an int8 field
func (b *TraceSourceEventBuilder) AddInt8Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeInt8, unit)
}

// AddInt16Field adds an int16 field
func (b *TraceSourceEventBuilder) AddInt16Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeInt16, unit)
}

// AddInt32Field adds an int32 field
func (b *TraceSourceEventBuilder) AddInt32Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeInt32, unit)
}

// AddInt64Field adds an int64 field
func (b *TraceSourceEventBuilder) AddInt64Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeInt64, unit)
}

// AddUint8Field adds a uint8 field
func (b *TraceSourceEventBuilder) AddUint8Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeUint8, unit)
}

// AddUint16Field adds a uint16 field
func (b *TraceSourceEventBuilder) AddUint16Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeUint16, unit)
}

// AddUint32Field adds a uint32 field
func (b *TraceSourceEventBuilder) AddUint32Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeUint32, unit)
}

// AddUint64Field adds a uint64 field
func (b *TraceSourceEventBuilder) AddUint64Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeUint64, unit)
}

// AddFloat32Field adds a float32 field
func (b *TraceSourceEventBuilder) AddFloat32Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeFloat32, unit)
}

// AddFloat64Field adds a float64 field
func (b *TraceSourceEventBuilder) AddFloat64Field(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeFloat64, unit)
}

// AddStringField adds a string field
func (b *TraceSourceEventBuilder) AddStringField(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeString, unit)
}

// AddBooleanField adds a boolean field
func (b *TraceSourceEventBuilder) AddBooleanField(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeBoolean, unit)
}

// AddBinaryField adds a binary field
func (b *TraceSourceEventBuilder) AddBinaryField(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeBinary, unit)
}

// AddTimestampNsField adds a timestamp field
func (b *TraceSourceEventBuilder) AddTimestampNsField(name string, unit *string) *TraceSourceEventBuilder {
	return b.AddField(name, DataTypeTimestampNs, unit)
}

// Build builds and registers the event with the source
func (b *TraceSourceEventBuilder) Build() (*TraceSourceEvent, error) {
	return b.source.AddEvent(b.name, b.schema)
}

// TraceSource represents a source of trace events
type TraceSource struct {
	ID         uuid.UUID
	SourceName string
	sender     Sender
	events     map[string]*TraceSourceEvent
	mu         sync.RWMutex
}

// NewTraceSource creates a new TraceSource
func NewTraceSource(sourceName string, sender Sender) (*TraceSource, error) {
	id := uuid.New() // In Rust this would be uuid::now_v7()

	source := &TraceSource{
		ID:         id,
		SourceName: sourceName,
		sender:     sender,
		events:     make(map[string]*TraceSourceEvent),
	}

	// Emit segment start
	if err := source.emitStart(); err != nil {
		return nil, fmt.Errorf("failed to emit segment start: %w", err)
	}

	return source, nil
}

// emitStart emits a segment start message
func (s *TraceSource) emitStart() error {
	msg := &IpcMessageWithId{
		SegmentID:  s.ID,
		SourceName: s.SourceName,
		Message: &IpcMessageSegmentStart{
			&TraceSegmentStart{
				TimeNs:     NowTimeNs(),
				SourceName: s.SourceName,
			},
		},
	}

	select {
	case s.sender <- msg:
		return nil
	default:
		return fmt.Errorf("failed to send segment start: channel full")
	}
}

// emitEnd emits a segment end message
func (s *TraceSource) emitEnd() error {
	msg := &IpcMessageWithId{
		SegmentID:  s.ID,
		SourceName: s.SourceName,
		Message: &IpcMessageSegmentEnd{
			&TraceSegmentEnd{
				TimeNs: NowTimeNs(),
			},
		},
	}

	select {
	case s.sender <- msg:
		return nil
	default:
		return fmt.Errorf("failed to send segment end: channel full")
	}
}

// Close emits a segment end message (similar to Drop in Rust)
func (s *TraceSource) Close() error {
	return s.emitEnd()
}

// BuildEvent creates a new TraceSourceEventBuilder
func (s *TraceSource) BuildEvent(name string) *TraceSourceEventBuilder {
	return NewTraceSourceEventBuilder(s, name)
}

// AddValueTable attaches a value table for a specific event field
func (s *TraceSource) AddValueTable(eventName, fieldName string, values map[*Value]string) error {
    msg := &IpcMessageWithId{
        SegmentID:  s.ID,
        SourceName: s.SourceName,
        Message: &IpcMessageEventFieldNamedValues{&TraceEventFieldNamedValues{
            EventName: eventName,
            FieldName: fieldName,
            Values:    values,
        }},
    }
    select {
    case s.sender <- msg:
        return nil
    default:
        return fmt.Errorf("failed to send value table: channel full")
    }
}

// AddEvent adds an event to the source
func (s *TraceSource) AddEvent(name string, schema map[string]*TraceEventFieldMetadata) (*TraceSourceEvent, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.events[name]; exists {
		return nil, fmt.Errorf("event '%s' already exists", name)
	}

	// Convert schema map to slice
	var fields []*TraceEventFieldMetadata
	for _, field := range schema {
		fields = append(fields, field)
	}

	event := NewTraceSourceEvent(s.ID, s.SourceName, name, s.sender, fields)

	// Emit the event schema
	schemaMsg := &IpcMessageWithId{
		SegmentID:  s.ID,
		SourceName: s.SourceName,
		Message: &IpcMessageEventSchema{
			&TraceEventSchema{
				Name:   name,
				Fields: fields,
			},
		},
	}

	select {
	case s.sender <- schemaMsg:
		// Success, store the event
		s.events[name] = event
		return event, nil
	default:
		return nil, fmt.Errorf("failed to send event schema: channel full")
	}
}

// GetEvent retrieves an event by name
func (s *TraceSource) GetEvent(name string) (*TraceSourceEvent, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	event, exists := s.events[name]
	if !exists {
		return nil, fmt.Errorf("event '%s' not found", name)
	}

	return event, nil
}

// TraceRouter routes trace messages between sources and sinks
type TraceRouter struct {
	sender Sender
	mu     sync.RWMutex
}

// NewTraceRouter creates a new TraceRouter
func NewTraceRouter(ctx context.Context) (*TraceRouter, Sender, Receiver) {
	sender, receiver := NewChannel(1024) // Default channel size like Rust

	router := &TraceRouter{
		sender: sender,
	}

	return router, sender, receiver
}

// Sender returns the sender channel for this router
func (r *TraceRouter) Sender() Sender {
	return r.sender
}
