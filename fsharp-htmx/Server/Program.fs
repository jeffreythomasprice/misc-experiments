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

module Views =
    open Giraffe.ViewEngine

    let htmlPage (content: XmlNode list) =
        html
            []
            [ head
                  []
                  [ title [] [ encodedText "F# Experiment" ]
                    link [ _rel "stylesheet"; _type "text/css"; _href "/main.css" ]
                    script [ _src "https://unpkg.com/htmx.org@1.9.7" ] []
                    script [] [ Text @"htmx.logAll()" ] ]
              body [] content ]
        |> htmlView

    let index () =
        [ button [ KeyValue("hx-post", "/click"); KeyValue("hx-target", "#clickResults") ] [ encodedText "Click Me" ]
          div [ _id "clickResults" ] [] ]
        |> htmlPage

    let clicks (clicks: int) = Text $"{clicks}" |> htmlView

let mutable clicks = 0

let clickHandler () =
    clicks <- clicks + 1
    Views.clicks (clicks)

let webApp =
    choose
        [ choose
              [ GET >=> route "/" >=> Views.index ()
                POST >=> route "/click" >=> warbler (fun _ -> clickHandler ()) ]
          // TODO better 404 page
          setStatusCode 404 >=> text "Not Found" ]

let errorHandler (ex: Exception) (logger: ILogger) =
    logger.LogError(ex, "An unhandled exception has occurred while executing the request.")
    // TODO better 500 page
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
