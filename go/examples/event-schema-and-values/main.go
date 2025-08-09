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

	source, err := zelos.NewTraceSource("status-demo", sender)
	if err != nil {
		log.Fatal(err)
	}
	defer source.Close()

	statusEvt, err := source.BuildEvent("status").
		AddUint8Field("status_code", nil).
		AddStringField("detail", nil).
		Build()
	if err != nil {
		log.Fatal(err)
	}

	valueTable := map[*zelos.Value]string{
		zelos.NewUint8Value(0): "idle",
		zelos.NewUint8Value(1): "busy",
		zelos.NewUint8Value(2): "error",
	}
	if err := source.AddValueTable("status", "status_code", valueTable); err != nil {
		log.Fatal(err)
	}

	b, err := statusEvt.Build().TryInsertUint8("status_code", 1)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertString("detail", "processing request")
	if err != nil {
		log.Fatal(err)
	}
	if err := b.Emit(); err != nil {
		log.Fatal(err)
	}
}
