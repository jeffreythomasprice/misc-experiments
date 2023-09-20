import { Component, createSignal } from "solid-js";
import { LoginResponse } from "./models";
import { Service } from "./Service";

interface Props {
	service: Service;
	callback: (response: LoginResponse & { name: string }) => void;
}

export const Login: Component<Props> = ({ service, callback }) => {
	const [name, setName] = createSignal("");

	const onSubmit = async () => {
		try {
			const _name = name();
			const response = await service.login(_name);
			callback({ ...response, name: _name });
		} catch (err) {
			console.error("error logging in", err);
		}
	};

	return <>
		<form onsubmit={(e) => {
			e.preventDefault();
			onSubmit();
		}}>
			<input
				type="text"
				placeholder="Name"
				autofocus
				oninput={(x) => {
					setName(x.target.value);
				}}
			></input>
			<button type="submit">Log In</button>
		</form>
	</>;
};
