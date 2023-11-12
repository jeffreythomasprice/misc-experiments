module Client.Main

open System
open System.Net.Http
open System.Net.Http.Json
open Microsoft.AspNetCore.Components
open Elmish
open Bolero
open Bolero.Html

type Counter() =
    inherit ElmishComponent<int, int>()

    override this.View model dispatch =
        concat {
            div { $"count: {model}" }

            button {
                on.click (fun _ -> dispatch (model + 1))
                "Click Me"
            }
        }

type Model = { counter: int }

let initModel = { counter = 0 }

type Message =
    // TODO no
    | TestApi
    | TestApiSuccess
    | Error of exn
    | SetCounter of int

let update (http: HttpClient) message model =
    match message with
    // TODO no
    | TestApi ->
        let testApi () =
            task {
                let! response = http.GetAsync("/test")
                printfn "status code = %A" response.StatusCode
                let! responseBody = response.Content.ReadAsStringAsync()
                printfn "response body = %s" responseBody
            }

        let cmd = Cmd.OfTask.either testApi () (fun _ -> TestApiSuccess) Error
        model, cmd
    | TestApiSuccess ->
        printfn "test api success"
        model, Cmd.none
    | Error e ->
        printfn "error %A" e
        model, Cmd.none
    | SetCounter newCount ->
        printfn "update counter to %A" newCount
        { model with counter = newCount }, Cmd.none

let view model dispatch =
    div {
        ecomp<Counter, _, _> model.counter (fun newCount -> dispatch (SetCounter newCount)) { attr.empty () }
        div { "Hello, World!" }
    }

type MyApp() =
    inherit ProgramComponent<Model, Message>()

    [<Inject>]
    member val HttpClient = Unchecked.defaultof<HttpClient> with get, set

    override this.Program =
        let update = update this.HttpClient

        Program.mkProgram (fun _ -> initModel, Cmd.ofMsg TestApi) update view
