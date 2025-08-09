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
	config := &zelos.TracePublishClientConfig{
		URL:            url,
		BatchSize:      256,
		BatchTimeout:   50 * time.Millisecond,
		ReconnectDelay: 500 * time.Millisecond,
	}
	client := zelos.NewTracePublishClient(ctx, receiver, config)
	go func() { _ = client.Run() }()
	if err := client.WaitUntilConnected(5 * time.Second); err != nil {
		log.Fatal(err)
	}

	source, err := zelos.NewTraceSource("publish-config-demo", sender)
	if err != nil {
		log.Fatal(err)
	}
	defer source.Close()

	evt, err := source.BuildEvent("sample").AddUint64Field("n", nil).Build()
	if err != nil {
		log.Fatal(err)
	}

	for n := uint64(1); n <= 10; n++ {
		b, err := evt.Build().TryInsertUint64("n", n)
		if err != nil {
			log.Fatal(err)
		}
		if err := b.Emit(); err != nil {
			log.Fatal(err)
		}
	}
}
