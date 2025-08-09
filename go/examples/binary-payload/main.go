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

	source, _ := zelos.NewTraceSource("binary-demo-go", sender)
	defer source.Close()
	evt, _ := source.BuildEvent("blob").
		AddStringField("name", nil).
		AddBinaryField("data", nil).
		Build()

	payload := make([]byte, 256)
	for i := 0; i <= 255; i++ {
		payload[i] = byte(i)
	}
	b, _ := evt.Build().TryInsertString("name", "bytes_0_255")
	b, _ = b.TryInsertBinary("data", payload)
	_ = b.Emit()
}
