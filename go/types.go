package zelos

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"time"

	"github.com/google/uuid"
)

// DataType represents the type of data being stored
type DataType int

const (
	DataTypeUnspecified DataType = iota
	DataTypeInt8
	DataTypeInt16
	DataTypeInt32
	DataTypeInt64
	DataTypeUint8
	DataTypeUint16
	DataTypeUint32
	DataTypeUint64
	DataTypeFloat32
	DataTypeFloat64
	DataTypeTimestampNs
	DataTypeBinary
	DataTypeString
	DataTypeBoolean
)

// String returns the string representation of the DataType
func (dt DataType) String() string {
	switch dt {
	case DataTypeUnspecified:
		return "unspecified"
	case DataTypeInt8:
		return "int8"
	case DataTypeInt16:
		return "int16"
	case DataTypeInt32:
		return "int32"
	case DataTypeInt64:
		return "int64"
	case DataTypeUint8:
		return "uint8"
	case DataTypeUint16:
		return "uint16"
	case DataTypeUint32:
		return "uint32"
	case DataTypeUint64:
		return "uint64"
	case DataTypeFloat32:
		return "float32"
	case DataTypeFloat64:
		return "float64"
	case DataTypeTimestampNs:
		return "timestamp_ns"
	case DataTypeBinary:
		return "binary"
	case DataTypeString:
		return "string"
	case DataTypeBoolean:
		return "boolean"
	default:
		return "unknown"
	}
}

// Value represents a typed value that can be stored in a trace
type Value struct {
	value interface{}
}

// NewValue creates a new Value with the given data
func NewValue(v interface{}) *Value {
	return &Value{value: v}
}

// NewInt8Value creates a new int8 Value
func NewInt8Value(v int8) *Value {
	return &Value{value: v}
}

// NewInt16Value creates a new int16 Value
func NewInt16Value(v int16) *Value {
	return &Value{value: v}
}

// NewInt32Value creates a new int32 Value
func NewInt32Value(v int32) *Value {
	return &Value{value: v}
}

// NewInt64Value creates a new int64 Value
func NewInt64Value(v int64) *Value {
	return &Value{value: v}
}

// NewUint8Value creates a new uint8 Value
func NewUint8Value(v uint8) *Value {
	return &Value{value: v}
}

// NewUint16Value creates a new uint16 Value
func NewUint16Value(v uint16) *Value {
	return &Value{value: v}
}

// NewUint32Value creates a new uint32 Value
func NewUint32Value(v uint32) *Value {
	return &Value{value: v}
}

// NewUint64Value creates a new uint64 Value
func NewUint64Value(v uint64) *Value {
	return &Value{value: v}
}

// NewFloat32Value creates a new float32 Value
func NewFloat32Value(v float32) *Value {
	return &Value{value: v}
}

// NewFloat64Value creates a new float64 Value
func NewFloat64Value(v float64) *Value {
	return &Value{value: v}
}

// NewTimestampNsValue creates a new timestamp Value
func NewTimestampNsValue(v int64) *Value {
	return &Value{value: v}
}

// NewBinaryValue creates a new binary Value
func NewBinaryValue(v []byte) *Value {
	return &Value{value: v}
}

// NewStringValue creates a new string Value
func NewStringValue(v string) *Value {
	return &Value{value: v}
}

// NewBooleanValue creates a new boolean Value
func NewBooleanValue(v bool) *Value {
	return &Value{value: v}
}

// DataType returns the DataType of this Value
func (v *Value) DataType() DataType {
	switch v.value.(type) {
	case int8:
		return DataTypeInt8
	case int16:
		return DataTypeInt16
	case int32:
		return DataTypeInt32
	case int64:
		return DataTypeInt64
	case uint8:
		return DataTypeUint8
	case uint16:
		return DataTypeUint16
	case uint32:
		return DataTypeUint32
	case uint64:
		return DataTypeUint64
	case float32:
		return DataTypeFloat32
	case float64:
		return DataTypeFloat64
	case []byte:
		return DataTypeBinary
	case string:
		return DataTypeString
	case bool:
		return DataTypeBoolean
	default:
		return DataTypeUnspecified
	}
}

