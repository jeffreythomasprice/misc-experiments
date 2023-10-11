using System.Net.Mime;
using CSharpHtmx.Components;
using Microsoft.AspNetCore.Components.Web;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddScoped<HtmlRenderer>();
builder.Services.AddScoped<BlazorRenderer>();

var app = builder.Build();

app.MapGet("/", async (BlazorRenderer renderer) =>
{
	return Results.Content(
		await renderer.RenderComponent<IndexPage>(),
		MediaTypeNames.Text.Html
	);
});

int clicks = 0;
app.MapPost("/click", async (BlazorRenderer renderer) =>
{
	clicks++;
	return Results.Content(
		await renderer.RenderComponent<ClickResults>(new()
		{
			{nameof(ClickResults.Count), clicks }
		}),
		MediaTypeNames.Text.Html
	);
});

app.Run();
