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
open Giraffe.ViewEngine

let mutable clicks = 0

let page (content: XmlNode list) =
    html
        []
        [ head
              []
              [ link [ _rel "stylesheet"; _type "text/css"; _href "/main.css" ]
                script [ _src "https://unpkg.com/htmx.org@1.9.12" ] []
                script [] [ rawText "htmx.logAll();" ] ]
          body [] content ]

let index =
    warbler (fun _ ->
        htmlView (
            page
                [ div [ _id "clickResults" ] [ encodedText $"Clicks: {clicks}" ]
                  button [ attr "hx-post" "/click"; attr "hx-target" "#clickResults" ] [ encodedText "Click me" ] ]
        ))

let clicker =
    warbler (fun _ ->
        clicks <- clicks + 1
        htmlView (encodedText $"Clicks: {clicks}"))

let webApp =
    choose
        [ choose [ route "/" >=> GET >=> index; route "/click" >=> POST >=> clicker ]
          // TODO pretty not found
          setStatusCode 404 >=> text "Not Found" ]

let errorHandler (ex: Exception) (logger: ILogger) =
    logger.LogError(ex, "An unhandled exception has occurred while executing the request.")
    // TODO pretty errors
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
                .UseUrls("http://localhost:8000")
                .UseContentRoot(contentRoot)
                .UseWebRoot(webRoot)
                .Configure(Action<IApplicationBuilder> configureApp)
                .ConfigureServices(configureServices)
                .ConfigureLogging(configureLogging)
            |> ignore)
        .Build()
        .Run()

    0
