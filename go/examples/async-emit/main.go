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
	go func() {
		if err := client.Run(); err != nil && err != context.Canceled {
			log.Printf("client error: %v", err)
		}
	}()
	if err := client.WaitUntilConnected(5 * time.Second); err != nil {
		log.Fatalf("connect failed: %v", err)
	}

	source, err := zelos.NewTraceSource("async-emit-go", sender)
	if err != nil {
		log.Fatal(err)
	}
	defer source.Close()

	evt, err := source.BuildEvent("progress").
		AddUint64Field("step", nil).
		AddStringField("message", nil).
		Build()
	if err != nil {
		log.Fatal(err)
	}

	b, err := evt.Build().TryInsertUint64("step", 1)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertString("message", "starting")
	if err != nil {
		log.Fatal(err)
	}
	if err := b.Emit(); err != nil {
		log.Fatal(err)
	}

	b, err = evt.Build().TryInsertUint64("step", 2)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertString("message", "working")
	if err != nil {
		log.Fatal(err)
	}
	if err := b.EmitAt(zelos.NowTimeNs()); err != nil {
		log.Fatal(err)
	}
}
