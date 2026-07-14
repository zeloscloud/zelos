//! A fluent, order-preserving builder for action JSON Schemas.
//!
//! Many form renderers display fields in the order the schema's `properties` are
//! declared, so [`ActionSchema`] keeps fields in a `Vec` and serializes the
//! `properties` object by hand — insertion order is preserved without pulling in
//! extra dependencies or flipping on serde_json's crate-wide `preserve_order`
//! feature.
//!
//! The typical use is inside [`Action::get_schema_json`](crate::Action::get_schema_json):
//!
//! ```
//! use serde_json::json;
//! use zelos_actions::ActionSchema;
//!
//! let schema = ActionSchema::new("Add Numbers", "Add two numbers together")
//!     .number("x", |f| f.description("First number").required())
//!     .number("y", |f| f.required())
//!     .text("label", |f| f)
//!     .boolean("verbose", |f| f.default_value(json!(false)))
//!     .integer("count", |f| f.minimum(0.0))
//!     .select("mode", &["fast", "precise"], |f| f.required());
//!
//! let (json_schema, ui_schema, description) = schema.to_schema_json();
//! assert!(json_schema.starts_with(r#"{"title":"Add Numbers""#));
//! assert_eq!(ui_schema, "{}");
//! assert_eq!(description, "Add two numbers together");
//! ```

use serde_json::Value;

/// Per-field options collected via the closure passed to a field method.
///
/// All setters are chainable. `minimum`/`maximum` are exposed on every field;
/// JSON Schema simply ignores them where they don't apply, which keeps the API
/// small.
#[derive(Debug, Clone)]
pub struct FieldBuilder {
    field_type: &'static str,
    title: Option<String>,
    description: Option<String>,
    required: bool,
    default: Option<Value>,
    minimum: Option<f64>,
    maximum: Option<f64>,
    choices: Option<Vec<String>>,
}

impl FieldBuilder {
    fn new(field_type: &'static str) -> Self {
        Self {
            field_type,
            title: None,
            description: None,
            required: false,
            default: None,
            minimum: None,
            maximum: None,
            choices: None,
        }
    }

    /// Set the field's human-readable title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the field's description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark the field as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set a default value for the field.
    pub fn default_value(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Set the minimum allowed value (numeric fields).
    pub fn minimum(mut self, minimum: f64) -> Self {
        self.minimum = Some(minimum);
        self
    }

    /// Set the maximum allowed value (numeric fields).
    pub fn maximum(mut self, maximum: f64) -> Self {
        self.maximum = Some(maximum);
        self
    }

    /// Serialize this field's schema object, emitting keys in a fixed order.
    fn to_schema_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        parts.push(format!("\"type\":{}", encode(&self.field_type)));
        if let Some(title) = &self.title {
            parts.push(format!("\"title\":{}", encode(title)));
        }
        if let Some(description) = &self.description {
            parts.push(format!("\"description\":{}", encode(description)));
        }
        if let Some(choices) = &self.choices {
            parts.push(format!("\"enum\":{}", encode(choices)));
        }
        if let Some(default) = &self.default {
            parts.push(format!("\"default\":{}", encode(default)));
        }
        if let Some(minimum) = self.minimum {
            parts.push(format!("\"minimum\":{}", encode(&minimum)));
        }
        if let Some(maximum) = self.maximum {
            parts.push(format!("\"maximum\":{}", encode(&maximum)));
        }
        format!("{{{}}}", parts.join(","))
    }
}

/// A fluent, order-preserving builder for an action's JSON Schema and UI Schema.
///
/// See the [module docs](self) for an example.
#[derive(Debug, Clone)]
pub struct ActionSchema {
    title: String,
    description: String,
    fields: Vec<(String, FieldBuilder)>,
}

impl ActionSchema {
    /// Create a new schema with the given `title` and `description`.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            fields: Vec::new(),
        }
    }

    fn add_field(
        mut self,
        name: &str,
        field_type: &'static str,
        choices: Option<Vec<String>>,
        build: impl FnOnce(FieldBuilder) -> FieldBuilder,
    ) -> Self {
        let mut builder = FieldBuilder::new(field_type);
        builder.choices = choices;
        self.fields.push((name.to_string(), build(builder)));
        self
    }

    /// Add a text (`string`) field.
    pub fn text(self, name: &str, build: impl FnOnce(FieldBuilder) -> FieldBuilder) -> Self {
        self.add_field(name, "string", None, build)
    }

    /// Add a `number` field.
    pub fn number(self, name: &str, build: impl FnOnce(FieldBuilder) -> FieldBuilder) -> Self {
        self.add_field(name, "number", None, build)
    }

    /// Add an `integer` field.
    pub fn integer(self, name: &str, build: impl FnOnce(FieldBuilder) -> FieldBuilder) -> Self {
        self.add_field(name, "integer", None, build)
    }

    /// Add a `boolean` field.
    pub fn boolean(self, name: &str, build: impl FnOnce(FieldBuilder) -> FieldBuilder) -> Self {
        self.add_field(name, "boolean", None, build)
    }

    /// Add a select (`string` with `enum`) field with static choices, in order.
    pub fn select(
        self,
        name: &str,
        choices: &[&str],
        build: impl FnOnce(FieldBuilder) -> FieldBuilder,
    ) -> Self {
        let choices = choices.iter().map(|c| c.to_string()).collect();
        self.add_field(name, "string", Some(choices), build)
    }

    /// The action description passed to [`ActionSchema::new`].
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Serialize the JSON Schema to a string, preserving field declaration order.
    pub fn to_json_schema_string(&self) -> String {
        let properties = self
            .fields
            .iter()
            .map(|(name, field)| format!("{}:{}", encode(name), field.to_schema_string()))
            .collect::<Vec<_>>()
            .join(",");

        let mut out = format!(
            "{{\"title\":{},\"description\":{},\"type\":\"object\",\"properties\":{{{}}}",
            encode(&self.title),
            encode(&self.description),
            properties,
        );

        let required: Vec<&str> = self
            .fields
            .iter()
            .filter(|(_, field)| field.required)
            .map(|(name, _)| name.as_str())
            .collect();
        if !required.is_empty() {
            out.push_str(&format!(",\"required\":{}", encode(&required)));
        }
        out.push('}');
        out
    }

    /// Serialize the UI Schema to a string. Empty (`{}`) for now.
    pub fn to_ui_schema_string(&self) -> String {
        "{}".to_string()
    }

    /// Convenience returning `(json_schema, ui_schema, description)`, matching the
    /// tuple shape of [`Action::get_schema_json`](crate::Action::get_schema_json).
    pub fn to_schema_json(&self) -> (String, String, String) {
        (
            self.to_json_schema_string(),
            self.to_ui_schema_string(),
            self.description.clone(),
        )
    }
}

