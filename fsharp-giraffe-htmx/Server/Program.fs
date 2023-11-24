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
open Microsoft.AspNetCore.Authentication.JwtBearer
open Microsoft.IdentityModel.Tokens
open System.Text
open System.IdentityModel.Tokens.Jwt
open System.Security.Claims

// TODO logging

// TODO move auth stuff to it's own file
type JWTService() =
    let issuer = "TODO issuer"
    let audience = "TODO audience"

    let signingKey =
        SymmetricSecurityKey(Encoding.UTF8.GetBytes("TODO signing key some more bits to get the key size up enough"))

    member this.tokenValidationParameters =
        TokenValidationParameters(
            ValidateIssuer = true,
            ValidateAudience = true,
            ValidateLifetime = true,
            ValidateIssuerSigningKey = true,
            ValidIssuer = issuer,
            ValidAudience = audience,
            IssuerSigningKey = signingKey
        )

    member this.createToken(username: string) =
        let credentials = SigningCredentials(signingKey, SecurityAlgorithms.HmacSha256)
        let expirationTime = DateTime.Now.AddMinutes 5

        let result =
            JwtSecurityToken(
                issuer,
                audience,
                [ Claim("username", username) ],
                DateTime.Now,
                expirationTime,
                credentials
            )

        (JwtSecurityTokenHandler().WriteToken(result), expirationTime)

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

let loginHandler (db: DBService) (jwt: JWTService) : HttpHandler =
    fun (next: HttpFunc) (ctx: HttpContext) ->
        task {
            let! request = ctx.BindFormAsync<LoginRequest>()

            let! result = db.checkUsernameAndPassword request.username request.password

            let response =
                match result with
                | Ok _ ->
                    let token, expirationTime = jwt.createToken request.username
                    ctx.Response.Cookies.Append("Authorization", token, CookieOptions(Expires = expirationTime))
                    Views.loginSuccess request.username
                | Error BadCredentials -> Views.loginFailure "invalid credentials"

            return! response next ctx
        }

let webApp db jwt =
    choose
        [ choose
              [ GET >=> route "/" >=> Views.index ()
                POST >=> route "/login" >=> loginHandler db jwt ]
          // TODO better 404 page
          setStatusCode 404 >=> text "Not Found" ]

let errorHandler (ex: Exception) (logger: ILogger) =
    logger.LogError(ex, "An unhandled exception has occurred while executing the request.")
    // TODO better 500 page
    clearResponse >=> setStatusCode 500 >=> text ex.Message

let configureCors (builder: CorsPolicyBuilder) =
    builder.AllowAnyOrigin().AllowAnyMethod().AllowAnyHeader() |> ignore

let configureApp db jwt =
    fun (app: IApplicationBuilder) ->
        let env = app.ApplicationServices.GetService<IWebHostEnvironment>()

        (match env.IsDevelopment() with
         | true -> app.UseDeveloperExceptionPage()
         | false -> app.UseGiraffeErrorHandler(errorHandler).UseHttpsRedirection())
            .UseCors(configureCors)
            .UseStaticFiles()
            .UseGiraffe(webApp db jwt)

let configureServices (jwt: JWTService) (services: IServiceCollection) =
    services.AddCors() |> ignore
    services.AddGiraffe() |> ignore

    services
        .AddAuthentication(JwtBearerDefaults.AuthenticationScheme)
        .AddJwtBearer(fun options ->
            options.TokenValidationParameters <- jwt.tokenValidationParameters

            ())
    |> ignore

let configureLogging (builder: ILoggingBuilder) =
    builder.AddConsole().AddDebug() |> ignore

[<EntryPoint>]
let main args =
    let contentRoot = Directory.GetCurrentDirectory()
    let webRoot = Path.Combine(contentRoot, "WebRoot")
    let exeDir = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)
    let dbPath = Path.Combine(exeDir, "db.sqlite")

    use dbConn = new SqliteConnection $"Data Source={dbPath}"
    initDb dbConn |> Async.AwaitTask |> Async.RunSynchronously
    let db = DBService dbConn

    let jwt = JWTService()

    Host
        .CreateDefaultBuilder(args)
        .ConfigureWebHostDefaults(fun webHostBuilder ->
            webHostBuilder
                .UseUrls("http://localhost:8000")
                .UseContentRoot(contentRoot)
                .UseWebRoot(webRoot)
                .Configure(Action<IApplicationBuilder>(configureApp db jwt))
                .ConfigureServices(configureServices jwt)
                .ConfigureLogging(configureLogging)
            |> ignore)
        .Build()
        .Run()

    0
