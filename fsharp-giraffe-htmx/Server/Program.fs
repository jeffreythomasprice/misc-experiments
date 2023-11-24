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
open System.Globalization
open Microsoft.Data.Sqlite
open Dapper
open System.Reflection
open System.Data.Common
open Microsoft.AspNetCore.Http
open Microsoft.AspNetCore.Authentication.JwtBearer
open Microsoft.IdentityModel.Tokens
open System.Text
open System.IdentityModel.Tokens.Jwt
open System.Security.Claims
open System.Collections
open System.Threading.Tasks

// TODO move various services out to their own files

type JWTService(log: ILogger<JWTService>) =
    // TODO use certs?
    let signingKey =
        SymmetricSecurityKey(Encoding.UTF8.GetBytes("TODO signing key some more bits to get the key size up enough"))

    member this.tokenValidationParameters =
        TokenValidationParameters(
            ValidateIssuer = true,
            ValidateAudience = true,
            ValidateLifetime = true,
            ValidateIssuerSigningKey = true,
            IssuerSigningKey = signingKey
        )

    member this.createToken(username: string) =
        let expirationTime = DateTime.Now.AddHours 1

        let result =
            JwtSecurityToken(
                null,
                null,
                [ Claim("username", username) ],
                DateTime.Now,
                expirationTime,
                this.signingCredentials
            )

        let result = (JwtSecurityTokenHandler().WriteToken(result), expirationTime)
        log.LogTrace("issued {token}", result)
        result

    member this.validateToken(token: string) =
        try
            Some(JwtSecurityTokenHandler().ValidateToken(token, this.tokenValidationParameters))
        with _ ->
            None

    member private this.signingCredentials =
        SigningCredentials(signingKey, SecurityAlgorithms.HmacSha256)

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

type CredentialsCheckError = | BadCredentials

type DBService(db: DbConnection) =
    member this.checkUsernameExists(username: string) =
        task {
            let! count =
                db.ExecuteScalarAsync<int>(
                    "SELECT COUNT(*) FROM users WHERE username = @username",
                    {| username = username |}
                )

            return count = 1
        }

    member this.checkUsernameAndPassword (username: string) (password: string) =
        task {
            let! count =
                db.ExecuteScalarAsync<int>(
                    "SELECT COUNT(*) FROM users WHERE username = @username AND password = @password",
                    {| username = username
                       password = password |}
                )

            return
                match count with
                | 1 -> Ok()
                | _ -> Error BadCredentials
        }

type ExtendedContextService
    (log: ILogger<ExtendedContextService>, ctxAcc: IHttpContextAccessor, jwt: JWTService, db: DBService) =
    let _user =
        Lazy<Task<(ClaimsPrincipal * SecurityToken * string) option>>(fun () ->
            let ctx = ctxAcc.HttpContext

            match
                (ctx.GetCookieValue "Authorization"
                 |> Option.map (fun token -> jwt.validateToken token)
                 |> Option.flatten
                 |> Option.map (fun (principal, token) ->
                     match principal.Claims |> Seq.tryFind (fun x -> x.Type = "username") with
                     | Some(username) -> Some(principal, token, username.Value)
                     | None -> None)
                 |> Option.flatten)
            with
            | Some(principal, token, username) ->
                task {
                    let! exists = db.checkUsernameExists username

                    return
                        match exists with
                        | true ->
                            log.LogTrace("{username} is logged in", username)
                            Some(principal, token, username)
                        | false ->
                            log.LogTrace("{username} token provided, but no such user", username)
                            None
                }
            | None ->
                log.LogTrace("no token provided")
                Task.FromResult(None))

    member val user = _user.Value

module RouteUtils =
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
    open Giraffe.ViewEngine

    let htmlPage (provider: IServiceProvider) (content: XmlNode list) =
        let env = provider.GetService<IWebHostEnvironment>()

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

    let index (provider: IServiceProvider) =
        [ div [] [ encodedText "Hello, World!" ] ] |> htmlPage provider

    let loginPage (provider: IServiceProvider) =
        [ form
              [ _id "login"; KeyValue("hx-post", "/login"); KeyValue("hx-swap", "none") ]
              [ label [ _for "username" ] [ encodedText "Username:" ]
                input [ _name "username"; _type "text" ]
                label [ _for "password" ] [ encodedText "Password:" ]
                input [ _name "password"; _type "password" ]
                div [] []
                div [] [ button [ _type "submit" ] [ encodedText "Login" ] ] ]
          div [ _id "loginErrors" ] [] ]
        |> htmlPage provider

    let loginFailure (message: string) =
        div [ _id "loginErrors"; _class "errors"; KeyValue("hx-swap-oob", "true") ] [ encodedText message ]
        |> htmlView

[<CLIMutable>]
type LoginRequest = { username: string; password: string }

let loginHandler: HttpHandler =
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
                    RouteUtils.htmxRedirect "/"
                | Error BadCredentials -> Views.loginFailure "Invalid credentials."

            return! response next ctx
        }

let webApp provider =
    choose
        [ choose
              [ GET
                >=> route "/"
                >=> RouteUtils.redirectIfNotAuthenticated
                >=> Views.index provider
                GET
                >=> route "/login"
                >=> RouteUtils.redirectIfAuthenticated
                >=> Views.loginPage provider
                POST >=> route "/login" >=> loginHandler ]
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

        app
            .UseCors(configureCors)
            .UseStaticFiles()
            .UseGiraffe(webApp app.ApplicationServices)

let configureServices (services: IServiceCollection) =
    services.AddCors().AddHttpContextAccessor().AddGiraffe() |> ignore

    services.AddSingleton<DBService>(fun _ ->
        let exeDir = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)
        let dbPath = Path.Combine(exeDir, "db.sqlite")

        use dbConn = new SqliteConnection $"Data Source={dbPath}"
        initDb dbConn |> Async.AwaitTask |> Async.RunSynchronously
        DBService dbConn)
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
