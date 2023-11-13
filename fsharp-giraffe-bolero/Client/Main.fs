module Client.Main

open System
open System.Net.Http
open System.Net.Http.Json
open Microsoft.AspNetCore.Components
open Elmish
open Bolero
open Bolero.Html
open Shared

// TODO authenticated route guards, client side

module LoginForm =
    type Model = { username: string; password: string }

    type Message =
        | SetUsername of string
        | SetPassword of string
        | Submit
        | SubmitSuccess of Login.Response
        | SubmitError of exn

    let init () : Model = { username = ""; password = "" }

    let update (http: HttpClient) (message: Message) (model: Model) =
        match message with
        | SetUsername username -> { model with username = username }, Cmd.none
        | SetPassword password -> { model with password = password }, Cmd.none
        | Submit ->
            printfn "TODO login, model = %A" model

            let submit (request: Login.Request) =
                async {
                    let! response = http.PostAsJsonAsync("/login", request) |> Async.AwaitTask

                    return ()
                }

            let cmd =
                Cmd.OfAsync.either
                    (fun _ ->
                        async {
                            let request: Login.Request =
                                { username = model.username
                                  password = model.password }

                            let! response = http.PostAsJsonAsync("/login", request) |> Async.AwaitTask
                            response.EnsureSuccessStatusCode() |> ignore

                            let! responseBody = response.Content.ReadFromJsonAsync<Login.Response>() |> Async.AwaitTask

                            return responseBody
                        })
                    ()
                    (fun result -> SubmitSuccess result)
                    (fun e -> SubmitError e)

            model, cmd
        | SubmitSuccess response ->
            printfn "TODO login success, %A" response
            model, Cmd.none
        | SubmitError e ->
            printfn "TODO login failure, %A" e
            model, Cmd.none

    let view (model: Model) (dispatch: Message -> unit) =
        form {
            attr.id "login"

            on.submit (fun _ -> dispatch Submit)

            label {
                attr.``for`` "username"
                "Username:"
            }

            input {
                attr.name "username"
                attr.``type`` "text"
                on.change (fun e -> dispatch (SetUsername(unbox e.Value)))
            }

            label {
                attr.``for`` "password"
                "Password:"
            }

            input {
                attr.name "password"
                attr.``type`` "password"
                on.change (fun e -> dispatch (SetPassword(unbox e.Value)))
            }

            div { attr.empty () }

            div {
                button {
                    attr.``type`` "submit"
                    "Login"
                }
            }
        }

type Model = { loginForm: LoginForm.Model }

let initModel = { loginForm = LoginForm.init () }

type Message = LoginForm of LoginForm.Message

let update (http: HttpClient) message model =
    match message with
    | LoginForm message ->
        let updated, cmd = LoginForm.update http message model.loginForm
        { model with loginForm = updated }, (Cmd.map (fun msg -> LoginForm msg) cmd)

let view model dispatch =
    div { LoginForm.view model.loginForm (fun message -> dispatch (LoginForm message)) }

type MyApp() =
    inherit ProgramComponent<Model, Message>()

    [<Inject>]
    member val HttpClient = Unchecked.defaultof<HttpClient> with get, set

    override this.Program =
        let update = update this.HttpClient

        Program.mkProgram (fun _ -> initModel, Cmd.none) update view
