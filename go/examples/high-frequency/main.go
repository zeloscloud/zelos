package main

import (
	"context"
	"log"
	"math"
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

	source, err := zelos.NewTraceSource("high-frequency", sender)
	if err != nil {
		log.Fatal(err)
	}
	defer source.Close()

	dataEvt, err := source.BuildEvent("data").AddFloat64Field("value", stringPtr("V")).Build()
	if err != nil {
		log.Fatal(err)
	}

	period := time.Millisecond // 1 kHz
	start := time.Now()
	for {
		elapsed := time.Since(start).Seconds()
		value := math.Sin(2 * math.Pi * 100 * elapsed)
		b, err := dataEvt.Build().TryInsertFloat64("value", value)
		if err != nil {
			log.Fatal(err)
		}
		if err := b.Emit(); err != nil {
			log.Fatal(err)
		}
		time.Sleep(period)
	}
}

func stringPtr(s string) *string { return &s }
