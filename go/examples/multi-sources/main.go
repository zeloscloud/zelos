package main

import (
	"context"
	"log"
	"math/rand"
	"os"
	"time"

	zelos "github.com/zeloscloud/zelos/go"
)

func streamMotor(source *zelos.TraceSource) error {
	telemetry, err := source.BuildEvent("telemetry").
		AddFloat64Field("rpm", stringPtr("rpm")).
		AddFloat64Field("torque", stringPtr("Nm")).
		Build()
	if err != nil {
		return err
	}

	ticker := time.NewTicker(10 * time.Millisecond)
	defer ticker.Stop()

	for range ticker.C {
		b, _ := telemetry.Build().TryInsertFloat64("rpm", 2000+rand.Float64()*200-100)
		b, _ = b.TryInsertFloat64("torque", 50+rand.Float64()*10-5)
		if err := b.Emit(); err != nil {
			return err
		}
	}
	return nil
}

func streamBattery(source *zelos.TraceSource) error {
	status, err := source.BuildEvent("status").
		AddFloat64Field("voltage", stringPtr("V")).
		AddFloat64Field("current", stringPtr("A")).
		AddFloat64Field("soc", stringPtr("%")).
		Build()
	if err != nil {
		return err
	}

	ticker := time.NewTicker(1 * time.Second)
	defer ticker.Stop()
	soc := 85.0
	for range ticker.C {
		soc = maxf(soc-0.1, 20.0)
		b, _ := status.Build().TryInsertFloat64("voltage", 48.0+rand.Float64()-0.5)
		b, _ = b.TryInsertFloat64("current", rand.Float64()*60-10)
		b, _ = b.TryInsertFloat64("soc", soc)
		if err := b.Emit(); err != nil {
			return err
		}
	}
	return nil
}

func streamGPS(source *zelos.TraceSource) error {
	position, err := source.BuildEvent("position").
		AddFloat64Field("lat", stringPtr("deg")).
		AddFloat64Field("lon", stringPtr("deg")).
		AddFloat64Field("alt", stringPtr("m")).
		Build()
	if err != nil {
		return err
	}

	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()
	baseLat := 37.4419
	baseLon := -122.1430
	for range ticker.C {
		b, _ := position.Build().TryInsertFloat64("lat", baseLat+rand.Float64()*0.002-0.001)
		b, _ = b.TryInsertFloat64("lon", baseLon+rand.Float64()*0.002-0.001)
		b, _ = b.TryInsertFloat64("alt", 30+rand.Float64()*2-1)
		if err := b.Emit(); err != nil {
			return err
		}
	}
	return nil
}

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

	motor, _ := zelos.NewTraceSource("motor", sender)
	battery, _ := zelos.NewTraceSource("battery", sender)
	gps, _ := zelos.NewTraceSource("gps", sender)

	go func() { _ = streamMotor(motor) }()
	go func() { _ = streamBattery(battery) }()
	go func() { _ = streamGPS(gps) }()

	select {}
}

func stringPtr(s string) *string { return &s }
func maxf(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}
