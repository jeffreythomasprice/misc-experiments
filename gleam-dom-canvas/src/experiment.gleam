import dom_utils
import gleam/int
import js_bindings
import runner

type State {
  State(
    canvas: js_bindings.Element,
    context: js_bindings.CanvasRenderingContext2D,
  )
}

pub fn main() -> Nil {
  runner.run(
    fn() {
      let document = js_bindings.get_document()

      let body = document |> js_bindings.document_body()
      dom_utils.element_remove_all_children(body)

      let canvas = document |> js_bindings.document_create_element("canvas")
      body |> js_bindings.element_append_child(canvas)
      canvas
      |> js_bindings.element_style
      |> js_bindings.style_set("position", "absolute")
      canvas
      |> js_bindings.element_style
      |> js_bindings.style_set("width", "100%")
      canvas
      |> js_bindings.element_style
      |> js_bindings.style_set("height", "100%")
      canvas
      |> js_bindings.element_style
      |> js_bindings.style_set("left", "0")
      canvas
      |> js_bindings.element_style
      |> js_bindings.style_set("top", "0")

      let context = js_bindings.canvas_get_context_2d(canvas)

      State(canvas, context)
    },
    fn(state, event) {
      let State(canvas:, context:) = state

      case event {
        runner.Resize(width:, height:) -> {
          canvas |> js_bindings.element_set_width(width)
          canvas |> js_bindings.element_set_height(height)
          Nil
        }

        runner.Animate(time: _) -> {
          let width = canvas |> js_bindings.element_width |> int.to_float
          let height = canvas |> js_bindings.element_height |> int.to_float
          context |> js_bindings.context_2d_fill_style("red")
          context |> js_bindings.context_2d_fill_rect(0.0, 0.0, width, height)
          Nil
        }
      }

      state
    },
  )
}