// AsInt8 returns the value as int8 if possible
func (v *Value) AsInt8() (int8, bool) {
	if val, ok := v.value.(int8); ok {
		return val, true
	}
	return 0, false
}

// AsInt16 returns the value as int16 if possible
func (v *Value) AsInt16() (int16, bool) {
	if val, ok := v.value.(int16); ok {
		return val, true
	}
	return 0, false
}

// AsInt32 returns the value as int32 if possible
func (v *Value) AsInt32() (int32, bool) {
	if val, ok := v.value.(int32); ok {
		return val, true
	}
	return 0, false
}

// AsInt64 returns the value as int64 if possible
func (v *Value) AsInt64() (int64, bool) {
	if val, ok := v.value.(int64); ok {
		return val, true
	}
	return 0, false
}

// AsUint8 returns the value as uint8 if possible
func (v *Value) AsUint8() (uint8, bool) {
	if val, ok := v.value.(uint8); ok {
		return val, true
	}
	return 0, false
}

// AsUint16 returns the value as uint16 if possible
func (v *Value) AsUint16() (uint16, bool) {
	if val, ok := v.value.(uint16); ok {
		return val, true
	}
	return 0, false
}

// AsUint32 returns the value as uint32 if possible
func (v *Value) AsUint32() (uint32, bool) {
	if val, ok := v.value.(uint32); ok {
		return val, true
	}
	return 0, false
}

// AsUint64 returns the value as uint64 if possible
func (v *Value) AsUint64() (uint64, bool) {
	if val, ok := v.value.(uint64); ok {
		return val, true
	}
	return 0, false
}

// AsFloat32 returns the value as float32 if possible
func (v *Value) AsFloat32() (float32, bool) {
	if val, ok := v.value.(float32); ok {
		return val, true
	}
	return 0, false
}

// AsFloat64 returns the value as float64 if possible
func (v *Value) AsFloat64() (float64, bool) {
	if val, ok := v.value.(float64); ok {
		return val, true
	}
	return 0, false
}

// AsTimestampNs returns the value as int64 timestamp if possible
func (v *Value) AsTimestampNs() (int64, bool) {
	if val, ok := v.value.(int64); ok {
		return val, true
	}
	return 0, false
}

// AsBinary returns the value as []byte if possible
func (v *Value) AsBinary() ([]byte, bool) {
	if val, ok := v.value.([]byte); ok {
		return val, true
	}
	return nil, false
}

// AsString returns the value as string if possible
func (v *Value) AsString() (string, bool) {
	if val, ok := v.value.(string); ok {
		return val, true
	}
	return "", false
}

// AsBoolean returns the value as bool if possible
func (v *Value) AsBoolean() (bool, bool) {
	if val, ok := v.value.(bool); ok {
		return val, true
	}
	return false, false
}

// String returns the string representation of the Value
func (v *Value) String() string {
	switch val := v.value.(type) {
	case int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64, bool:
		return fmt.Sprintf("%v", val)
	case []byte:
		return base64.StdEncoding.EncodeToString(val)
	case string:
		return val
	default:
		return fmt.Sprintf("%v", val)
	}
}

// MarshalJSON implements json.Marshaler
func (v *Value) MarshalJSON() ([]byte, error) {
	switch val := v.value.(type) {
	case int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64, bool, string:
		return json.Marshal(val)
	case []byte:
		return json.Marshal(base64.StdEncoding.EncodeToString(val))
	default:
		return json.Marshal(val)
	}
}

// TraceEventFieldMetadata represents metadata for a field in an event
type TraceEventFieldMetadata struct {
	Name     string   `json:"name"`
	DataType DataType `json:"data_type"`
	Unit     *string  `json:"unit,omitempty"`
}

// NewTraceEventFieldMetadata creates a new TraceEventFieldMetadata
func NewTraceEventFieldMetadata(name string, dataType DataType, unit *string) *TraceEventFieldMetadata {
	return &TraceEventFieldMetadata{
		Name:     name,
		DataType: dataType,
		Unit:     unit,
	}
}

