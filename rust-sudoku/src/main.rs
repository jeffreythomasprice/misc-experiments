mod dom;
mod errors;
mod graphics;
mod sudoku;
use std::{
    mem::forget,
    panic,
    sync::{Arc, Mutex},
};

use errors::*;

use dom::{body, create_canvas, get_context, window};
use graphics::{Point, Rectangle, Size, UIState};
use log::*;
use rand::{rngs::ThreadRng, thread_rng};
use sudoku::{GameState, Number};
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, MouseEvent,
};

struct AppState {
    rng: ThreadRng,

    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,

    state: GameState,
    ui_state: UIState,
}

impl AppState {
    fn new(canvas: HtmlCanvasElement, context: CanvasRenderingContext2d) -> Result<Self> {
        let mut rng = thread_rng();
        let state = GameState::new_random(&mut rng, 25)?;
        Ok(Self {
            rng,

            canvas,
            context,

            state,
            ui_state: UIState::new(Rectangle::from_two_points(
                &Point { x: 0.0, y: 0.0 },
                &Point { x: 0.0, y: 0.0 },
            )),
        })
    }

    fn resize(&mut self) -> Result<()> {
        let width = window()?
            .inner_width()?
            .as_f64()
            .ok_or("failed to get width as f64")?;
        let height = window()?
            .inner_height()?
            .as_f64()
            .ok_or("failed to get height as f64".to_string())?;

        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);

        if let Ok(size) = Size::new(width, height) {
            self.ui_state
                .set_destination_bounds(Rectangle::from_origin_size(
                    Point { x: 0.0, y: 0.0 },
                    size,
                ));
        }

        Ok(())
    }

    fn mousemove(&mut self, e: MouseEvent) -> Result<()> {
        self.ui_state.hover(
            &self.state,
            &Point {
                x: e.client_x() as f64,
                y: e.client_y() as f64,
            },
        )?;
        Ok(())
    }

    fn mouseup(&mut self, e: MouseEvent) -> Result<()> {
        self.ui_state.select(
            &self.state,
            Some(&Point {
                x: e.client_x() as f64,
                y: e.client_y() as f64,
            }),
        )?;
        Ok(())
    }

    fn keyup(&mut self, e: KeyboardEvent) -> Result<()> {
        info!(
            "TODO code={}, key={}, key_code={}",
            e.code(),
            e.key(),
            e.key_code()
        );

        let mut number: Option<Number> = None;
        match e.code().as_str() {
            "Digit1" | "Numpad1" => number = Some(1.try_into()?),
            "Digit2" | "Numpad2" => number = Some(2.try_into()?),
            "Digit3" | "Numpad3" => number = Some(3.try_into()?),
            "Digit4" | "Numpad4" => number = Some(4.try_into()?),
            "Digit5" | "Numpad5" => number = Some(5.try_into()?),
            "Digit6" | "Numpad6" => number = Some(6.try_into()?),
            "Digit7" | "Numpad7" => number = Some(7.try_into()?),
            "Digit8" | "Numpad8" => number = Some(8.try_into()?),
            "Digit9" | "Numpad9" => number = Some(9.try_into()?),
            "Escape" => self.ui_state.select(&self.state, None)?,
            "KeyP" => self.ui_state.toggle_pencil_mode()?,
            // TODO undo, redo
            // TODO delete
            // TODO copy, paste
            _ => (),
        };

        if let Some(number) = number {
            self.ui_state.number(&mut self.state, number)?;
        }

        Ok(())
    }

    fn animate(&mut self, _time: f64) -> Result<()> {
        self.ui_state.draw_to_context(&self.context, &self.state)?;

        Ok(())
    }
}

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| e.to_string())?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let canvas = create_canvas()?;
    canvas.style().set_property("position", "absolute")?;
    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;
    canvas.style().set_property("left", "0px")?;
    canvas.style().set_property("top", "0px")?;
    body()?.replace_children_with_node_1(&canvas);

    let context = get_context(&canvas)?;

    let state = Arc::new(Mutex::new(AppState::new(canvas, context)?));

    // resize events
    {
        let state = state.clone();
        let c = Closure::<dyn Fn()>::new(move || {
            let mut state = state.lock().unwrap();
            if let Err(e) = state.resize() {
                error!("{e:?}");
            }
        });
        window()?.add_event_listener_with_callback("resize", c.as_ref().unchecked_ref())?;
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
                .add_event_listener_with_callback("mousemove", c.as_ref().unchecked_ref())?;
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
                .add_event_listener_with_callback("mouseup", c.as_ref().unchecked_ref())?;
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
        window()?.add_event_listener_with_callback("keyup", c.as_ref().unchecked_ref())?;
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
        window()?.request_animation_frame(c.as_ref().unchecked_ref())?;
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