/// Encode a serializable value to a JSON string fragment.
fn encode<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("JSON schema fragment is always serializable")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn golden_multi_field_schema_preserves_declaration_order() {
        // Fields declared in deliberately non-alphabetical order (z, a, m, ...).
        let schema = ActionSchema::new("Widget", "Configure a widget")
            .number("z", |f| f.description("Z axis").required())
            .text("a", |f| f.title("Label"))
            .select("m", &["one", "two"], |f| f.required())
            .boolean("verbose", |f| f.default_value(json!(false)))
            .integer("count", |f| f.minimum(0.0).maximum(10.0));

        let expected = r#"{"title":"Widget","description":"Configure a widget","type":"object","properties":{"z":{"type":"number","description":"Z axis"},"a":{"type":"string","title":"Label"},"m":{"type":"string","enum":["one","two"]},"verbose":{"type":"boolean","default":false},"count":{"type":"integer","minimum":0.0,"maximum":10.0}},"required":["z","m"]}"#;
        assert_eq!(schema.to_json_schema_string(), expected);
    }

    #[test]
    fn required_omitted_when_empty() {
        let schema = ActionSchema::new("NoReq", "no required fields")
            .text("a", |f| f)
            .number("b", |f| f);
        let out = schema.to_json_schema_string();
        assert!(!out.contains("\"required\""), "got: {}", out);
    }

    #[test]
    fn select_emits_enum_in_given_order() {
        let schema = ActionSchema::new("S", "select").select("mode", &["fast", "precise"], |f| f);
        let out = schema.to_json_schema_string();
        assert!(
            out.contains(r#""mode":{"type":"string","enum":["fast","precise"]}"#),
            "got: {}",
            out
        );
    }

    #[test]
    fn output_round_trips_as_valid_json() {
        let schema = ActionSchema::new("Add Numbers", "Add two numbers together")
            .number("x", |f| f.description("First number").required())
            .number("y", |f| f.required())
            .select("mode", &["fast", "precise"], |f| f.required());

        let parsed: Value = serde_json::from_str(&schema.to_json_schema_string()).unwrap();
        assert_eq!(parsed["title"], "Add Numbers");
        assert_eq!(parsed["description"], "Add two numbers together");
        assert_eq!(parsed["type"], "object");
        assert_eq!(parsed["properties"]["x"]["type"], "number");
        assert_eq!(parsed["properties"]["x"]["description"], "First number");
        assert_eq!(
            parsed["properties"]["mode"]["enum"],
            json!(["fast", "precise"])
        );
        assert_eq!(parsed["required"], json!(["x", "y", "mode"]));

        // UI schema also parses.
        let ui: Value = serde_json::from_str(&schema.to_ui_schema_string()).unwrap();
        assert_eq!(ui, json!({}));
    }

    #[test]
    fn default_min_max_title_description_emitted() {
        let schema = ActionSchema::new("Opts", "options").number("n", |f| {
            f.title("N")
                .description("a number")
                .default_value(json!(1.5))
                .minimum(0.0)
                .maximum(9.0)
        });
        let parsed: Value = serde_json::from_str(&schema.to_json_schema_string()).unwrap();
        let n = &parsed["properties"]["n"];
        assert_eq!(n["title"], "N");
        assert_eq!(n["description"], "a number");
        assert_eq!(n["default"], json!(1.5));
        assert_eq!(n["minimum"], json!(0.0));
        assert_eq!(n["maximum"], json!(9.0));
    }

    #[test]
    fn to_schema_json_matches_tuple_shape() {
        let schema = ActionSchema::new("T", "desc").text("a", |f| f);
        let (json_schema, ui_schema, description) = schema.to_schema_json();
        assert_eq!(json_schema, schema.to_json_schema_string());
        assert_eq!(ui_schema, "{}");
        assert_eq!(description, "desc");
    }
}
