package main

import (
	"context"
	"log"
	"math/rand"
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

	source, err := zelos.NewTraceSource("sensors", sender)
	if err != nil {
		log.Fatal(err)
	}
	defer source.Close()

	temp, err := source.BuildEvent("temperature").AddFloat64Field("value", stringPtr("°C")).Build()
	if err != nil {
		log.Fatal(err)
	}

	for {
		value := 20.0 + rand.Float64()*4 - 2
		b, err := temp.Build().TryInsertFloat64("value", value)
		if err != nil {
			log.Fatal(err)
		}
		if err := b.Emit(); err != nil {
			log.Fatal(err)
		}
		time.Sleep(1 * time.Second)
	}
}

func stringPtr(s string) *string { return &s }
