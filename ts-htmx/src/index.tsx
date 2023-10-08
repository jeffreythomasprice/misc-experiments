import Router from "@koa/router";
import Koa from "koa";
import React from "react";
import ReactDom from "react-dom/server";

const app = new Koa();
const router = new Router();

router.get("/", htmlHandler(() => {
	return <html>
		<head lang="en">
			<meta charSet="utf-8" />
			<script src="https://unpkg.com/htmx.org@1.9.6"></script>
			<script>
				htmx.logAll();
			</script>
		</head>
		<body>
			<h1>Hello, World!</h1>
			<button
				hx-post="/click"
				hx-swap="innerHTML"
				hx-target="#clickResults"
			>Click Me</button>
			<div id="clickResults"></div>
		</body>
	</html>;
}));

let clicks = 0;
router.post("/click", htmlHandler(() => {
	clicks++;
	return <div>Clicks: {clicks}</div>
}));

app.
	use(router.routes()).
	use(router.allowedMethods());
app.listen(8000, "127.0.0.1");

function htmlHandler<Context extends Koa.ParameterizedContext>(f: (ctx: Context) => React.ReactElement) {
	return (ctx: Context) => {
		const result = ReactDom.renderToString(f(ctx));
		if (/^\s+<html>/i.exec(result)) {
			ctx.body = `<!DOCTYPE html>${result}`;
		} else {
			ctx.body = result;
		}
	};
}
