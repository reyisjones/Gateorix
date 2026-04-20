// Hello Gateorix — C# backend example.
//
// Run modes:
//   HTTP server (dev bridge):
//     dotnet run -- --http
//     → starts on http://localhost:3001
//
//   Stdio sidecar (production):
//     echo '{"id":"1","channel":"runtime.greet","payload":{"name":"Alice"}}' | dotnet run

using System.Text.Json;
using System.Text.Json.Serialization;

var handlers = new Dictionary<string, Func<JsonElement, object?>>
{
    ["greet"] = payload =>
    {
        var name = payload.TryGetProperty("name", out var n) ? n.GetString() : "World";
        if (string.IsNullOrEmpty(name)) name = "World";
        return new { message = $"Hello from C#, {name}! Welcome to Gateorix." };
    },
    ["echo"] = payload => payload
};

object? Dispatch(string id, string channel, JsonElement payload)
{
    var action = channel.Contains('.') ? channel[(channel.LastIndexOf('.') + 1)..] : channel;
    if (!handlers.TryGetValue(action, out var handler))
        return new { id, ok = false, payload = new { error = $"unknown command: {action}" } };

    try
    {
        var result = handler(payload);
        return new { id, ok = true, payload = result };
    }
    catch (Exception ex)
    {
        return new { id, ok = false, payload = new { error = ex.Message } };
    }
}

var jsonOpts = new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase };

if (args.Contains("--http"))
{
    var builder = WebApplication.CreateBuilder(args);
    var app = builder.Build();

    app.Use(async (ctx, next) =>
    {
        ctx.Response.Headers["Access-Control-Allow-Origin"] = "http://localhost:5173";
        ctx.Response.Headers["Access-Control-Allow-Methods"] = "POST, OPTIONS";
        ctx.Response.Headers["Access-Control-Allow-Headers"] = "Content-Type";
        if (ctx.Request.Method == "OPTIONS") { ctx.Response.StatusCode = 204; return; }
        await next();
    });

    app.MapGet("/invoke", () => Results.Json(new
    {
        status = "running",
        usage = "POST /invoke with JSON body: {id, channel, payload}"
    }));

    app.MapPost("/invoke", async (HttpRequest req) =>
    {
        using var doc = await JsonDocument.ParseAsync(req.Body);
        var root = doc.RootElement;
        var id = root.GetProperty("id").GetString() ?? "";
        var channel = root.GetProperty("channel").GetString() ?? "";
        var payload = root.GetProperty("payload");
        return Results.Json(Dispatch(id, channel, payload));
    });

    app.MapGet("/health", () => Results.Json(new { status = "ok" }));

    Console.WriteLine("\n  Gateorix HTTP dev bridge (C#)");
    Console.WriteLine("  → listening on http://localhost:3001/invoke");
    Console.WriteLine("  → CORS allowed: http://localhost:5173\n");
    app.Run("http://localhost:3001");
}
else
{
    // Stdio sidecar mode
    string? line;
    while ((line = Console.ReadLine()) != null)
    {
        if (string.IsNullOrWhiteSpace(line)) continue;

        try
        {
            using var doc = JsonDocument.Parse(line);
            var root = doc.RootElement;
            var id = root.GetProperty("id").GetString() ?? "";
            var channel = root.GetProperty("channel").GetString() ?? "";
            var payload = root.GetProperty("payload");
            var response = Dispatch(id, channel, payload);
            Console.WriteLine(JsonSerializer.Serialize(response, jsonOpts));
        }
        catch (JsonException)
        {
            Console.WriteLine(JsonSerializer.Serialize(
                new { id = "unknown", ok = false, payload = new { error = "invalid JSON" } }, jsonOpts));
        }
    }
}
