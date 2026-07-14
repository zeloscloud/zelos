package zelos

import (
	"context"
	"encoding/json"
	"errors"
	"sort"
	"sync"
	"time"

	actionspb "github.com/zeloscloud/zelos/go/zeloscloud/actions"
)

// ExecuteStatus mirrors the actions.ExecuteStatus proto enum. It classifies the
// outcome of an action execution.
type ExecuteStatus int

const (
	ExecuteStatusPass  ExecuteStatus = iota // Action completed and passed
	ExecuteStatusFail                       // Action completed but failed a check
	ExecuteStatusError                      // Action could not complete (system-level error)
	ExecuteStatusDone                       // Action completed without a pass/fail classification
)

// proto converts an ExecuteStatus to its protobuf enum value.
func (s ExecuteStatus) proto() actionspb.ExecuteStatus {
	switch s {
	case ExecuteStatusPass:
		return actionspb.ExecuteStatus_PASS
	case ExecuteStatusFail:
		return actionspb.ExecuteStatus_FAIL
	case ExecuteStatusError:
		return actionspb.ExecuteStatus_ERROR
	case ExecuteStatusDone:
		return actionspb.ExecuteStatus_DONE
	default:
		return actionspb.ExecuteStatus_DONE
	}
}

// ActionResult is the outcome of executing an action. Value is marshaled to JSON
// for the wire.
type ActionResult struct {
	Value  any
	Status ExecuteStatus
}

// ActionResultPass builds a result with ExecuteStatusPass.
func ActionResultPass(value any) *ActionResult {
	return &ActionResult{Value: value, Status: ExecuteStatusPass}
}

// ActionResultFail builds a result with ExecuteStatusFail.
func ActionResultFail(value any) *ActionResult {
	return &ActionResult{Value: value, Status: ExecuteStatusFail}
}

// ActionResultError builds a result with ExecuteStatusError.
func ActionResultError(value any) *ActionResult {
	return &ActionResult{Value: value, Status: ExecuteStatusError}
}

// ActionResultDone builds a result with ExecuteStatusDone.
func ActionResultDone(value any) *ActionResult {
	return &ActionResult{Value: value, Status: ExecuteStatusDone}
}

// Action is a provider-side action that can be listed, described, and executed
// by the agent over the Actions stream.
type Action interface {
	// Execute runs the action. params is the raw JSON parameters object;
	// implementations typically json.Unmarshal into their own struct. Actions
	// should stop promptly when ctx is cancelled.
	Execute(ctx context.Context, params json.RawMessage) (*ActionResult, error)
	// Schema returns the action's JSON Schema and UI Schema as JSON strings.
	// The strings are sent verbatim over the wire, so property order is
	// preserved (never round-trip through map[string]any, which would reorder
	// keys).
	Schema(currentValues json.RawMessage) (actionSchema, uiSchema string, err error)
	// DefaultTimeout returns the action's default execution timeout; 0 means
	// none.
	DefaultTimeout() time.Duration
}

// actionFunc adapts a plain function into an Action with a static schema, an
// empty UI schema, and no default timeout.
type actionFunc struct {
	actionSchema string
	fn           func(ctx context.Context, params json.RawMessage) (*ActionResult, error)
}

// NewAction builds an Action from a static JSON Schema string and an execute
// function. The UI schema is "{}" and there is no default timeout.
func NewAction(actionSchema string, fn func(ctx context.Context, params json.RawMessage) (*ActionResult, error)) Action {
	return &actionFunc{actionSchema: actionSchema, fn: fn}
}

func (a *actionFunc) Execute(ctx context.Context, params json.RawMessage) (*ActionResult, error) {
	if a.fn == nil {
		return nil, errors.New("action has no execute function")
	}
	return a.fn(ctx, params)
}

func (a *actionFunc) Schema(currentValues json.RawMessage) (string, string, error) {
	return a.actionSchema, "{}", nil
}

func (a *actionFunc) DefaultTimeout() time.Duration {
	return 0
}

// ActionsRegistry holds locally registered actions keyed by their full path.
type ActionsRegistry struct {
	mu      sync.RWMutex
	actions map[string]Action
}

// NewActionsRegistry creates an empty registry.
func NewActionsRegistry() *ActionsRegistry {
	return &ActionsRegistry{actions: make(map[string]Action)}
}

// Register adds (or replaces) an action at the given path. The path may include
// namespaces, e.g. "namespace/class/action". A nil action is rejected.
func (r *ActionsRegistry) Register(path string, a Action) error {
	if a == nil {
		return errors.New("action must not be nil")
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	r.actions[path] = a
	return nil
}

// Get returns the action registered at path, and whether it was found.
func (r *ActionsRegistry) Get(path string) (Action, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	a, ok := r.actions[path]
	return a, ok
}

// List returns all registered action paths, sorted for determinism.
func (r *ActionsRegistry) List() []string {
	r.mu.RLock()
	defer r.mu.RUnlock()
	paths := make([]string, 0, len(r.actions))
	for p := range r.actions {
		paths = append(paths, p)
	}
	sort.Strings(paths)
	return paths
}
