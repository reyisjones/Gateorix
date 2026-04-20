using System.Text.Json;
using System.Text.Json.Serialization;

namespace Gateorix;

/// <summary>
/// Request message from the host core to the sidecar.
/// </summary>
public record AdapterRequest
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = "";

    [JsonPropertyName("channel")]
    public string Channel { get; init; } = "";

    [JsonPropertyName("payload")]
    public JsonElement Payload { get; init; }
}

/// <summary>
/// Response message from the sidecar to the host core.
/// </summary>
public record AdapterResponse
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = "";

    [JsonPropertyName("ok")]
    public bool Ok { get; init; }

    [JsonPropertyName("payload")]
    public object? Payload { get; init; }
}

/// <summary>
/// Delegate for handling Gateorix adapter commands.
/// </summary>
public delegate object? CommandHandler(JsonElement payload);

/// <summary>
/// Gateorix runtime adapter for .NET.
/// Implements the adapter protocol over stdio (newline-delimited JSON).
///
/// <example>
/// <code>
/// var adapter = new GateorixAdapter();
/// adapter.Command("greet", payload =>
/// {
///     var name = payload.TryGetProperty("name", out var n) ? n.GetString() : "World";
///     return new { message = $"Hello, {name}!" };
/// });
/// adapter.Run();
/// </code>
/// </example>
/// </summary>
public class GateorixAdapter
{
    private readonly Dictionary<string, CommandHandler> _handlers = new();

    /// <summary>
    /// Register a command handler for the given channel name.
    /// The "runtime." prefix is stripped automatically when matching.
    /// </summary>
    public void Command(string name, CommandHandler handler)
    {
        _handlers[name] = handler;
    }

    private AdapterResponse Dispatch(AdapterRequest request)
    {
        var channel = request.Channel;
        if (channel.StartsWith("runtime."))
            channel = channel[8..];

        if (!_handlers.TryGetValue(channel, out var handler))
        {
            return new AdapterResponse
            {
                Id = request.Id,
                Ok = false,
                Payload = new { error = $"unknown command: {request.Channel}" }
            };
        }

        try
        {
            var result = handler(request.Payload);
            return new AdapterResponse
            {
                Id = request.Id,
                Ok = true,
                Payload = result
            };
        }
        catch (Exception ex)
        {
            return new AdapterResponse
            {
                Id = request.Id,
                Ok = false,
                Payload = new { error = ex.Message }
            };
        }
    }

    /// <summary>
    /// Start the stdio message loop. Reads JSON from stdin, dispatches
    /// to handlers, writes JSON to stdout. Blocks until stdin is closed.
    /// </summary>
    public void Run()
    {
        var options = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase
        };

        string? line;
        while ((line = Console.ReadLine()) != null)
        {
            if (string.IsNullOrWhiteSpace(line))
                continue;

            AdapterRequest? request;
            try
            {
                request = JsonSerializer.Deserialize<AdapterRequest>(line);
            }
            catch (JsonException ex)
            {
                var errorResp = new AdapterResponse
                {
                    Id = "",
                    Ok = false,
                    Payload = new { error = $"invalid JSON: {ex.Message}" }
                };
                Console.WriteLine(JsonSerializer.Serialize(errorResp, options));
                continue;
            }

            if (request == null)
                continue;

            var response = Dispatch(request);
            Console.WriteLine(JsonSerializer.Serialize(response, options));
        }
    }

    /// <summary>
    /// Start an HTTP server using ASP.NET Minimal API.
    /// The host core sends POST /invoke with a JSON body.
    /// </summary>
    public void RunHttp(string url = "http://localhost:3001")
    {
        var builder = WebApplication.CreateBuilder();
        var app = builder.Build();

        app.MapPost("/invoke", (AdapterRequest request) =>
        {
            var response = Dispatch(request);
            return Results.Json(response);
        });

        app.MapGet("/health", () => Results.Json(new { status = "ok" }));

        Console.WriteLine($"Gateorix .NET adapter listening on {url}");
        app.Run(url);
    }
}
