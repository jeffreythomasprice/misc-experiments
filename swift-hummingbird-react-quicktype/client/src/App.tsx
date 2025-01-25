import { useEffect, useState } from "react";
import { Convert, type ExampleRequest, type ExampleResponse } from "./generated/generated";

export function App() {
	let [count, setCount] = useState(0);

	useEffect(
		() => {
			(async () => {
				const response = await getCount();
				setCount(response.count);
			})();
		},
		[]
	);

	const click = () => {
		(async () => {
			const response = await postCount({ incrementBy: 1 });
			setCount(response.count);
		})();
	};

	return <div>
		<div>Count: {count}</div>
		<button onClick={() => click()}>Click Me</button>
	</div>;
}

async function getCount(): Promise<ExampleResponse> {
	const response = await fetch(
		"/count",
		{
			headers: {
				"Accept": "application/json",
			}
		}
	);
	return Convert.toExampleResponse(await response.text());
}

async function postCount(request: ExampleRequest): Promise<ExampleResponse> {
	const response = await fetch(
		"/count",
		{
			method: "POST",
			headers: {
				"Accept": "application/json",
				"Content-Type": "application/json",
			},
			body: Convert.exampleRequestToJson(request),
		}
	);
	return Convert.toExampleResponse(await response.text());
}