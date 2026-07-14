package zelos

import (
	"encoding/json"
	"testing"
)

func TestActionSchemaDeclarationOrder(t *testing.T) {
	// Declare fields non-alphabetically (z, a, m) to prove insertion order is
	// preserved rather than alphabetized.
	got := NewActionSchema("T", "D").
		Number("z", Required()).
		Text("a").
		Boolean("m").
		JSONSchema()

	want := `{"title":"T","description":"D","type":"object","properties":{"z":{"type":"number"},"a":{"type":"string"},"m":{"type":"boolean"}},"required":["z"]}`
	if got != want {
		t.Fatalf("JSONSchema mismatch\n got: %s\nwant: %s", got, want)
	}
}

func TestActionSchemaRequiredOmittedWhenEmpty(t *testing.T) {
	got := NewActionSchema("T", "D").Text("a").JSONSchema()

	want := `{"title":"T","description":"D","type":"object","properties":{"a":{"type":"string"}}}`
	if got != want {
		t.Fatalf("JSONSchema mismatch\n got: %s\nwant: %s", got, want)
	}
}

func TestActionSchemaRequiredContents(t *testing.T) {
	got := NewActionSchema("T", "D").
		Number("x", Required()).
		Number("y").
		Text("label", Required()).
		JSONSchema()

	want := `{"title":"T","description":"D","type":"object","properties":{"x":{"type":"number"},"y":{"type":"number"},"label":{"type":"string"}},"required":["x","label"]}`
	if got != want {
		t.Fatalf("JSONSchema mismatch\n got: %s\nwant: %s", got, want)
	}
}

func TestActionSchemaSelectEnumOrder(t *testing.T) {
	got := NewActionSchema("T", "D").
		Select("mode", []string{"fast", "precise"}, Required()).
		JSONSchema()

	want := `{"title":"T","description":"D","type":"object","properties":{"mode":{"type":"string","enum":["fast","precise"]}},"required":["mode"]}`
	if got != want {
		t.Fatalf("JSONSchema mismatch\n got: %s\nwant: %s", got, want)
	}
}

func TestActionSchemaFieldAttributes(t *testing.T) {
	got := NewActionSchema("Add Numbers", "Add two numbers together").
		Number("x", Description("First number"), Required()).
		Integer("count", Minimum(0), Maximum(10)).
		Boolean("verbose", Default(false)).
		Text("label", Title("Label")).
		JSONSchema()

	want := `{"title":"Add Numbers","description":"Add two numbers together","type":"object","properties":{"x":{"type":"number","description":"First number"},"count":{"type":"integer","minimum":0,"maximum":10},"verbose":{"type":"boolean","default":false},"label":{"type":"string","title":"Label"}},"required":["x"]}`
	if got != want {
		t.Fatalf("JSONSchema mismatch\n got: %s\nwant: %s", got, want)
	}
}

func TestActionSchemaValidJSON(t *testing.T) {
	s := NewActionSchema("Add Numbers", "Add two numbers together").
		Number("x", zeroDescription(), Required()).
		Select("mode", []string{"fast", "precise"}).
		Boolean("verbose", Default(true))

	out := s.JSONSchema()
	if !json.Valid([]byte(out)) {
		t.Fatalf("JSONSchema is not valid JSON: %s", out)
	}

	var doc map[string]any
	if err := json.Unmarshal([]byte(out), &doc); err != nil {
		t.Fatalf("round-trip unmarshal failed: %v", err)
	}
	if doc["type"] != "object" {
		t.Fatalf("expected type=object, got %v", doc["type"])
	}

	if s.UISchema() != "{}" {
		t.Fatalf("expected UISchema {}, got %s", s.UISchema())
	}
}

func zeroDescription() FieldOption { return Description("desc with \"quotes\" & symbols") }
