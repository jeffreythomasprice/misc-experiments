import js_bindings

type RunnerState

@external(javascript, "./runner_ffi.mjs", "new_runner_state")
fn new_runner_state(initial_state: state) -> RunnerState

@external(javascript, "./runner_ffi.mjs", "get_runner_state")
fn get_runner_state(state: RunnerState) -> state

@external(javascript, "./runner_ffi.mjs", "update_runner_state")
fn update_runner_state(state: RunnerState, new_state: state) -> Nil

pub type Event {
  Resize(width: Int, height: Int)
  Animate(time: Float)
}

pub fn run(init: fn() -> state, update: fn(state, Event) -> state) -> Nil {
  let state = new_runner_state(init())

  let emit_event = fn(event: Event) {
    update_runner_state(state, update(get_runner_state(state), event))
  }

  js_bindings.get_window()
  |> js_bindings.window_to_event_receiver
  |> js_bindings.add_event_listener("resize", fn(_) {
    Resize(
      js_bindings.get_window() |> js_bindings.window_inner_width,
      js_bindings.get_window() |> js_bindings.window_inner_height,
    )
    |> emit_event
  })

  js_bindings.request_animation_frame(fn(time) {
    animate_impl(state, time, emit_event)
  })
}

fn animate_impl(
  state: RunnerState,
  time: Float,
  emit_event: fn(Event) -> Nil,
) -> Nil {
  emit_event(Animate(time))
  js_bindings.request_animation_frame(fn(time) {
    animate_impl(state, time, emit_event)
  })
}
