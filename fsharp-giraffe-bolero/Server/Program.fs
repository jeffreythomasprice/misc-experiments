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

// TODO deduplicate
[<CLIMutable>]
type LoginRequest = { username: string; password: string }

// TODO deduplicate
type LoginResponse = { username: string }

let loginHandler (db: Db) : HttpHandler =
    fun (next: HttpFunc) (ctx: HttpContext) ->
        task {
            let! request = ctx.BindJsonAsync<LoginRequest>()

            let! result = db.CheckUsernameAndPassword request.username request.password

            let responseBody: LoginResponse = { username = request.username }

            return!
                match result with
                | CredentialsCheck.Success -> Successful.OK responseBody next ctx
                // | BadCredentials -> Successful.OK (Failure "invalid credentials") next ctx
                | BadCredentials -> RequestErrors.UNAUTHORIZED "schema" "realm" (Failure "invalid credentials") next ctx
        }

let webApp db =
    choose
        [ choose [ POST >=> route "/login" >=> loginHandler db ]
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
                .UseUrls("http://localhost:8001")
                .UseContentRoot(contentRoot)
                .UseWebRoot(webRoot)
                .Configure(Action<IApplicationBuilder>(configureApp (Db db)))
                .ConfigureServices(configureServices)
                .ConfigureLogging(configureLogging)
            |> ignore)
        .Build()
        .Run()

    0
