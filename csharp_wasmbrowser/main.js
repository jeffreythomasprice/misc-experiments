// Licensed to the .NET Foundation under one or more agreements.
// The .NET Foundation licenses this file to you under the MIT license.

import { dotnet } from "./dotnet.js"

const { setModuleImports, getAssemblyExports, getConfig } = await dotnet
	.withDiagnosticTracing(false)
	.withApplicationArgumentsFromQuery()
	.create();

const config = getConfig();
const exports = await getAssemblyExports(config.mainAssemblyName);

let animating = false;
let lastAnimationFrame = null;

function animate(time) {
	if (!animating) {
		return;
	}
	exports.Experiments.Dom.Utils.Animate(time);
	lastAnimationFrame = requestAnimationFrame(animate);
}

setModuleImports("main.js", {
	utils: {
		startAnimation: () => {
			if (animating) {
				return;
			}
			animating = true;
			lastAnimationFrame = requestAnimationFrame(animate);
		},
		stopAnimation: () => {
			animating = false;
			if (lastAnimationFrame) {
				cancelAnimationFrame(lastAnimationFrame);
				lastAnimationFrame = null;
			}
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

await dotnet.run();