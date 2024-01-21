import { FastifyRequest, RouteHandlerMethod } from "fastify";
import React from "react";
import { renderToString } from "react-dom/server";

export function htmlPageFunc(f: (request: FastifyRequest) => React.JSX.Element): RouteHandlerMethod {
	return htmlFunc((request) =>
		"<!DOCTYPE html>" +
		renderToString(<html>
			<head>
				<meta charSet="utf-8" />
				<script src="https://unpkg.com/htmx.org@1.9.10"></script>
				<script>
					htmx.logAll();
				</script>
			</head>
			<body>
				{f(request)}
			</body>
		</html>));
}

export function htmlSnippetFunc(f: (request: FastifyRequest) => React.JSX.Element): RouteHandlerMethod {
	return htmlFunc((request) => renderToString(f(request)));
}

function htmlFunc(f: (request: FastifyRequest) => string): RouteHandlerMethod {
	return (request, reply) => {
		const { log } = request;
		try {
			const response = f(request);
			reply
				.code(200)
				.header("Content-Type", "text/html")
				.send(response);
		} catch (err) {
			log.error({ err }, "error rendering page");
			// TODO better error handling
			reply
				.code(500)
				.send("Internal server error");
		}
	};
}
