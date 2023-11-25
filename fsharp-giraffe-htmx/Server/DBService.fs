namespace Experiment.Server

open System.Data.Common
open Dapper

type CredentialsCheckError = | BadCredentials

type DBService private (db: DbConnection) =
    static member create(db: DbConnection) =
        task {
            let! _ =
                db.ExecuteAsync
                    @"CREATE TABLE IF NOT EXISTS users (
                        username STRING NOT NULL UNIQUE PRIMARY KEY,
                        password STRING NOT NULL
                    )"

            let! _ = db.ExecuteAsync @"INSERT OR IGNORE INTO users (username, password) VALUES (""admin"", ""admin"")"

            return DBService db
        }

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
