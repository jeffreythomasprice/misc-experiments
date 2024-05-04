module Experiment.Server

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
open Microsoft.AspNetCore.Http

let mutable clicks = 0

// TODO JEFF no, logger should be easier
type loggerPlaceholderType = interface end

let getLogger (ctx: HttpContext) =
    let t = typeof<loggerPlaceholderType>.DeclaringType
    ctx.GetLogger(t.FullName)

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
                  button [ attr "hx-post" "/click"; attr "hx-target" "#clickResults" ] [ encodedText "Click me" ]
                  form
                      [ attr "hx-post" "/formTest"; attr "hx-target" "#formOutput" ]
                      [ div
                            [ _class "form" ]
                            [ label [ _for "foo" ] [ encodedText "Foo" ]
                              input [ _name "foo"; _type "text" ]
                              label [ _for "bar" ] [ encodedText "Bar" ]
                              input [ _name "bar"; _type "text" ] ]
                        button [ _type "submit" ] [ encodedText "Submit" ] ]
                  div [ _id "formOutput" ] [] ]
        ))

let clicker =
    warbler (fun _ ->
        clicks <- clicks + 1
        htmlView (encodedText $"Clicks: {clicks}"))

[<CLIMutable>]
type Request = { foo: string; bar: string }

let formTest: HttpHandler =
    let makeResponse request =
        htmlView (div [] [ div [] [ encodedText request.foo ]; div [] [ encodedText request.bar ] ])

    fun (next) (ctx) ->
        let logger = ctx |> getLogger
        let logger2 = ctx.GetLogger()
        logger.LogInformation("test1")
        logger2.LogInformation("test2")

        task {
            let! request = ctx.BindFormAsync<Request>()
            logger.LogInformation($"request = {request}")
            return! makeResponse request next ctx
        }

let webApp =
    choose
        [ choose
              [ route "/" >=> GET >=> index
                route "/click" >=> POST >=> clicker
                route "/formTest" >=> POST >=> formTest ]
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
