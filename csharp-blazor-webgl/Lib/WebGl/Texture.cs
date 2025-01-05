using BlazorExperiments.Lib.Math;
using System.Drawing;

namespace BlazorExperiments.Lib.WebGl;

public class Texture : IDisposable
{
    private readonly WebGL2RenderingContext gl;
    private readonly WebGL2RenderingContext.Texture texture;
    private bool disposedValue;

    public Texture(WebGL2RenderingContext gl, Size size, Span<ColorRGBA<byte>> data)
    {
        this.gl = gl;

        if (data.Length != size.Width * size.Height)
        {
            throw new ArgumentOutOfRangeException("data doesn't match expected size");
        }
        var bytes = new byte[data.Length * 4];
        for (int bytesIndex = 0, dataIndex = 0; dataIndex < data.Length; dataIndex++, bytesIndex += 4)
        {
            bytes[bytesIndex + 0] = data[dataIndex].Red;
            bytes[bytesIndex + 1] = data[dataIndex].Green;
            bytes[bytesIndex + 2] = data[dataIndex].Blue;
            bytes[bytesIndex + 3] = data[dataIndex].Alpha;
        }

        texture = gl.CreateTexture();
        gl.BindTexture(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, texture);
        gl.TexImage2D(
            WebGL2RenderingContext.TextureTarget.TEXTURE_2D,
            0,
            WebGL2RenderingContext.TextureInternalFormat.RGBA,
            size.Width,
            size.Height,
            0,
            WebGL2RenderingContext.TextureFormat.RGBA,
            WebGL2RenderingContext.TextureDataType.UNSIGNED_BYTE,
            bytes
        );

        bool isPowerOf2(int x) => (x & (x - 1)) == 0;
        if (isPowerOf2(size.Width) && isPowerOf2(size.Height))
        {
            gl.GenerateMipmap(WebGL2RenderingContext.TextureTarget.TEXTURE_2D);
            gl.TexParameter(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, WebGL2RenderingContext.TextureParameter.TEXTURE_MAG_FILTER, WebGL2RenderingContext.TextureMagFilter.LINEAR);
            gl.TexParameter(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, WebGL2RenderingContext.TextureParameter.TEXTURE_MIN_FILTER, WebGL2RenderingContext.TextureMinFilter.NEAREST_MIPMAP_LINEAR);
            gl.TexParameter(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, WebGL2RenderingContext.TextureParameter.TEXTURE_WRAP_S, WebGL2RenderingContext.TextureWrap.REPEAT);
            gl.TexParameter(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, WebGL2RenderingContext.TextureParameter.TEXTURE_WRAP_S, WebGL2RenderingContext.TextureWrap.REPEAT);
        }
        else
        {
            gl.TexParameter(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, WebGL2RenderingContext.TextureParameter.TEXTURE_MAG_FILTER, WebGL2RenderingContext.TextureMagFilter.LINEAR);
            gl.TexParameter(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, WebGL2RenderingContext.TextureParameter.TEXTURE_MIN_FILTER, WebGL2RenderingContext.TextureMinFilter.NEAREST);
            gl.TexParameter(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, WebGL2RenderingContext.TextureParameter.TEXTURE_WRAP_S, WebGL2RenderingContext.TextureWrap.CLAMP_TO_EDGE);
            gl.TexParameter(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, WebGL2RenderingContext.TextureParameter.TEXTURE_WRAP_S, WebGL2RenderingContext.TextureWrap.CLAMP_TO_EDGE);
        }

        gl.BindTexture(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, null);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (!disposedValue)
        {
            gl.DeleteTexture(texture);

            disposedValue = true;
        }
    }

    ~Texture()
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

    public void Bind()
    {
        gl.BindTexture(WebGL2RenderingContext.TextureTarget.TEXTURE_2D, texture);
    }
}
