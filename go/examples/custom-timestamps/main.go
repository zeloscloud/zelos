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

	source, _ := zelos.NewTraceSource("replay", sender)
	defer source.Close()
	measurement, _ := source.BuildEvent("measurement").AddFloat64Field("value", nil).Build()

	b, _ := measurement.Build().TryInsertFloat64("value", 1.0)
	_ = b.Emit()

	specific := int64(1699564234567890123)
	b, _ = measurement.Build().TryInsertFloat64("value", 2.0)
	_ = b.EmitAt(specific)

	past := zelos.NowTimeNs() - (60 * int64(time.Second))
	b, _ = measurement.Build().TryInsertFloat64("value", 3.0)
	_ = b.EmitAt(past)

	// Synchronized timestamps
	for i := 0; i < 5; i++ {
		b, _ = measurement.Build().TryInsertFloat64("value", 4.2)
		_ = b.EmitAt(zelos.NowTimeNs() + 123456)
		time.Sleep(100 * time.Millisecond)
	}
}
