use std::{cell::RefCell, rc::Rc};

use log::*;

use wasm_bindgen::{prelude::Closure, JsCast};

use crate::errors::Result;

use super::getters::get_window;

pub enum RequestAnimationFrameStatus {
    Continue,
    Stop,
}

pub fn request_animation_frame_loop<F>(f: F) -> Result<()>
where
    F: Fn(f64) -> Result<RequestAnimationFrameStatus> + 'static,
{
    // two references to the same closure
    // one is going inside the closure
    // the other will get used once immediately to register the animation
    let g1 = Rc::new(RefCell::<Option<Closure<dyn Fn(f64)>>>::new(None));
    let g2 = g1.clone();
    // assigning to one of them
    *g1.borrow_mut() = {
        let g = g2.clone();
        Some(Closure::<dyn Fn(f64)>::new(move |time: f64| {
            // execute the actual operation we were given
            match f(time) {
                Ok(RequestAnimationFrameStatus::Continue) => {
                    // some extra error handling, but we're registering this same callback again, and just abort if any errors occur
                    let window = match get_window() {
                        Ok(window) => window,
                        Err(e) => {
                            error!("error getting window to request new animation frame: {e:?}");
                            return;
                        }
                    };
                    if let Err(e) = window.request_animation_frame(
                        g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                    ) {
                        error!("error requesting new animation frame{e:?}");
                    }
                }
                Ok(RequestAnimationFrameStatus::Stop) => {
                    // intentionally empty, we're stopping
                }
                Err(e) => {
                    error!("error animating: {e:?}");
                }
            }
        }))
    };
    // and then register the first one
    get_window()?
        .request_animation_frame(g1.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    Ok(())
}
