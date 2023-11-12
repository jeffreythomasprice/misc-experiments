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
open System.Globalization
open Microsoft.Data.Sqlite
open Dapper
open System.Reflection
open System.Data.Common
open Microsoft.AspNetCore.Http

let initDb (db: DbConnection) =
    task {
        let! _ =
            db.ExecuteAsync
                @"CREATE TABLE IF NOT EXISTS users (
                    username STRING NOT NULL UNIQUE PRIMARY KEY,
                    password STRING NOT NULL
                )"

        let! _ = db.ExecuteAsync @"INSERT OR IGNORE INTO users (username, password) VALUES (""admin"", ""admin"")"
        ()
    }

type CredentialsCheck =
    | Success
    | BadCredentials

type Db(db: DbConnection) =
    member this.CheckUsernameAndPassword (username: string) (password: string) =
        task {
            let! count =
                db.ExecuteScalarAsync<int>(
                    "SELECT COUNT(*) FROM users WHERE username = @username AND password = @password",
                    {| username = username
                       password = password |}
                )

            return
                match count with
                | 1 -> Success
                | _ -> BadCredentials
        }

module Views =
    open Giraffe.ViewEngine

    let htmlPage (content: XmlNode list) =
        html
            []
            [ head
                  []
                  [ title [] [ encodedText "F# Experiment" ]
                    link [ _rel "stylesheet"; _type "text/css"; _href "/index.css" ]
                    script [ _src "https://unpkg.com/htmx.org@1.9.7" ] []
                    // TODO only in debug mode?
                    script [] [ Text @"htmx.logAll()" ] ]
              body [] content ]
        |> htmlView

    let index () =
        [ form
              [ _id "login"; KeyValue("hx-post", "/login"); KeyValue("hx-swap", "none") ]
              [ label [ _for "username" ] [ encodedText "Username:" ]
                input [ _name "username"; _type "text" ]
                label [ _for "password" ] [ encodedText "Password:" ]
                input [ _name "password"; _type "password" ]
                div [] []
                div [] [ button [ _type "submit" ] [ encodedText "Login" ] ] ]
          div [ _id "loginErrors" ] [] ]
        |> htmlPage

    let clicks (clicks: int) = Text $"{clicks}" |> htmlView

    let loginSuccess (username: string) =
        div [ KeyValue("hx-swap-oob", "innerHTML:body") ] [ encodedText $"TODO login success: {username}" ]
        |> htmlView

    let loginFailure (message: string) =
        div
            [ _id "loginErrors"; _class "errors"; KeyValue("hx-swap-oob", "true") ]
            [ encodedText $"TODO login failure: {message}" ]
        |> htmlView

[<CLIMutable>]
type LoginRequest = { username: string; password: string }

let loginHandler (db: Db) : HttpHandler =
    fun (next: HttpFunc) (ctx: HttpContext) ->
        task {
            let! request = ctx.BindFormAsync<LoginRequest>()

            let! result = db.CheckUsernameAndPassword request.username request.password

            let response =
                match result with
                | Success -> Views.loginSuccess request.username
                | BadCredentials -> Views.loginFailure "invalid credentials"

            return! response next ctx
        }

let webApp db =
    choose
        [ choose
              [ GET >=> route "/" >=> Views.index ()
                POST >=> route "/login" >=> loginHandler db ]
          // TODO better 404 page
          setStatusCode 404 >=> text "Not Found" ]

let errorHandler (ex: Exception) (logger: ILogger) =
    logger.LogError(ex, "An unhandled exception has occurred while executing the request.")
    // TODO better 500 page
    clearResponse >=> setStatusCode 500 >=> text ex.Message

let configureCors (builder: CorsPolicyBuilder) =
    builder.AllowAnyOrigin().AllowAnyMethod().AllowAnyHeader() |> ignore

let configureApp db =
    fun (app: IApplicationBuilder) ->
        let env = app.ApplicationServices.GetService<IWebHostEnvironment>()

        (match env.IsDevelopment() with
         | true -> app.UseDeveloperExceptionPage()
         | false -> app.UseGiraffeErrorHandler(errorHandler).UseHttpsRedirection())
            .UseCors(configureCors)
            .UseStaticFiles()
            .UseGiraffe(webApp db)

let configureServices (services: IServiceCollection) =
    services.AddCors() |> ignore
    services.AddGiraffe() |> ignore

let configureLogging (builder: ILoggingBuilder) =
    builder.AddConsole().AddDebug() |> ignore

[<EntryPoint>]
let main args =
    let contentRoot = Directory.GetCurrentDirectory()
    let webRoot = Path.Combine(contentRoot, "WebRoot")
    let exeDir = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)
    let dbPath = Path.Combine(exeDir, "db.sqlite")

    use db = new SqliteConnection $"Data Source={dbPath}"
    initDb db |> Async.AwaitTask |> Async.RunSynchronously

    Host
        .CreateDefaultBuilder(args)
        .ConfigureWebHostDefaults(fun webHostBuilder ->
            webHostBuilder
                .UseUrls("http://localhost:8000")
                .UseContentRoot(contentRoot)
                .UseWebRoot(webRoot)
                .Configure(Action<IApplicationBuilder>(configureApp (Db db)))
                .ConfigureServices(configureServices)
                .ConfigureLogging(configureLogging)
            |> ignore)
        .Build()
        .Run()

    0
