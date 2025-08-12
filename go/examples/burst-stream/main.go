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

	source, err := zelos.NewTraceSource("events-go", sender)
	if err != nil {
		log.Fatal(err)
	}
	defer source.Close()

	startEvt, _ := source.BuildEvent("event_start").AddStringField("trigger", nil).AddInt64Field("timestamp_ms", nil).Build()
	sampleEvt, _ := source.BuildEvent("event_sample").AddFloat64Field("value", nil).AddUint32Field("index", nil).Build()
	endEvt, _ := source.BuildEvent("event_end").AddFloat64Field("duration_ms", nil).AddUint32Field("sample_count", nil).Build()

	for {
		time.Sleep(5 * time.Second)
		start := time.Now()
		b, _ := startEvt.Build().TryInsertString("trigger", "threshold_exceeded")
		b, _ = b.TryInsertInt64("timestamp_ms", start.UnixMilli())
		_ = b.Emit()

		for i := 0; i < 100; i++ {
			b, _ := sampleEvt.Build().TryInsertFloat64("value", float64(i)*0.1)
			b, _ = b.TryInsertUint32("index", uint32(i))
			_ = b.Emit()
		}

		dur := time.Since(start).Seconds() * 1000
		b, _ = endEvt.Build().TryInsertFloat64("duration_ms", dur)
		b, _ = b.TryInsertUint32("sample_count", 100)
		_ = b.Emit()
	}
}
