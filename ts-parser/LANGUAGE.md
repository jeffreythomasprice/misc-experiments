TODO design a language

goals:
- emit valid typescript
- rust-like, or maybe gleam-like
- but with at least some interop with other typescript

enum Color {
	Red,
	Green,
	Blue
}

struct Point {
	x: number,
	y: number,
}

struct Vertex {
	p: Point,
	c: Color,
}

enum Result<T,E> {
	Ok(T),
	Err(E),
}

enum Option<T> {
	Some(T),
	None,
}

fn main() {
	let c = parseColor("red");
	println(`c = ${c}`);
	let c = parseColor("orange");
	println(`c = ${c}`);
}

fn parseColor(s: string) -> Some(Color) {
	match s.toLower() {
		"red" => Option::Some(Color::Red),
		"green" => Option::Some(Color::Green),
		"blue" => Option::Some(Color::Blue),
		_ => None,
	}
}
