mod dom;
mod graphics;
use std::{
    mem::forget,
    panic,
    sync::{Arc, Mutex},
};

use dom::{body, create_canvas, window};
use graphics::{CanvasRenderingContext2dRenderer, UIState};
use lib::{
    graphics::{Rectangle, Renderer, Size},
    Number, Result,
};
use lib::{GameState, History};
use log::*;
use rand::thread_rng;
use rusttype::Font;
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    HtmlCanvasElement, KeyboardEvent, MouseEvent,
};

struct AppState {
    canvas: HtmlCanvasElement,
    renderer: CanvasRenderingContext2dRenderer,

    state: History<GameState>,
    ui_state: UIState<CanvasRenderingContext2dRenderer>,
}

impl AppState {
    fn new(canvas: HtmlCanvasElement, renderer: CanvasRenderingContext2dRenderer) -> Result<Self> {
        let mut rng = thread_rng();
        let state = GameState::new_random(&mut rng, 25)?;

        let font = Font::try_from_bytes(include_bytes!(
            "../assets/Space_Grotesk/static/SpaceGrotesk-Medium.ttf"
        ))
        .ok_or(format!("failed to parse font"))?;

        Ok(Self {
            canvas,
            renderer,

            state: History::new(state),
            ui_state: UIState::new(
                Rectangle::from_two_points(
                    &lib::graphics::Point { x: 0.0, y: 0.0 },
                    &lib::graphics::Point { x: 0.0, y: 0.0 },
                ),
                font,
            )?,
        })
    }

    fn resize(&mut self) -> Result<()> {
        let width = window()?
            .inner_width()
            .map_err(|e| format!("{e:?}"))?
            .as_f64()
            .ok_or("failed to get width as f64")?;
        let height = window()?
            .inner_height()
            .map_err(|e| format!("{e:?}"))?
            .as_f64()
            .ok_or("failed to get height as f64".to_string())?;

        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);

        if let Ok(size) = Size::new(width, height) {
            self.ui_state
                .set_destination_bounds(Rectangle::from_origin_size(
                    lib::graphics::Point { x: 0.0, y: 0.0 },
                    size,
                ));
        }

