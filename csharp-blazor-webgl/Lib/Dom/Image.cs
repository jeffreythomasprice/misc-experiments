using Microsoft.JSInterop;
using System.Drawing;

namespace BlazorExperiments.Lib.Dom;

public class Image : IDisposable
{
    private readonly IJSInProcessObjectReference objRef;
    private bool disposedValue;

    public static async Task<Image> FromUrl(IJSRuntime js, string url)
    {
        var module = await js.InvokeAsync<IJSInProcessObjectReference>("import", "./_content/Lib/Image.js");
        var rootObj = await module.InvokeAsync<IJSInProcessObjectReference>("init");
        return new(await rootObj.InvokeAsync<IJSInProcessObjectReference>("loadImageUrl", url));
    }

    private Image(IJSInProcessObjectReference objRef)
    {
        this.objRef = objRef;
    }

    public Size Size => new(objRef.Invoke<int>("getWidth"), objRef.Invoke<int>("getHeight"));

    protected virtual void Dispose(bool disposing)
    {
        if (!disposedValue)
        {
            objRef.Dispose();
            disposedValue = true;
        }
    }

    ~Image()
    {
        // Do not change this code. Put cleanup code in 'Dispose(bool disposing)' method
        Dispose(disposing: false);
    }

    public void Dispose()
    {
        // Do not change this code. Put cleanup code in 'Dispose(bool disposing)' method
        Dispose(disposing: true);
        GC.SuppressFinalize(this);
    }
}