// TraceSegmentStart represents the start of a trace segment
type TraceSegmentStart struct {
	TimeNs     int64  `json:"time_ns"`
	SourceName string `json:"source_name"`
}

// TraceSegmentEnd represents the end of a trace segment
type TraceSegmentEnd struct {
	TimeNs int64 `json:"time_ns"`
}

// TraceEventSchema represents the schema for an event
type TraceEventSchema struct {
	Name   string                     `json:"name"`
	Fields []*TraceEventFieldMetadata `json:"fields"`
}

// TraceEventFieldNamedValues represents named values for event fields
type TraceEventFieldNamedValues struct {
	EventName string            `json:"event_name"`
	FieldName string            `json:"field_name"`
	Values    map[*Value]string `json:"values"`
}

// TraceEvent represents a trace event with timestamp and fields
type TraceEvent struct {
	TimeNs int64             `json:"time_ns"`
	Name   string            `json:"name"`
	Fields map[string]*Value `json:"fields"`
}

// IpcMessage represents the different types of messages in the IPC system
type IpcMessage interface {
	ipcMessage()
}

// IpcMessageSegmentStart implements IpcMessage
type IpcMessageSegmentStart struct {
	*TraceSegmentStart
}

func (m *IpcMessageSegmentStart) ipcMessage() {}

// IpcMessageSegmentEnd implements IpcMessage
type IpcMessageSegmentEnd struct {
	*TraceSegmentEnd
}

func (m *IpcMessageSegmentEnd) ipcMessage() {}

// IpcMessageEventSchema implements IpcMessage
type IpcMessageEventSchema struct {
	*TraceEventSchema
}

func (m *IpcMessageEventSchema) ipcMessage() {}

// IpcMessageEventFieldNamedValues implements IpcMessage
type IpcMessageEventFieldNamedValues struct {
	*TraceEventFieldNamedValues
}

func (m *IpcMessageEventFieldNamedValues) ipcMessage() {}

// IpcMessageEvent implements IpcMessage
type IpcMessageEvent struct {
	*TraceEvent
}

func (m *IpcMessageEvent) ipcMessage() {}

// IpcMessageWithId wraps an IPC message with identification
type IpcMessageWithId struct {
	SegmentID  uuid.UUID  `json:"segment_id"`
	SourceName string     `json:"source_name"`
	Message    IpcMessage `json:"message"`
}

// Signal represents a signal in the tracing system
type Signal struct {
	DataSegmentID uuid.UUID         `json:"data_segment_id"`
	Source        string            `json:"source"`
	Message       string            `json:"message"`
	Signal        string            `json:"signal"`
	DataType      DataType          `json:"data_type"`
	Unit          *string           `json:"unit,omitempty"`
	ValueTable    map[string]string `json:"value_table,omitempty"`
}

// NewSignal creates a new Signal
func NewSignal(dataSegmentID uuid.UUID, source, message, signal string, dataType DataType) *Signal {
	return &Signal{
		DataSegmentID: dataSegmentID,
		Source:        source,
		Message:       message,
		Signal:        signal,
		DataType:      dataType,
	}
}

// KeyString returns the key string for this signal
func (s *Signal) KeyString() string {
	return fmt.Sprintf("%s/%s/%s.%s", s.DataSegmentID, s.Source, s.Message, s.Signal)
}

// FullyQualifiedTableName returns the fully qualified table name
func (s *Signal) FullyQualifiedTableName() string {
	return fmt.Sprintf(`"%s"."%s/%s"`, s.DataSegmentID, s.Source, s.Message)
}

// TraceMetadata represents metadata for a trace
type TraceMetadata struct {
	SourceName string
	StartTime  time.Time
	EndTime    *time.Time
}

// NewTraceMetadata creates a new TraceMetadata
func NewTraceMetadata(sourceName string) *TraceMetadata {
	return &TraceMetadata{
		SourceName: sourceName,
		StartTime:  time.Now(),
	}
}

// NowTimeNs returns the current time in nanoseconds
func NowTimeNs() int64 {
	return time.Now().UnixNano()
}
