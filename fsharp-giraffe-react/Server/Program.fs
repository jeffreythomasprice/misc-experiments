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
open Microsoft.Data.Sqlite
open Dapper
open System.Collections.Generic
open System.Threading.Tasks

[<CLIMutable>]
type User = { username: string }

type UserPasswordCheck =
    | User of User
    | BadCredentials

let checkUsersPassword (db: SqliteConnection) (username: string) (password: string) : Task<UserPasswordCheck> =
    task {
        let! results =
            db.QueryAsync<User>(
                "select username from users where username = @username and password = @password",
                {| username = username
                   password = password |}
            )

        return
            match results |> List.ofSeq with
            | [ head ] -> User head
            | []
            | _ :: _ -> BadCredentials
    }

[<CLIMutable>]
type LoginRequest = { username: string; password: string }

let login (db: SqliteConnection) =
    fun next (ctx: HttpContext) ->
        task {
            let! request = ctx.BindJsonAsync<LoginRequest>()
            printfn "TODO login request = %A" request

            let! user = checkUsersPassword db request.username request.password

            return!
                (match user with
                 | User(user) ->
                     printfn "TODO login successful, user = %A" user
                     Successful.NO_CONTENT
                 | BadCredentials ->
                     printfn "TODO login failed, bad credentials"
                     RequestErrors.UNAUTHORIZED "TODO scheme" "TODO realm" "TODO message")
                    next
                    ctx
        }

// TODO make a typed websocket thing with json parsing? channels?
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

let webApp (db: SqliteConnection) =
    choose
        [ choose
              [ POST >=> route "/login" >=> (login db)
                GET >=> route "/ws" >=> websocketHandler ]
          setStatusCode 404 >=> text "Not Found" ]

let errorHandler (ex: Exception) (logger: ILogger) =
    logger.LogError(ex, "An unhandled exception has occurred while executing the request.")
    clearResponse >=> setStatusCode 500 >=> text ex.Message

let configureCors (builder: CorsPolicyBuilder) =
    builder.AllowAnyOrigin().AllowAnyMethod().AllowAnyHeader() |> ignore

let dbInit (db: SqliteConnection) =
    task {
        let! _ =
            db.ExecuteAsync
                @"create table if not exists users (
                username varchar(256) not null primary key unique,
                password varcahr(256) not null
            )"

        let! adminUserCount = db.QuerySingleAsync<int>("select count(*) from users where username = 'admin'")

        if adminUserCount = 0 then
            let! _ = db.ExecuteAsync "insert into users (username, password) values ('admin', 'admin')"
            ()

        ()
    }
    |> Async.AwaitTask

let configureApp (app: IApplicationBuilder) =
    let env = app.ApplicationServices.GetService<IWebHostEnvironment>()

    use db = new SqliteConnection "Data Source=db.sqlite"
    dbInit db |> Async.RunSynchronously

    (match env.IsDevelopment() with
     | true -> app.UseDeveloperExceptionPage()
     | false -> app.UseGiraffeErrorHandler(errorHandler).UseHttpsRedirection())
        .UseCors(configureCors)
        .UseStaticFiles()
        .UseWebSockets()
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
