using BlazorExperiments.Client.Demo;
using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace BlazorExperiments.Client.Pages;

public partial class Home : ComponentBase
{
    [Inject]
    public required IJSRuntime JS { get; set; }

    private ElementReference Canvas;

    protected override async Task OnAfterRenderAsync(bool firstRender)
    {
        if (firstRender)
        {
            await Lib.Dom.Canvas.Create(JS, Canvas, DemoState.Create());
        }
    }
}
