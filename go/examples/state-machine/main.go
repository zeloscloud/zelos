package main

import (
	"context"
	"log"
	"os"
	"time"

	zelos "github.com/zeloscloud/zelos/go"
)

func main() {
	ctx := context.Background()
	url := os.Getenv("ZELOS_URL")
	if url == "" {
		url = "grpc://127.0.0.1:2300"
	}

	_, sender, receiver := zelos.NewTraceRouter(ctx)
	config := zelos.DefaultTracePublishClientConfig()
	config.URL = url
	client := zelos.NewTracePublishClient(ctx, receiver, config)
	go func() { _ = client.Run() }()
	if err := client.WaitUntilConnected(5 * time.Second); err != nil {
		log.Fatal(err)
	}

	source, _ := zelos.NewTraceSource("state-machine-go", sender)
	defer source.Close()
	state, _ := source.BuildEvent("state").
		AddUint8Field("current", nil).
		AddUint8Field("previous", nil).
		AddFloat64Field("transition_time_ms", nil).
		Build()

	// Value tables
	values := map[*zelos.Value]string{
		zelos.NewUint8Value(0): "IDLE",
		zelos.NewUint8Value(1): "INIT",
		zelos.NewUint8Value(2): "RUNNING",
		zelos.NewUint8Value(3): "ERROR",
	}
	_ = source.AddValueTable("state", "current", values)
	_ = source.AddValueTable("state", "previous", values)

	transitions := []struct {
		prev, curr uint8
		ms         float64
	}{
		{0, 1, 12.3}, {1, 2, 5.4}, {2, 3, 1.1}, {3, 0, 20.7},
	}
	for _, t := range transitions {
		b, _ := state.Build().TryInsertUint8("current", t.curr)
		b, _ = b.TryInsertUint8("previous", t.prev)
		b, _ = b.TryInsertFloat64("transition_time_ms", t.ms)
		_ = b.Emit()
	}
}
