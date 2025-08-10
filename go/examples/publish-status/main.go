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
		log.Fatal(err)
	}

	source, _ := zelos.NewTraceSource("publish-status-demo", sender)
	defer source.Close()
	evt, _ := source.BuildEvent("sample").AddUint64Field("n", nil).Build()

	for n := uint64(1); n <= 5; n++ {
		b, _ := evt.Build().TryInsertUint64("n", n)
		if err := b.Emit(); err != nil {
			log.Printf("emit error: %v", err)
		}
	}

	log.Printf("status: %s", client.GetConnectionStatus().String())
}
