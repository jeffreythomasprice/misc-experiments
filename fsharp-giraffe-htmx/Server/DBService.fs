namespace Experiment.Server

open System.Data.Common
open Dapper
open System.Threading.Tasks

type CredentialsCheckError = | BadCredentials

[<CLIMutable>]
type User = { username: string; isAdmin: bool }

type DBService private (db: DbConnection) =
    static member create(db: DbConnection) =
        task {
            let! _ =
                db.ExecuteAsync
                    @"CREATE TABLE IF NOT EXISTS users (
                        username STRING NOT NULL UNIQUE PRIMARY KEY,
                        password STRING NOT NULL,
                        isAdmin BOOLEAN NOT NULL
                    )"

            let! _ =
                db.ExecuteAsync
                    @"INSERT OR IGNORE INTO users (username, password, isAdmin) VALUES
                        (""admin"", ""admin"", true),
                        (""user"", ""password"", false)
                    "

            return DBService db
        }

    member this.getUser(username: string) : Task<User option> =
        task {
            let! results =
                db.QueryAsync<User>(
                    "SELECT username, isAdmin FROM users WHERE username = @username",
                    {| username = username |}
                )

            return
                match List.ofSeq (results) with
                | [] -> None
                | [ result ] -> Some(result)
                | _ -> None
        }

    member this.checkUsernameAndPassword
        (username: string)
        (password: string)
        : Task<Result<unit, CredentialsCheckError>> =
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
