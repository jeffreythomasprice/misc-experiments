// Licensed to the .NET Foundation under one or more agreements.
// The .NET Foundation licenses this file to you under the MIT license.

import { dotnet } from "./dotnet.js"

const { setModuleImports, getAssemblyExports, getConfig } = await dotnet
    .withDiagnosticTracing(false)
    .withApplicationArgumentsFromQuery()
    .create();

setModuleImports("main.js", {
    document: {
        createElement: (tagName) => {
            return document.createElement(tagName);
        }
    }
});

const config = getConfig();
const exports = await getAssemblyExports(config.mainAssemblyName);
exports.Foobar.Main();

await dotnet.run();