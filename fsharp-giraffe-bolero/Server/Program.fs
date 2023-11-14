module Server

open System
open System.IO
open Microsoft.AspNetCore.Builder
open Microsoft.AspNetCore.Cors.Infrastructure
open Microsoft.AspNetCore.Hosting
open Microsoft.Extensions.Hosting
open Microsoft.Extensions.Logging
open Microsoft.Extensions.DependencyInjection
open Giraffe
open System.Reflection
open System.Data.Common
open Dapper
open Microsoft.Data.Sqlite
open Microsoft.AspNetCore.Http
open Shared
open Microsoft.AspNetCore.Authentication.JwtBearer
open Microsoft.IdentityModel.Tokens
open System.Text
open System.IdentityModel.Tokens.Jwt

// TODO logging

// TODO move auth stuff to it's own file
type JWTService() =
    let issuer = "TODO issuer"
    let audience = "TODO audience"
    let signingKey = SymmetricSecurityKey(Encoding.UTF8.GetBytes("TODO signing key"))

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

    member this.createToken() =
        let credentials = SigningCredentials(signingKey, SecurityAlgorithms.HmacSha256)

        let result =
            JwtSecurityToken(issuer, audience, null, DateTime.Now, DateTime.Now.AddMinutes 60, credentials)

        JwtSecurityTokenHandler().WriteToken(result)

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

// TODO move auth stuff to it's own file
let defaultUnauthorized =
    let response: GenericFailureResponse = { message = "invalid credentials" }
    RequestErrors.UNAUTHORIZED "Bearer" "TODO realm" response

// TODO move auth stuff to it's own file
let requiresAuthentication = requiresAuthentication defaultUnauthorized

// TODO move db stuff to it's own file
type CredentialsCheck =
    | Success
    | BadCredentials

// TODO move db stuff to it's own file
type DBService(db: DbConnection) =
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

let loginHandler (db: DBService) (jwt: JWTService) : HttpHandler =
    fun (next: HttpFunc) (ctx: HttpContext) ->
        task {
            try
                let! request = ctx.BindJsonAsync<Login.Request>()

                let! result = db.CheckUsernameAndPassword request.username request.password

                let responseBody: Login.Response =
                    { username = request.username
                      token = jwt.createToken () }

                return!
                    match result with
                    | Success -> Successful.OK responseBody next ctx
                    | BadCredentials -> defaultUnauthorized next ctx
            with _ ->
                return! defaultUnauthorized next ctx
        }

// TODO no
let testHandler (next: HttpFunc) (ctx: HttpContext) = Successful.OK "test" next ctx

let webApp db jwt =
    choose
        [ choose
              [ route "/login" >=> POST >=> loginHandler db jwt
                route "/test" >=> GET >=> requiresAuthentication >=> testHandler ]
          // TODO better 404 page
          setStatusCode 404 >=> text "Not Found" ]

let errorHandler (ex: Exception) (logger: ILogger) =
    logger.LogError(ex, "An unhandled exception has occurred while executing the request.")
    // TODO better 500 page
    clearResponse >=> setStatusCode 500 >=> text ex.Message

let configureCors (builder: CorsPolicyBuilder) =
    builder.AllowAnyOrigin().AllowAnyMethod().AllowAnyHeader() |> ignore

let configureApp db jwt (app: IApplicationBuilder) =
    let env = app.ApplicationServices.GetService<IWebHostEnvironment>()

    app.UseAuthentication() |> ignore

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
    let webRoot = Path.Combine(contentRoot, "wwwroot")
    let exeDir = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)
    let dbPath = Path.Combine(exeDir, "db.sqlite")

    use db = new SqliteConnection $"Data Source={dbPath}"
    initDb db |> Async.AwaitTask |> Async.RunSynchronously
    let db = DBService db

    let jwt = JWTService()

    Host
        .CreateDefaultBuilder(args)
        .ConfigureWebHostDefaults(fun webHostBuilder ->
            webHostBuilder
                .UseUrls("http://localhost:8001")
                .UseContentRoot(contentRoot)
                .UseWebRoot(webRoot)
                .Configure(configureApp db jwt)
                .ConfigureServices(configureServices jwt)
                .ConfigureLogging(configureLogging)
            |> ignore)
        .Build()
        .Run()

    0
