use sauron::{node, text, Application, Cmd, Program};

enum Message {
    Increment,
}

struct App {
    count: i32,
}

impl Application for App {
    type MSG = Message;

    fn update(&mut self, msg: Self::MSG) -> sauron::Cmd<Self::MSG> {
        match msg {
            Message::Increment => self.count += 1,
        };
        Cmd::none()
    }

    fn view(&self) -> sauron::Node<Self::MSG> {
        node! {
            <main>
                <div>{text(format!("Count: {}", self.count))}</div>
                <button on_click=|_| Message::Increment>Click Me</button>
            </main>
        }
    }
}

fn main() {
    Program::mount_to_body(App { count: 0 });
}
