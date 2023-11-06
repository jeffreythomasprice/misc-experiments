module Server.App

open System
open System.IO
open Microsoft.AspNetCore.Builder
open Microsoft.AspNetCore.Cors.Infrastructure
open Microsoft.AspNetCore.Hosting
open Microsoft.Extensions.Hosting
open Microsoft.Extensions.Logging
open Microsoft.Extensions.DependencyInjection
open Giraffe
open Microsoft.AspNetCore.Http
open System.Net.WebSockets
open System.Threading

[<CLIMutable>]
type HelloWorldResponse = { message: string }

let helloWorldHandler: HttpHandler =
    fun (next: HttpFunc) (ctx: HttpContext) ->
        task {
            let response: HelloWorldResponse = { message = "Hello, World!" }
            do! Async.Sleep(TimeSpan.FromSeconds 2)
            return! Successful.OK response next ctx
        }

let websocketHandler: HttpHandler =
    fun (next: HttpFunc) (ctx: HttpContext) ->
        task {
            if ctx.WebSockets.IsWebSocketRequest then
                let! ws = ctx.WebSockets.AcceptWebSocketAsync()

                task {
                    do!
                        ws.SendAsync(
                            ArraySegment(Text.Encoding.UTF8.GetBytes("sent from server")),
                            WebSocketMessageType.Text,
                            true,
                            CancellationToken.None
                        )
                }
                |> ignore

                let buffer = Array.zeroCreate 2048

                while not ws.CloseStatus.HasValue do
                    let! result = ws.ReceiveAsync(ArraySegment(buffer), CancellationToken.None)

                    printfn
                        "TODO JEFF result, end of message = %b, count = %i type = %A"
                        result.EndOfMessage
                        result.Count
                        result.MessageType

                    if result.MessageType = WebSocketMessageType.Text then
                        let s = Text.Encoding.UTF8.GetString(buffer, 0, result.Count)
                        printfn "received websocket message: %s" s
                        ()
                    else
                        printfn "unhandled websocket message type %A" result.MessageType

                ()

                do! ws.CloseAsync(WebSocketCloseStatus.NormalClosure, "", CancellationToken.None)
                return None
            else
                return! ServerErrors.INTERNAL_ERROR (text "expected websocket request") next ctx
        }

let webApp =
    choose
        [ choose
              [ GET >=> route "/" >=> helloWorldHandler
                GET >=> route "/ws" >=> websocketHandler ]
          setStatusCode 404 >=> text "Not Found" ]

let errorHandler (ex: Exception) (logger: ILogger) =
    logger.LogError(ex, "An unhandled exception has occurred while executing the request.")
    clearResponse >=> setStatusCode 500 >=> text ex.Message

let configureCors (builder: CorsPolicyBuilder) =
    builder.AllowAnyOrigin().AllowAnyMethod().AllowAnyHeader() |> ignore

let configureApp (app: IApplicationBuilder) =
    let env = app.ApplicationServices.GetService<IWebHostEnvironment>()

    (match env.IsDevelopment() with
     | true -> app.UseDeveloperExceptionPage()
     | false -> app.UseGiraffeErrorHandler(errorHandler).UseHttpsRedirection())
        .UseCors(configureCors)
        .UseStaticFiles()
        .UseWebSockets()
        .UseGiraffe(webApp)

let configureServices (services: IServiceCollection) =
    services.AddCors() |> ignore
    services.AddGiraffe() |> ignore

let configureLogging (builder: ILoggingBuilder) =
    builder.AddConsole().AddDebug() |> ignore

[<EntryPoint>]
let main args =
    let contentRoot = Directory.GetCurrentDirectory()
    let webRoot = Path.Combine(contentRoot, "WebRoot")

    Host
        .CreateDefaultBuilder(args)
        .ConfigureWebHostDefaults(fun webHostBuilder ->
            webHostBuilder
                .UseUrls("http://127.0.0.1:8001")
                .UseContentRoot(contentRoot)
                .UseWebRoot(webRoot)
                .Configure(Action<IApplicationBuilder> configureApp)
                .ConfigureServices(configureServices)
                .ConfigureLogging(configureLogging)
            |> ignore)
        .Build()
        .Run()

    0
