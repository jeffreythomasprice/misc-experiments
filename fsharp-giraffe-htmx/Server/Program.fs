module Experiment.Server.App

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
open Microsoft.Data.Sqlite
open System.Reflection
open Microsoft.AspNetCore.Http

module Routes =
    let ifAuthenticated (other: HttpHandler) : HttpHandler =
        fun (next: HttpFunc) (ctx: HttpContext) ->
            task {
                let extCtx = ctx.RequestServices.GetService<ExtendedContextService>()
                let! auth = extCtx.user

                if auth.IsSome then
                    return! other next ctx
                else
                    return! next ctx
            }

    let ifNotAuthenticated (other: HttpHandler) : HttpHandler =
        fun (next: HttpFunc) (ctx: HttpContext) ->
            task {
                let extCtx = ctx.RequestServices.GetService<ExtendedContextService>()
                let! auth = extCtx.user

                if auth.IsSome then
                    return! next ctx
                else
                    return! other next ctx
            }

    let htmxRedirect (location: string) : HttpHandler =
        fun (next: HttpFunc) (ctx: HttpContext) ->
            if ctx.TryGetRequestHeader("hx-request").IsSome then
                ctx.SetHttpHeader("hx-redirect", location)
                next ctx
            else
                redirectTo false location next ctx

    let redirectIfAuthenticated: HttpHandler = ifAuthenticated (htmxRedirect "/")

    let redirectIfNotAuthenticated: HttpHandler =
        ifNotAuthenticated (htmxRedirect "/login")

module Views =
    let htmlPage (content: XmlNode list) : HttpHandler =
        fun (next: HttpFunc) (ctx: HttpContext) ->
            let env = ctx.GetService<IWebHostEnvironment>()

            let result =
                html
                    []
                    [ head
                          []
                          [ title [] [ encodedText "F# Experiment" ]
                            link [ _rel "stylesheet"; _type "text/css"; _href "/index.css" ]
                            script [ _src "https://unpkg.com/htmx.org@1.9.9" ] []
                            if env.IsDevelopment() then
                                script [] [ Text @"htmx.logAll()" ] ]
                      body [] content ]
                |> htmlView

            result next ctx

    let index: HttpHandler =
        fun (next: HttpFunc) (ctx: HttpContext) ->
            task {
                let! user = ctx.GetService<ExtendedContextService>().user
                let _, _, user = user.Value

                let result =
                    [ div [] [ button [ KeyValue("hx-post", "/logout") ] [ encodedText "Log Out" ] ]
                      div [] [ encodedText (sprintf "Hello, %s!" user.username) ] ]
                    |> htmlPage

                return! result next ctx
            }

    let loginPage: HttpHandler =
        fun (next: HttpFunc) (ctx: HttpContext) ->
            let result =
                [ form
                      [ _id "login"; KeyValue("hx-post", "/login"); KeyValue("hx-swap", "none") ]
                      [ label [ _for "username" ] [ encodedText "Username:" ]
                        input [ _name "username"; _type "text"; _autofocus ]
                        label [ _for "password" ] [ encodedText "Password:" ]
                        input [ _name "password"; _type "password" ]
                        div [] []
                        div [] [ button [ _type "submit" ] [ encodedText "Login" ] ] ]
                  div [ _id "loginErrors" ] [] ]
                |> htmlPage

            result next ctx

    let loginFailure (message: string) =
        div [ _id "loginErrors"; _class "errors"; KeyValue("hx-swap-oob", "true") ] [ encodedText message ]
        |> htmlView

module APIs =
    [<CLIMutable>]
    type LoginRequest = { username: string; password: string }

    let login: HttpHandler =
        fun (next: HttpFunc) (ctx: HttpContext) ->
            task {
                let! request = ctx.BindFormAsync<LoginRequest>()

                let db = ctx.GetService<DBService>()
                let! result = db.checkUsernameAndPassword request.username request.password

                let response =
                    match result with
                    | Ok _ ->
                        let jwt = ctx.GetService<JWTService>()
                        let token, expirationTime = jwt.createToken request.username
                        ctx.Response.Cookies.Append("Authorization", token, CookieOptions(Expires = expirationTime))
                        Routes.htmxRedirect "/"
                    | Error BadCredentials -> Views.loginFailure "Invalid credentials."

                return! response next ctx
            }

    let logout: HttpHandler =
        fun (next: HttpFunc) (ctx: HttpContext) ->
            ctx.Response.Cookies.Delete("Authorization")
            Routes.htmxRedirect "/login" next ctx

let webApp =
    choose
        [ choose
              [ GET >=> route "/" >=> Routes.redirectIfNotAuthenticated >=> Views.index
                GET >=> route "/login" >=> Routes.redirectIfAuthenticated >=> Views.loginPage
                POST >=> route "/login" >=> APIs.login
                POST >=> route "/logout" >=> APIs.logout ]
          // TODO better 404 page
          setStatusCode 404 >=> text "Not Found" ]

let errorHandler (ex: Exception) (logger: ILogger) =
    logger.LogError(ex, "An unhandled exception has occurred while executing the request.")
    // TODO better 500 page
    clearResponse >=> setStatusCode 500 >=> text ex.Message

let configureCors (builder: CorsPolicyBuilder) =
    builder.AllowAnyOrigin().AllowAnyMethod().AllowAnyHeader() |> ignore

let configureApp =
    fun (app: IApplicationBuilder) ->
        let env = app.ApplicationServices.GetService<IWebHostEnvironment>()

        match env.IsDevelopment() with
        | true -> app.UseDeveloperExceptionPage()
        | false -> app.UseGiraffeErrorHandler(errorHandler).UseHttpsRedirection()
        |> ignore

        app.UseCors(configureCors).UseStaticFiles().UseGiraffe(webApp)

let configureServices (services: IServiceCollection) =
    services.AddCors().AddHttpContextAccessor().AddGiraffe() |> ignore

    services.AddSingleton<DBService>(fun _ ->
        let exeDir = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)
        let dbPath = Path.Combine(exeDir, "db.sqlite")

        DBService.create (new SqliteConnection $"Data Source={dbPath}")
        |> Async.AwaitTask
        |> Async.RunSynchronously)
    |> ignore

    services.AddSingleton<JWTService>() |> ignore

    services.AddScoped<ExtendedContextService>() |> ignore

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
                .Configure(Action<IApplicationBuilder>(configureApp))
                .ConfigureServices(configureServices)
                .ConfigureLogging(configureLogging)
            |> ignore)
        .Build()
        .Run()

    0
