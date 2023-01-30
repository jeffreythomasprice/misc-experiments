// Licensed to the .NET Foundation under one or more agreements.
// The .NET Foundation licenses this file to you under the MIT license.

import { dotnet } from "./dotnet.js"

const { setModuleImports, getAssemblyExports, getConfig } = await dotnet
	.withDiagnosticTracing(false)
	.withApplicationArgumentsFromQuery()
	.create();

setModuleImports("main.js", {
	utils: {
		createObject: () => {
			return {};
		},
	},
	document: {
		createElement: (tagName) => {
			return document.createElement(tagName);
		}
	},
	body: {
		replaceChildren: (children) => {
			return document.body.replaceChildren(...children);
		}
	},
	canvas: {
		getContext: (canvas, contextType, contextAttributes) => {
			contextAttributes = JSON.parse(contextAttributes);
			if (contextAttributes) {
				return canvas.getContext(contextType, contextAttributes);
			} else {
				return canvas.getContext(contextType);
			}
		}
	}
});

// const config = getConfig();
// const exports = await getAssemblyExports(config.mainAssemblyName);
// exports.Foobar.Main();

await dotnet.run();