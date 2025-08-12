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

	source, err := zelos.NewTraceSource("all-types", sender)
	if err != nil {
		log.Fatal(err)
	}
	defer source.Close()

	evt, err := source.BuildEvent("all_types").
		AddInt8Field("i8", nil).
		AddInt16Field("i16", nil).
		AddInt32Field("i32", nil).
		AddInt64Field("i64", nil).
		AddUint8Field("u8", nil).
		AddUint16Field("u16", nil).
		AddUint32Field("u32", nil).
		AddUint64Field("u64", nil).
		AddFloat32Field("f32", nil).
		AddFloat64Field("f64", nil).
		AddTimestampNsField("ts", stringPtr("ns")).
		AddBinaryField("bin", nil).
		AddStringField("str", nil).
		AddBooleanField("bool", nil).
		Build()
	if err != nil {
		log.Fatal(err)
	}

	b, err := evt.Build().
		TryInsertInt8("i8", -8)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertInt16("i16", -16)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertInt32("i32", -32)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertInt64("i64", -64)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertUint8("u8", 8)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertUint16("u16", 16)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertUint32("u32", 32)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertUint64("u64", 64)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertFloat32("f32", float32(3.1415927))
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertFloat64("f64", 2.718281828)
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertTimestampNs("ts", zelos.NowTimeNs())
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertBinary("bin", []byte{0x01, 0x02, 0x03})
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertString("str", "hello")
	if err != nil {
		log.Fatal(err)
	}
	b, err = b.TryInsertBoolean("bool", true)
	if err != nil {
		log.Fatal(err)
	}
	if err := b.Emit(); err != nil {
		log.Fatal(err)
	}
}

func stringPtr(s string) *string { return &s }
