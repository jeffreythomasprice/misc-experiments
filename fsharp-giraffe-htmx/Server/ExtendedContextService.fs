namespace Experiment.Server

open Microsoft.AspNetCore.Http
open Microsoft.Extensions.Logging
open System.Threading.Tasks
open System.Security.Claims
open Microsoft.IdentityModel.Tokens
open Giraffe

type ExtendedContextService
    (log: ILogger<ExtendedContextService>, ctxAcc: IHttpContextAccessor, jwt: JWTService, db: DBService) =
    let _user =
        Lazy<Task<(ClaimsPrincipal * SecurityToken * User) option>>(fun () ->
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
                    let! user = db.getUser username

                    return
                        match user with
                        | Some(user) ->
                            log.LogTrace("{user} is logged in", user)
                            Some(principal, token, user)
                        | None ->
                            log.LogTrace("{username} token provided, but no such user", username)
                            None
                }
            | None ->
                log.LogTrace("no token provided")
                Task.FromResult(None))

    member val user = _user.Value
