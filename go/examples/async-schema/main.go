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

	source, err := zelos.NewTraceSource("async-schema", sender)
	if err != nil {
		log.Fatal(err)
	}
	defer source.Close()

	// Register schema, then emit shortly after
	evt, err := source.BuildEvent("measurement").
		AddFloat64Field("value", nil).
		Build()
	if err != nil {
		log.Fatal(err)
	}

	// Emit a couple of samples
	for i := 0; i < 3; i++ {
		b, err := evt.Build().TryInsertFloat64("value", float64(i))
		if err != nil {
			log.Fatal(err)
		}
		if err := b.Emit(); err != nil {
			log.Fatal(err)
		}
	}
}
