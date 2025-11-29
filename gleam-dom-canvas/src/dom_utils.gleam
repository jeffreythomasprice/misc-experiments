import js_bindings

pub fn element_children(
  element: js_bindings.Element,
) -> List(js_bindings.Element) {
  html_collection_to_list(js_bindings.element_children(element), 0)
}

pub fn element_remove_all_children(element: js_bindings.Element) -> Nil {
  case element_children(element) {
    [child, ..] -> {
      js_bindings.element_remove_child(element, child)
      element_remove_all_children(element)
    }
    _ -> Nil
  }
}

fn html_collection_to_list(
  collection: js_bindings.HTMLCollection,
  index: Int,
) -> List(js_bindings.Element) {
  case index < js_bindings.html_collection_length(collection) {
    False -> []
    True -> {
      let next = js_bindings.html_collection_item(collection, index)
      [next, ..html_collection_to_list(collection, index + 1)]
    }
  }
}
