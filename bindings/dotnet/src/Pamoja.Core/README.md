# Pamoja.Core

.NET bindings for the [pamoja](https://github.com/molexxxx/pamoja) device SDK core, a single memory-safe Rust engine for IoT, robotics, and drones.

The package ships in two tiers. The default surface is a hand-written, idiomatic facade in `Pamoja.Core`; the low-level escape hatch is the P/Invoke layer in `Pamoja.Core.Interop`, a one-to-one mirror of the generated C ABI. A prebuilt native library is bundled per runtime identifier, so there is nothing to compile.

## Install

```
dotnet add package Pamoja.Core
```

## Quick look

```csharp
using Pamoja.Core;

await using var client = new MqttClient(new MqttClientOptions
{
    ClientId = "sensor-1",
    Host = "localhost",
    Port = 1883,
});

await client.ConnectAsync();
await client.SubscribeAsync("sensors/+/temperature");
await client.PublishAsync("sensors/1/temperature", "21.5");

await foreach (var message in client)
{
    Console.WriteLine($"{message.Topic}: {message.Payload.Length} bytes");
}
```

Errors surface as `PamojaException`, the incoming-message stream is an
`IAsyncEnumerable<MqttMessage>`, and the client implements `IAsyncDisposable`.

## License

MIT
