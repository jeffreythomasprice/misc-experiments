namespace Client

open Microsoft.AspNetCore.Components.WebAssembly.Hosting
open Microsoft.Extensions.DependencyInjection
open System
open System.Net.Http

module Program =

    [<EntryPoint>]
    let Main args =
        let builder = WebAssemblyHostBuilder.CreateDefault(args)
        builder.RootComponents.Add<Main.MyApp>("#main")

        builder.Services.AddScoped<HttpClient>(fun _ ->
            // TODO change builder.HostEnvironment.BaseAddress to be the server addr and use that
            new HttpClient(BaseAddress = Uri "http://localhost:8001"))
        |> ignore

        builder.Build().RunAsync() |> ignore
        0
