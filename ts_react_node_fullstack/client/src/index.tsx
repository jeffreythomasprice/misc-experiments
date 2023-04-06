import { createRoot } from "react-dom/client";
import { App } from "./components/App";

const container = document.getElementById("app");
if (!container) {
	throw new Error("can't find root element");
}
const root = createRoot(container)
root.render(<App />);
