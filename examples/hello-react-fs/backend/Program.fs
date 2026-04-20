// Hello Gateorix — F# backend example.
//
// Run modes:
//   HTTP server (dev bridge):
//     dotnet run -- --http
//     → starts on http://localhost:3001
//
//   Stdio sidecar (production):
//     echo '{"id":"1","channel":"runtime.greet","payload":{"name":"Alice"}}' | dotnet run

open System
open System.Text.Json
open System.IO
open Microsoft.AspNetCore.Builder
open Microsoft.AspNetCore.Http

// --- Command handlers ---

let handleGreet (payload: JsonElement) =
    let mutable nameElem = Unchecked.defaultof<JsonElement>
    let name =
        if payload.TryGetProperty("name", &nameElem) then
            match nameElem.GetString() with
            | null | "" -> "World"
            | n -> n
        else "World"
    {| message = $"Hello from F#, {name}! Welcome to Gateorix." |}

let handleEcho (payload: JsonElement) = payload

let handlers: Map<string, JsonElement -> obj> =
    Map.ofList [
        "greet", (fun p -> handleGreet p :> obj)
        "echo", (fun p -> handleEcho p :> obj)
    ]

let dispatch (id: string) (channel: string) (payload: JsonElement) =
    let action =
        match channel.LastIndexOf('.') with
        | -1 -> channel
        | i -> channel.Substring(i + 1)

    match handlers.TryFind(action) with
    | None -> {| id = id; ok = false; payload = {| error = $"unknown command: {action}" |} |} :> obj
    | Some handler ->
        try
            {| id = id; ok = true; payload = handler payload |} :> obj
        with ex ->
            {| id = id; ok = false; payload = {| error = ex.Message |} |} :> obj

let jsonOpts = JsonSerializerOptions(PropertyNamingPolicy = JsonNamingPolicy.CamelCase)

let runStdio () =
    let mutable line = Console.ReadLine()
    while line <> null do
        if not (String.IsNullOrWhiteSpace(line)) then
            try
                use doc = JsonDocument.Parse(line)
                let root = doc.RootElement
                let id = root.GetProperty("id").GetString()
                let channel = root.GetProperty("channel").GetString()
                let payload = root.GetProperty("payload")
                let response = dispatch id channel payload
                Console.WriteLine(JsonSerializer.Serialize(response, jsonOpts))
            with :? JsonException ->
                Console.WriteLine(JsonSerializer.Serialize({| id = "unknown"; ok = false; payload = {| error = "invalid JSON" |} |}, jsonOpts))
        line <- Console.ReadLine()

let runHttp () =
    let builder = WebApplication.CreateBuilder([||])
    let app = builder.Build()

    app.Use(fun (ctx: HttpContext) (next: RequestDelegate) ->
        ctx.Response.Headers.Append("Access-Control-Allow-Origin", "http://localhost:5173")
        ctx.Response.Headers.Append("Access-Control-Allow-Methods", "POST, OPTIONS")
        ctx.Response.Headers.Append("Access-Control-Allow-Headers", "Content-Type")
        if ctx.Request.Method = "OPTIONS" then
            ctx.Response.StatusCode <- 204
            Threading.Tasks.Task.CompletedTask
        else
            next.Invoke(ctx)
    ) |> ignore

    app.MapGet("/invoke", Func<IResult>(fun () ->
        Results.Json({| status = "running"; usage = "POST /invoke with JSON body: {id, channel, payload}" |}))) |> ignore

    app.MapPost("/invoke", Func<HttpContext, Threading.Tasks.Task<IResult>>(fun ctx ->
        task {
            use! doc = JsonDocument.ParseAsync(ctx.Request.Body)
            let root = doc.RootElement
            let id = root.GetProperty("id").GetString()
            let channel = root.GetProperty("channel").GetString()
            let payload = root.GetProperty("payload")
            return Results.Json(dispatch id channel payload)
        })) |> ignore

    app.MapGet("/health", Func<IResult>(fun () -> Results.Json({| status = "ok" |}))) |> ignore

    Console.WriteLine("\n  Gateorix HTTP dev bridge (F#)")
    Console.WriteLine("  → listening on http://localhost:3001/invoke")
    Console.WriteLine("  → CORS allowed: http://localhost:5173\n")
    app.Run("http://localhost:3001")

[<EntryPoint>]
let main args =
    if args |> Array.contains "--http" then
        runHttp ()
    else
        runStdio ()
    0