        Ok(())
    }

    fn mousemove(&mut self, e: MouseEvent) -> Result<()> {
        self.ui_state.hover(&lib::graphics::Point {
            x: e.client_x() as f64,
            y: e.client_y() as f64,
        })?;
        Ok(())
    }

    fn mouseup(&mut self, e: MouseEvent) -> Result<()> {
        self.apply_state_change(|app, state| {
            app.ui_state.select(
                state,
                Some(&lib::graphics::Point {
                    x: e.client_x() as f64,
                    y: e.client_y() as f64,
                }),
            )
        })
    }

    fn keyup(&mut self, e: KeyboardEvent) -> Result<()> {
        let mut number: Option<Number> = None;
        match (e.code().as_str(), e.ctrl_key()) {
            ("Digit1", false) | ("Numpad1", false) => number = Some(1.try_into()?),
            ("Digit2", false) | ("Numpad2", false) => number = Some(2.try_into()?),
            ("Digit3", false) | ("Numpad3", false) => number = Some(3.try_into()?),
            ("Digit4", false) | ("Numpad4", false) => number = Some(4.try_into()?),
            ("Digit5", false) | ("Numpad5", false) => number = Some(5.try_into()?),
            ("Digit6", false) | ("Numpad6", false) => number = Some(6.try_into()?),
            ("Digit7", false) | ("Numpad7", false) => number = Some(7.try_into()?),
            ("Digit8", false) | ("Numpad8", false) => number = Some(8.try_into()?),
            ("Digit9", false) | ("Numpad9", false) => number = Some(9.try_into()?),
            ("Escape", false) => {
                self.apply_state_change(|app, state| app.ui_state.select(state, None))?
            }
            ("KeyP", false) => self.ui_state.toggle_pencil_mode(),
            ("Backspace", false) | ("Delete", false) => {
                self.apply_state_change(|app, state| {
                    app.ui_state.clear(state);
                    Ok(())
                })?;
            }
            ("ArrowLeft", false) => self.ui_state.move_select(0, -1)?,
            ("ArrowRight", false) => self.ui_state.move_select(0, 1)?,
            ("ArrowUp", false) => self.ui_state.move_select(-1, 0)?,
            ("ArrowDown", false) => self.ui_state.move_select(1, 0)?,
            ("KeyZ", true) => self.state.undo(),
            ("KeyY", true) => self.state.redo(),
            ("KeyC", true) => self.apply_state_change(|app, state| {
                app.ui_state.copy(state);
                Ok(())
            })?,
            ("KeyV", true) => self.apply_state_change(|app, state| {
                app.ui_state.paste(state);
                Ok(())
            })?,
            _ => (),
        };

        if let Some(number) = number {
            self.apply_state_change(|app, state| {
                app.ui_state.number(state, number);
                Ok(())
            })?;
        }

        Ok(())
    }

    fn animate(&mut self, _time: f64) -> Result<()> {
        self.ui_state
            .draw_to_context(&mut self.renderer, self.state.current())?;

        Ok(())
    }

    fn apply_state_change<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AppState, &mut GameState) -> Result<()>,
    {
        let mut state = self.state.current().clone();
        f(self, &mut state)?;
        if state != *self.state.current() {
            self.state.push(state);
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| e.to_string())?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let canvas = create_canvas()?;
    canvas
        .style()
        .set_property("position", "absolute")
        .map_err(|e| format!("{e:?}"))?;
    canvas
        .style()
        .set_property("width", "100%")
        .map_err(|e| format!("{e:?}"))?;
    canvas
        .style()
        .set_property("height", "100%")
        .map_err(|e| format!("{e:?}"))?;
    canvas
        .style()
        .set_property("left", "0px")
        .map_err(|e| format!("{e:?}"))?;
    canvas
        .style()
        .set_property("top", "0px")
        .map_err(|e| format!("{e:?}"))?;
    body()?.replace_children_with_node_1(&canvas);

    let renderer = CanvasRenderingContext2dRenderer::new_from_canvas(&canvas)?;

    let state = Arc::new(Mutex::new(AppState::new(canvas, renderer)?));

    // resize events
    {
        let state = state.clone();
        let c = Closure::<dyn Fn()>::new(move || {
            let mut state = state.lock().unwrap();
            if let Err(e) = state.resize() {
                error!("{e:?}");
            }
        });
        window()?
            .add_event_listener_with_callback("resize", c.as_ref().unchecked_ref())
            .map_err(|e| format!("{e:?}"))?;
        // don't ever free this so the js callback stays valid
        forget(c);
    }

    // mouse move events
    {
        let c = {
            let state = state.clone();
            Closure::<dyn Fn(MouseEvent)>::new(move |e: MouseEvent| {
                let mut state = state.lock().unwrap();
                if let Err(e) = state.mousemove(e) {
                    error!("{e:?}");
                }
            })
        };
        {
            let state = state.lock().unwrap();
            state
                .canvas
                .add_event_listener_with_callback("mousemove", c.as_ref().unchecked_ref())
                .map_err(|e| format!("{e:?}"))?;
            // don't ever free this so the js callback stays valid
        }
        forget(c);
    }

    // mouse up events
    {
        let c = {
            let state = state.clone();
            Closure::<dyn Fn(MouseEvent)>::new(move |e: MouseEvent| {
                let mut state = state.lock().unwrap();
                if let Err(e) = state.mouseup(e) {
                    error!("{e:?}");
                }
            })
        };
        {
            let state = state.lock().unwrap();
            state
                .canvas
                .add_event_listener_with_callback("mouseup", c.as_ref().unchecked_ref())
                .map_err(|e| format!("{e:?}"))?;
            // don't ever free this so the js callback stays valid
        }
        forget(c);
    }

    // key up events
    {
        let c = {
            let state = state.clone();
            Closure::<dyn Fn(KeyboardEvent)>::new(move |e: KeyboardEvent| {
                let mut state = state.lock().unwrap();
                if let Err(e) = state.keyup(e) {
                    error!("{e:?}");
                }
            })
        };
        window()?
            .add_event_listener_with_callback("keyup", c.as_ref().unchecked_ref())
            .map_err(|e| format!("{e:?}"))?;
        // don't ever free this so the js callback stays valid
        forget(c);
    }

    // animation
    fn request_anim_frame(state: Arc<Mutex<AppState>>) -> Result<()> {
        let c = Closure::once_into_js(move |time| {
            {
                let mut state = state.lock().unwrap();
                if let Err(e) = state.animate(time) {
                    error!("{e:?}");
                }
            }
            if let Err(e) = request_anim_frame(state) {
                error!("{e:?}");
            }
        });
        window()?
            .request_animation_frame(c.as_ref().unchecked_ref())
            .map_err(|e| format!("{e:?}"))?;
        Ok(())
    }

    // resize once on startup because resize won't be called right away
    {
        let mut state = state.lock().unwrap();
        if let Err(e) = state.resize() {
            error!("{e:?}");
        }
    }

    // kick off the first frame
    if let Err(e) = request_anim_frame(state.clone()) {
        error!("{e:?}");
    }

    Ok(())
}