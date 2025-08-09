package main

import (
	"context"
	"log"
	"os"
	"strconv"
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

	source, _ := zelos.NewTraceSource("sensor-array-go", sender)
	defer source.Close()

	b := source.BuildEvent("array")
	for i := 0; i < 16; i++ {
		b = b.AddFloat32Field(field(i), nil)
	}
	array, _ := b.Build()

	for t := 0; t < 10; t++ {
		builder := array.Build()
		for i := 0; i < 16; i++ {
			builder, _ = builder.TryInsertFloat32(field(i), float32(t)*0.1+float32(i)*0.01)
		}
		_ = builder.Emit()
	}
}

func field(i int) string { return "sensor_" + strconv.Itoa(i) }
