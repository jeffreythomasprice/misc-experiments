module Client.Main

open System.Net.Http
open System.Net.Http.Json
open Microsoft.AspNetCore.Components
open Elmish
open Bolero
open Bolero.Html
open Shared
open System.Net

type Page =
    | [<EndPoint "/login">] Login
    | [<EndPoint "/">] LoggedIn

module LoginForm =
    open System.Net.Http.Headers

    type Model =
        { username: string
          password: string
          errorMessage: string option }

    type Message =
        | SetUsername of string
        | SetPassword of string
        | Submit
        | SubmitSuccess of Login.Response
        | SubmitError of GenericFailureResponse

    let init () : Model =
        { username = ""
          password = ""
          errorMessage = None }

    let update (http: HttpClient) (message: Message) (model: Model) =
        match message with
        | SetUsername username -> { model with username = username }, Cmd.none, None
        | SetPassword password -> { model with password = password }, Cmd.none, None
        | Submit ->
            let cmd =
                Cmd.OfAsync.either
                    (fun _ ->
                        async {
                            let request: Login.Request =
                                { username = model.username
                                  password = model.password }

                            let! response = http.PostAsJsonAsync("/login", request) |> Async.AwaitTask

                            return!
                                async {
                                    match response.StatusCode with
                                    | HttpStatusCode.OK ->
                                        let! responseBody =
                                            response.Content.ReadFromJsonAsync<Login.Response>() |> Async.AwaitTask

                                        return SubmitSuccess responseBody
                                    | _ ->
                                        let! responseBody =
                                            response.Content.ReadFromJsonAsync<GenericFailureResponse>()
                                            |> Async.AwaitTask

                                        return SubmitError responseBody
                                }
                        })
                    ()
                    (fun result -> result)
                    (fun e -> SubmitError { message = e.ToString() })

            model, cmd, None
        | SubmitSuccess response ->
            // TODO save token in local store?
            http.DefaultRequestHeaders.Authorization <- AuthenticationHeaderValue("Bearer", response.token)

            // TODO no
            task {
                let! s = http.GetStringAsync("/test") |> Async.AwaitTask
                printfn "test api = %s" s
            }
            |> ignore

            model, Cmd.none, Some(LoggedIn)
        | SubmitError e ->
            { model with
                errorMessage = Some(e.message) },
            Cmd.none,
            None

    let view (model: Model) (dispatch: Message -> unit) =
        concat {
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

            cond model.errorMessage
            <| function
                | Some s ->
                    div {
                        attr.``class`` "error"
                        s
                    }
                | None -> empty ()
        }

type Model =
    { page: Page
      loginForm: LoginForm.Model }

let initModel =
    { page = Login
      loginForm = LoginForm.init () }

type Message =
    | SetPage of Page
    | LoginForm of LoginForm.Message

let update (http: HttpClient) message model =
    match message with
    | SetPage page ->
        // TODO check page, only login is valid if unauthenticated
        { model with page = page }, Cmd.none
    | LoginForm message ->
        let updated, cmd, page = LoginForm.update http message model.loginForm

        { model with
            loginForm = updated
            page = page |> Option.defaultValue model.page },
        (Cmd.map (fun msg -> LoginForm msg) cmd)

let view model dispatch =
    div {
        cond model.page
        <| function
            | Login -> LoginForm.view model.loginForm (fun message -> dispatch (LoginForm message))
            | LoggedIn -> div { "TODO logged in page" }
    }

type App() =
    inherit ProgramComponent<Model, Message>()

    [<Inject>]
    member val HttpClient = Unchecked.defaultof<HttpClient> with get, set

    override this.Program =
        let update = update this.HttpClient

        let router =
            Router.infer SetPage (fun model -> model.page) |> Router.withNotFound Login

        Program.mkProgram (fun _ -> initModel, Cmd.none) update view
        |> Program.withRouter router
