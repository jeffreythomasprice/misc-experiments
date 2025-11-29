export function get_document() {
	return document;
}

export function get_window() {
	return window;
}

export function window_inner_width(window) {
	return window.innerWidth;
}

export function window_inner_height(window) {
	return window.innerHeight;
}

export function document_body(document) {
	return document.body;
}

export function document_create_element(document, tag) {
	return document.createElement(tag);
}

export function element_children(element) {
	return element.children;
}

export function element_append_child(element, child) {
	element.appendChild(child);
}

export function element_remove_child(element, child) {
	element.removeChild(child);
}

export function element_style(element) {
	return element.style;
}

export function element_width(element) {
	return element.width;
}

export function element_set_width(element, width) {
	element.width = width;
}

export function element_height(element) {
	return element.height;
}

export function element_set_height(element, height) {
	element.height = height;
}

export function html_collection_length(collection) {
	return collection.length;
}

export function html_collection_item(collection, index) {
	return collection.item(index);
}

export function style_set(style, key, value) {
	style[key] = value;
}

export function canvas_get_context_2d(canvas) {
	return canvas.getContext("2d");
}

export function context_2d_fill_style(context, fill) {
	context.fillStyle = fill;
}

export function context_2d_fill_rect(context, x, y, width, height) {
	context.fillRect(x, y, width, height);
}

export function request_animation_frame(callback) {
	window.requestAnimationFrame(callback);
}

export function window_to_event_receiver(window) {
	return window;
}

export function add_event_listener(receiver, event_name, callback) {
	receiver.addEventListener(event_name, callback);
}