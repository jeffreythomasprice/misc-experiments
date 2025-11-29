export function new_runner_state(initial_state) {
	return {
		state: initial_state,
	};
}

export function get_runner_state(state) {
	return state.state;
}

export function update_runner_state(state, new_state) {
	state.state = new_state;
}