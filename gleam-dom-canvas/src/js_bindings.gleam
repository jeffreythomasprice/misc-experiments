pub type Document

pub type Window

pub type Element

pub type HTMLCollection

pub type Style

pub type CanvasRenderingContext2D

pub type EventReceiver

pub type Event

@external(javascript, "./js_bindings_ffi.mjs", "get_document")
pub fn get_document() -> Document

@external(javascript, "./js_bindings_ffi.mjs", "get_window")
pub fn get_window() -> Window

@external(javascript, "./js_bindings_ffi.mjs", "window_inner_width")
pub fn window_inner_width(window: Window) -> Int

@external(javascript, "./js_bindings_ffi.mjs", "window_inner_height")
pub fn window_inner_height(window: Window) -> Int

@external(javascript, "./js_bindings_ffi.mjs", "document_body")
pub fn document_body(document: Document) -> Element

@external(javascript, "./js_bindings_ffi.mjs", "document_create_element")
pub fn document_create_element(document: Document, tag: String) -> Element

@external(javascript, "./js_bindings_ffi.mjs", "element_children")
pub fn element_children(element: Element) -> HTMLCollection

@external(javascript, "./js_bindings_ffi.mjs", "element_append_child")
pub fn element_append_child(element: Element, child: Element) -> Nil

@external(javascript, "./js_bindings_ffi.mjs", "element_remove_child")
pub fn element_remove_child(element: Element, child: Element) -> Nil

@external(javascript, "./js_bindings_ffi.mjs", "element_style")
pub fn element_style(elment: Element) -> Style

@external(javascript, "./js_bindings_ffi.mjs", "element_width")
pub fn element_width(element: Element) -> Int

@external(javascript, "./js_bindings_ffi.mjs", "element_set_width")
pub fn element_set_width(element: Element, width: Int) -> Nil

@external(javascript, "./js_bindings_ffi.mjs", "element_height")
pub fn element_height(element: Element) -> Int

@external(javascript, "./js_bindings_ffi.mjs", "element_set_height")
pub fn element_set_height(element: Element, height: Int) -> Nil

@external(javascript, "./js_bindings_ffi.mjs", "html_collection_length")
pub fn html_collection_length(collection: HTMLCollection) -> Int

@external(javascript, "./js_bindings_ffi.mjs", "html_collection_item")
pub fn html_collection_item(collection: HTMLCollection, index: Int) -> Element

@external(javascript, "./js_bindings_ffi.mjs", "style_set")
pub fn style_set(style: Style, key: String, value: String) -> Nil

@external(javascript, "./js_bindings_ffi.mjs", "canvas_get_context_2d")
pub fn canvas_get_context_2d(canvas: Element) -> CanvasRenderingContext2D

@external(javascript, "./js_bindings_ffi.mjs", "context_2d_fill_style")
pub fn context_2d_fill_style(
  context: CanvasRenderingContext2D,
  fill: String,
) -> Nil

@external(javascript, "./js_bindings_ffi.mjs", "context_2d_fill_rect")
pub fn context_2d_fill_rect(
  context: CanvasRenderingContext2D,
  x: Float,
  y: Float,
  width: Float,
  height: Float,
) -> Nil

@external(javascript, "./js_bindings_ffi.mjs", "request_animation_frame")
pub fn request_animation_frame(f: fn(Float) -> Nil) -> Nil

@external(javascript, "./js_bindings_ffi.mjs", "window_to_event_receiver")
pub fn window_to_event_receiver(window: Window) -> EventReceiver

@external(javascript, "./js_bindings_ffi.mjs", "add_event_listener")
pub fn add_event_listener(
  receiver: EventReceiver,
  event_name: String,
  callback: fn(Event) -> Nil,
) -> Nil
