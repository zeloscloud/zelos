package zelos

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"time"
)

// ActionSchema builds a JSON Schema string for an action's parameters. Fields
// are kept in a slice and serialized in declaration order, because consumers
// render form fields in the order the schema's properties are declared.
// Building the string by hand (rather than round-tripping through a
// map[string]any, which would alphabetize keys) preserves that order.
//
// Example:
//
//	schema := zelos.NewActionSchema("Add Numbers", "Add two numbers together").
//		Number("x", zelos.Description("First number"), zelos.Required()).
//		Number("y", zelos.Required()).
//		Text("label").
//		Boolean("verbose", zelos.Default(false)).
//		Integer("count", zelos.Minimum(0)).
//		Select("mode", []string{"fast", "precise"}, zelos.Required())
//
//	action := zelos.NewActionFromSchema(schema, func(ctx context.Context, params json.RawMessage) (*zelos.ActionResult, error) {
//		// ... decode params and do work ...
//		return zelos.ActionResultDone(nil), nil
//	})
type ActionSchema struct {
	title       string
	description string
	fields      []*fieldSpec
}

// fieldSpec captures a single form field's declaration. Only attributes that are
// explicitly set are serialized.
type fieldSpec struct {
	name        string
	jsonType    string
	title       string
	description string
	required    bool
	hasDefault  bool
	defaultVal  any
	min         *float64
	max         *float64
	enum        []string
}

// FieldOption configures a single field within an ActionSchema. Options are
// applied in order; later options override earlier ones for the same attribute.
type FieldOption func(*fieldSpec)

// Description sets a field's "description".
func Description(s string) FieldOption {
	return func(f *fieldSpec) { f.description = s }
}

// Title sets a field's "title". When unset, no title is emitted.
func Title(s string) FieldOption {
	return func(f *fieldSpec) { f.title = s }
}

// Required marks a field as required.
func Required() FieldOption {
	return func(f *fieldSpec) { f.required = true }
}

// Default sets a field's default value.
func Default(v any) FieldOption {
	return func(f *fieldSpec) {
		f.hasDefault = true
		f.defaultVal = v
	}
}

// Minimum sets a numeric field's "minimum".
func Minimum(v float64) FieldOption {
	return func(f *fieldSpec) { f.min = &v }
}

// Maximum sets a numeric field's "maximum".
func Maximum(v float64) FieldOption {
	return func(f *fieldSpec) { f.max = &v }
}

// NewActionSchema starts a new schema with the given form title and description.
func NewActionSchema(title, description string) *ActionSchema {
	return &ActionSchema{title: title, description: description}
}

// Text appends a string field.
func (s *ActionSchema) Text(name string, opts ...FieldOption) *ActionSchema {
	return s.addField(name, "string", nil, opts)
}

// Number appends a floating-point number field.
func (s *ActionSchema) Number(name string, opts ...FieldOption) *ActionSchema {
	return s.addField(name, "number", nil, opts)
}

// Integer appends an integer field.
func (s *ActionSchema) Integer(name string, opts ...FieldOption) *ActionSchema {
	return s.addField(name, "integer", nil, opts)
}

// Boolean appends a boolean field.
func (s *ActionSchema) Boolean(name string, opts ...FieldOption) *ActionSchema {
	return s.addField(name, "boolean", nil, opts)
}

// Select appends a string field constrained to the given choices, emitted as an
// "enum" in declaration order.
func (s *ActionSchema) Select(name string, choices []string, opts ...FieldOption) *ActionSchema {
	return s.addField(name, "string", choices, opts)
}

func (s *ActionSchema) addField(name, jsonType string, enum []string, opts []FieldOption) *ActionSchema {
	f := &fieldSpec{name: name, jsonType: jsonType, enum: enum}
	for _, opt := range opts {
		opt(f)
	}
	s.fields = append(s.fields, f)
	return s
}

// JSONSchema serializes the schema to a JSON Schema string with properties in
// declaration order. "required" is omitted when no field is required.
func (s *ActionSchema) JSONSchema() string {
	var b strings.Builder
	b.WriteString(`{"title":`)
	b.Write(marshalJSON(s.title))
	b.WriteString(`,"description":`)
	b.Write(marshalJSON(s.description))
	b.WriteString(`,"type":"object","properties":{`)
	for i, f := range s.fields {
		if i > 0 {
			b.WriteByte(',')
		}
		b.Write(marshalJSON(f.name))
		b.WriteByte(':')
		writeFieldSchema(&b, f)
	}
	b.WriteByte('}')

	var required []string
	for _, f := range s.fields {
		if f.required {
			required = append(required, f.name)
		}
	}
	if len(required) > 0 {
		b.WriteString(`,"required":`)
		b.Write(marshalJSON(required))
	}
	b.WriteByte('}')
	return b.String()
}

// UISchema returns the UI schema. It is "{}" for v1.
func (s *ActionSchema) UISchema() string {
	return "{}"
}

func writeFieldSchema(b *strings.Builder, f *fieldSpec) {
	b.WriteString(`{"type":`)
	b.Write(marshalJSON(f.jsonType))
	if f.title != "" {
		b.WriteString(`,"title":`)
		b.Write(marshalJSON(f.title))
	}
	if f.description != "" {
		b.WriteString(`,"description":`)
		b.Write(marshalJSON(f.description))
	}
	if len(f.enum) > 0 {
		b.WriteString(`,"enum":`)
		b.Write(marshalJSON(f.enum))
	}
	if f.hasDefault {
		b.WriteString(`,"default":`)
		b.Write(marshalJSON(f.defaultVal))
	}
	if f.min != nil {
		b.WriteString(`,"minimum":`)
		b.Write(marshalJSON(*f.min))
	}
	if f.max != nil {
		b.WriteString(`,"maximum":`)
		b.Write(marshalJSON(*f.max))
	}
	b.WriteByte('}')
}

// marshalJSON marshals v to JSON. Marshaling only fails for unsupported types
// (channels, functions, cyclic values), which are not valid schema values; in
// that case it falls back to null so the overall document stays valid JSON.
func marshalJSON(v any) []byte {
	data, err := json.Marshal(v)
	if err != nil {
		return []byte("null")
	}
	return data
}

// NewActionFromSchema builds an Action whose JSON Schema is produced by an
// ActionSchema builder and whose UI schema is "{}". It has no default timeout.
func NewActionFromSchema(schema *ActionSchema, fn func(ctx context.Context, params json.RawMessage) (*ActionResult, error)) Action {
	return &builtSchemaAction{schema: schema, fn: fn}
}

// builtSchemaAction adapts an ActionSchema plus an execute function into an Action.
type builtSchemaAction struct {
	schema *ActionSchema
	fn     func(ctx context.Context, params json.RawMessage) (*ActionResult, error)
}

func (a *builtSchemaAction) Execute(ctx context.Context, params json.RawMessage) (*ActionResult, error) {
	if a.fn == nil {
		return nil, errors.New("action has no execute function")
	}
	return a.fn(ctx, params)
}

func (a *builtSchemaAction) Schema(currentValues json.RawMessage) (string, string, error) {
	if a.schema == nil {
		return "", "", errors.New("action has no schema")
	}
	return a.schema.JSONSchema(), a.schema.UISchema(), nil
}

func (a *builtSchemaAction) DefaultTimeout() time.Duration {
	return 0
}
