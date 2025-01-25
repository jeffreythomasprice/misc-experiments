import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";

const rootDomElement = document.getElementById("root");
if (!rootDomElement) {
	throw new Error("can't find root element");
}
const rootReactNode = ReactDOM.createRoot(rootDomElement);
rootReactNode.render(<React.StrictMode>
	<App />
</React.StrictMode>);
