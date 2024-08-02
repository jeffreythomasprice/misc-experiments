use std::{
    ops::DerefMut,
    rc::Rc,
    sync::{Arc, Mutex},
};

use log::*;

use lib::{
    graphics::{
        fill_rectangle, fill_string, fit_strings_to_size, stroke_line, stroke_rectangle,
        HorizontalStringAlign, RGBColor, Rectangle, Renderer, Size, VerticalStringAlign,
    },
    AllPointsIterator, Cell, CellStatus, Coordinate, GameState, NeighborIterator, Number, Point,
    Result,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonRow {
    Top,
    Bottom,
}

#[derive(Debug)]
struct ButtonLocation {
    row: ButtonRow,
    column: Number,
}

struct Button<R>
where
    R: Renderer,
{
    location: ButtonLocation,
    on_draw: Box<
        dyn Fn(&ButtonLocation, &UIState<R>, &DependentState, &mut R, &GameState) -> Result<()>,
    >,
    on_click: Box<dyn Fn(&ButtonLocation, &mut UIState<R>, &mut GameState)>,
}

struct DependentState {
    puzzle_bounds: Rectangle,
    cell_size: Size,
    sub_cell_size: Size,

    buttons_bounds: Rectangle,
    cell_font_scale: rusttype::Scale,
    sub_cell_font_scale: rusttype::Scale,
}

impl DependentState {
    fn cell_bounds(&self, p: Point) -> Result<Rectangle> {
        //TODO cache
        Ok(Rectangle::from_origin_size(
            lib::graphics::Point {
                x: self.puzzle_bounds.origin().x + (p.column.0 as f64) * self.cell_size.width(),
                y: self.puzzle_bounds.origin().y + (p.row.0 as f64) * self.cell_size.height(),
            },
            self.cell_size,
        ))
    }

    fn sub_cell_bounds(&self, p: Point, x_sub: i8, y_sub: i8) -> Result<Rectangle> {
        //TODO cache
        let cell_bounds = self.cell_bounds(p)?;
        Ok(Rectangle::from_origin_size(
            lib::graphics::Point {
                x: cell_bounds.origin().x + (x_sub as f64) * self.sub_cell_size.width(),
                y: cell_bounds.origin().y + (y_sub as f64) * self.sub_cell_size.height(),
            },
            self.sub_cell_size,
        ))
    }

    fn button_bounds(&self, location: &ButtonLocation) -> Result<Rectangle> {
        // TODO cache
        let column: i8 = location.column.into();
        let x = self.buttons_bounds.min().x + ((column - 1) as f64) * self.cell_size.width();
        let y = self.buttons_bounds.min().y
            + match location.row {
                ButtonRow::Top => 0.0,
                ButtonRow::Bottom => self.cell_size.height(),
            };
        Ok(Rectangle::from_origin_size(
            lib::graphics::Point { x, y },
            self.cell_size,
        ))
    }

    fn draw_button_str<R>(
        &self,
        ui: &UIState<R>,
        renderer: &mut R,
        location: &ButtonLocation,
        s: &str,
        selected: bool,
    ) -> Result<()>
    where
        R: Renderer,
    {
        let button_bounds = self.button_bounds(location)?;

        let bg_color = if selected {
            &ui.button_selected_color
        } else {
            &ui.button_deselected_color
        };
        fill_rectangle(renderer, &button_bounds, bg_color);

        stroke_rectangle(renderer, &button_bounds, &ui.button_border_color, 1.0);

        renderer.set_fill_color(&ui.button_text_color);
        fill_string(
            renderer,
            s,
            &button_bounds,
            &ui.font,
            self.cell_font_scale,
            HorizontalStringAlign::Center,
            VerticalStringAlign::Center,
        )?;

        Ok(())
    }

    fn draw_button_svg<R>(
        &self,
        ui: &UIState<R>,
        renderer: &mut R,
        location: &ButtonLocation,
        svg: &R::SVG,
        selected: bool,
    ) -> Result<()>
    where
        R: Renderer,
    {
        // TODO top part of this is shared with the other draw_button
        let button_bounds = self.button_bounds(location)?;

        let bg_color = if selected {
            &ui.button_selected_color
        } else {
            &ui.button_deselected_color
        };
        fill_rectangle(renderer, &button_bounds, bg_color);

        stroke_rectangle(renderer, &button_bounds, &ui.button_border_color, 1.0);

        // TODO what to do if selected = true?

        renderer.draw_svg(svg, &button_bounds)?;

        Ok(())
    }
}

pub struct UIState<R>
where
    R: Renderer,
{
    renderer: Arc<Mutex<R>>,
    destination_bounds: Rectangle,
    font: rusttype::Font<'static>,
    copy_svg: R::SVG,

    background_color: RGBColor,
    puzzle_color: RGBColor,
    hover_color: RGBColor,
    select_color: RGBColor,
    grid_line_color: RGBColor,
    puzzle_input_text_color: RGBColor,
    solution_text_color: RGBColor,
    conflict_text_color: RGBColor,
    buttons_background_color: RGBColor,
    button_deselected_color: RGBColor,
    button_selected_color: RGBColor,
    button_border_color: RGBColor,
    button_text_color: RGBColor,

    buttons: Vec<Rc<Button<R>>>,

    dependent_state: Option<Result<Rc<DependentState>>>,

    hover_location: Option<Point>,
    select_location: Option<Point>,
    is_penciling: bool,
    clipboard: Option<Cell>,
}

impl<R> UIState<R>
where
    R: Renderer,
{
    pub fn new(
        renderer: Arc<Mutex<R>>,
        destination_bounds: Rectangle,
        font: rusttype::Font<'static>,
        copy_svg: R::SVG,
    ) -> Result<Self> {
        let mut buttons = Vec::new();
        for number in Number::all() {
            let row = ButtonRow::Top;
            buttons.push(Rc::new(Button {
                location: ButtonLocation {
                    row,
                    column: number,
                },
                on_draw: Box::new(|location, ui, ds, renderer, state| {
                    let number = location.column;
                    let is_number_selected = match ui.select_location {
                        Some(p) => match state[p] {
                            Cell::Empty => false,
                            Cell::PuzzleInput(_) => false,
                            Cell::Solution(value) => value == number,
                            Cell::PencilMark(value) => value.is_set(number),
                        },
                        None => false,
                    };
                    ds.draw_button_str(
                        ui,
                        renderer,
                        location,
                        format!("{}", number).as_str(),
                        is_number_selected,
                    )?;
                    Ok(())
                }),
                on_click: Box::new(|location, ui, state| {
                    let number = location.column;
                    ui.number(state, number)
                }),
            }));
        }
        buttons.push(Rc::new(Button {
            location: ButtonLocation {
                row: ButtonRow::Bottom,
                column: 1.try_into()?,
            },
            on_draw: Box::new(|location, ui, ds, renderer, _state| {
                // TODO pencil svg
                ds.draw_button_str(ui, renderer, location, "P", ui.is_penciling)?;
                Ok(())
            }),
            on_click: Box::new(|_location, ui, _state| {
                ui.toggle_pencil_mode();
            }),
        }));
        buttons.push(Rc::new(Button {
            location: ButtonLocation {
                row: ButtonRow::Bottom,
                column: 2.try_into()?,
            },
            on_draw: Box::new(|location, ui, ds, renderer, _state| {
                ds.draw_button_svg(ui, renderer, location, &ui.copy_svg, false)?;
                Ok(())
            }),
            on_click: Box::new(|_location, _ui, _state| {
                todo!("copy");
            }),
        }));
        buttons.push(Rc::new(Button {
            location: ButtonLocation {
                row: ButtonRow::Bottom,
                column: 3.try_into()?,
            },
            on_draw: Box::new(|location, ui, ds, renderer, _state| {
                ds.draw_button_str(ui, renderer, location, "📋", false)?;
                Ok(())
            }),
            on_click: Box::new(|_location, _ui, _state| {
                todo!("paste");
            }),
        }));
        buttons.push(Rc::new(Button {
            location: ButtonLocation {
                row: ButtonRow::Bottom,
                column: 4.try_into()?,
            },
            on_draw: Box::new(|location, ui, ds, renderer, _state| {
                ds.draw_button_str(ui, renderer, location, "🗑", false)?;
                Ok(())
            }),
            on_click: Box::new(|_location, _ui, _state| {
                todo!("delete");
            }),
        }));
        buttons.push(Rc::new(Button {
            location: ButtonLocation {
                row: ButtonRow::Bottom,
                column: 5.try_into()?,
            },
            on_draw: Box::new(|location, ui, ds, renderer, _state| {
                ds.draw_button_str(ui, renderer, location, "⎌", false)?;
                Ok(())
            }),
            on_click: Box::new(|_location, _ui, _state| {
                todo!("undo");
            }),
        }));
        buttons.push(Rc::new(Button {
            location: ButtonLocation {
                row: ButtonRow::Bottom,
                column: 6.try_into()?,
            },
            on_draw: Box::new(|location, ui, ds, renderer, _state| {
                ds.draw_button_str(ui, renderer, location, "⟳", false)?;
                Ok(())
            }),
            on_click: Box::new(|_location, _ui, _state| {
                todo!("redo");
            }),
        }));

        Ok(Self {
            renderer,
            destination_bounds,
            font,
            copy_svg,

            background_color: RGBColor {
                red: 0x22,
                green: 0x22,
                blue: 0x22,
            },
            puzzle_color: RGBColor {
                red: 0xff,
                green: 0xff,
                blue: 0xff,
            },
            hover_color: RGBColor {
                red: 0xdd,
                green: 0xdd,
                blue: 0xff,
            },
            select_color: RGBColor {
                red: 0x66,
                green: 0x66,
                blue: 0xff,
            },
            grid_line_color: RGBColor {
                red: 0x00,
                green: 0x00,
                blue: 0x00,
            },
            puzzle_input_text_color: RGBColor {
                red: 0x55,
                green: 0x55,
                blue: 0x55,
            },
            solution_text_color: RGBColor {
                red: 0x00,
                green: 0x00,
                blue: 0x00,
            },
            conflict_text_color: RGBColor {
                red: 0xff,
                green: 0x00,
                blue: 0x00,
            },
            buttons_background_color: RGBColor {
                red: 0xff,
                green: 0x99,
                blue: 0x99,
            },
            button_deselected_color: RGBColor {
                red: 0x88,
                green: 0x88,
                blue: 0x88,
            },
            button_selected_color: RGBColor {
                red: 0xdd,
                green: 0xdd,
                blue: 0xff,
            },
            button_border_color: RGBColor {
                red: 0x00,
                green: 0x00,
                blue: 0x00,
            },
            button_text_color: RGBColor {
                red: 0x00,
                green: 0x00,
                blue: 0x00,
            },

            buttons,

            dependent_state: None,

            hover_location: None,
            select_location: None,
            is_penciling: false,
            clipboard: None,
        })
    }

    pub fn destination_bounds(&self) -> &Rectangle {
        &self.destination_bounds
    }

    pub fn set_destination_bounds(&mut self, r: Rectangle) {
        self.destination_bounds = r;
        self.dependent_state = None;
    }

    pub fn selected(&self) -> Option<Point> {
        self.select_location
    }

    pub fn hover(&mut self, p: &lib::graphics::Point) -> Result<()> {
        let ds = self.refresh_dependent_state()?;

        for sp in AllPointsIterator::new() {
            if ds.cell_bounds(sp)?.contains(p) {
                self.hover_location = Some(sp);
                return Ok(());
            }
        }
        self.hover_location = None;

        for button in self.buttons.iter() {
            if ds.button_bounds(&button.location)?.contains(p) {
                trace!("TODO hover text for button {:?}", button.location);
                break;
            }
        }

        Ok(())
    }

    pub fn select(
        &mut self,
        state: &mut GameState,
        p: Option<&lib::graphics::Point>,
    ) -> Result<()> {
        let ds = self.refresh_dependent_state()?;

        match p {
            Some(p) => {
                for sp in AllPointsIterator::new() {
                    if ds.cell_bounds(sp)?.contains(p) {
                        self.select_location = Some(sp);
                        trace!("selecting puzzle cell {sp:?}");
                        return Ok(());
                    }
                }

                let mut selected_button = None;
                for button in self.buttons.iter() {
                    if ds.button_bounds(&button.location)?.contains(p) {
                        selected_button = Some(button.clone());
                    }
                }
                if let Some(button) = selected_button {
                    trace!("selecting button {:?}", button.location);
                    (*button.on_click)(&button.location, self, state);
                    return Ok(());
                }

                trace!("deselecting because clicked off puzzle");
                self.select_location = None;
            }
            None => {
                trace!("force deselecting");
                self.select_location = None
            }
        }
        Ok(())
    }

    pub fn move_select(&mut self, rows: i8, columns: i8) -> Result<()> {
        self.select_location = match self.select_location {
            Some(cur) => {
                let row: Result<Coordinate> = (cur.row.0 + rows).try_into();
                let column: Result<Coordinate> = (cur.column.0 + columns).try_into();
                match (row, column) {
                    (Ok(row), Ok(column)) => {
                        let result = Point { row, column };
                        trace!("moving to {result:?}");
                        Some(result)
                    }
                    _ => {
                        trace!("deselecting, moving off the puzzle");
                        None
                    }
                }
            }
            None => {
                trace!("no selection, can't move");
                None
            }
        };
        Ok(())
    }

    pub fn number(&self, state: &mut GameState, number: Number) {
        let is_penciling = self.is_penciling;
        self.set_selected_cell_value(state, |existing| {
            Cell::from_input(*existing, number, is_penciling)
        });
    }

    pub fn clear(&mut self, state: &mut GameState) {
        self.set_selected_cell_value(state, |_| Cell::Empty);
    }

    pub fn toggle_pencil_mode(&mut self) {
        self.is_penciling = !self.is_penciling;
        debug!("is_penciling = {}", self.is_penciling);
    }

    pub fn copy(&mut self, state: &GameState) {
        self.clipboard = self.select_location.and_then(|p| match state[p] {
            Cell::PuzzleInput(_) => None,
            value => Some(value),
        });
        trace!("clipboard = {:?}", self.clipboard);
    }

    pub fn paste(&mut self, state: &mut GameState) {
        if let Some(clipboard) = self.clipboard {
            self.set_selected_cell_value(state, |_| clipboard);
        }
    }

    pub fn draw_to_context(&mut self, state: &GameState) -> Result<()> {
        let ds = self.refresh_dependent_state()?;

        let renderer = &mut self.renderer.lock().unwrap();
        let renderer = renderer.deref_mut();

        fill_rectangle(renderer, &self.destination_bounds, &self.background_color);

        fill_rectangle(renderer, &ds.puzzle_bounds, &self.puzzle_color);

        fill_rectangle(renderer, &ds.buttons_bounds, &self.buttons_background_color);

        for p in AllPointsIterator::new() {
            let cell_bounds = ds.cell_bounds(p)?;

            if self.select_location == Some(p) {
                fill_rectangle(renderer, &cell_bounds, &self.select_color);
            } else if self.hover_location == Some(p) {
                fill_rectangle(renderer, &cell_bounds, &self.hover_color);
            }

            let (cell_value, cell_status) = state.status_at(&p);
            match cell_value {
                Cell::Empty => (),
                Cell::PuzzleInput(value) => {
                    renderer.set_fill_color(&self.puzzle_input_text_color);
                    fill_string(
                        renderer,
                        &format!("{}", value),
                        &cell_bounds,
                        &self.font,
                        ds.cell_font_scale,
                        HorizontalStringAlign::Center,
                        VerticalStringAlign::Center,
                    )?;
                }
                Cell::Solution(value) => {
                    renderer.set_fill_color(match cell_status {
                        CellStatus::Conflict => &self.conflict_text_color,
                        CellStatus::NoConflict => &self.solution_text_color,
                    });
                    fill_string(
                        renderer,
                        &format!("{}", value),
                        &cell_bounds,
                        &self.font,
                        ds.cell_font_scale,
                        HorizontalStringAlign::Center,
                        VerticalStringAlign::Center,
                    )?;
                }
                Cell::PencilMark(value) => {
                    for x_sub in 0..3 {
                        for y_sub in 0..3 {
                            let number = (y_sub * 3 + x_sub + 1).try_into()?;
                            let sub_cell_bounds = ds.sub_cell_bounds(p, x_sub, y_sub)?;

                            if value.is_set(number) {
                                renderer.set_fill_color(&self.solution_text_color);
                                fill_string(
                                    renderer,
                                    &format!("{}", number),
                                    &sub_cell_bounds,
                                    &self.font,
                                    ds.sub_cell_font_scale,
                                    HorizontalStringAlign::Center,
                                    VerticalStringAlign::Center,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        for i in 0..=9 {
            let x = ds.puzzle_bounds.min().x + (i as f64) * ds.cell_size.width();
            let y = ds.puzzle_bounds.min().y + (i as f64) * ds.cell_size.height();
            let width = if i % 3 == 0 { 5.0 } else { 1.0 };
            let c = &self.grid_line_color;
            stroke_line(
                renderer,
                &lib::graphics::Point {
                    x,
                    y: ds.puzzle_bounds.min().y,
                },
                &lib::graphics::Point {
                    x,
                    y: ds.puzzle_bounds.max().y,
                },
                c,
                width,
            );
            stroke_line(
                renderer,
                &lib::graphics::Point {
                    x: ds.puzzle_bounds.min().x,
                    y,
                },
                &lib::graphics::Point {
                    x: ds.puzzle_bounds.max().x,
                    y,
                },
                c,
                width,
            );
        }

        for button in self.buttons.iter() {
            (*button.on_draw)(&button.location, self, &ds, renderer, state)?;
        }

        Ok(())
    }

    fn refresh_dependent_state(&mut self) -> Result<Rc<DependentState>> {
        let renderer = &mut self.renderer.lock().unwrap();
        let renderer = renderer.deref_mut();

        self.dependent_state
            .get_or_insert_with(|| {
                // number of effective cells
                // game board is only 9 columns wide
                // but including the buttons at the bottom it's like it's 11 rows tall
                let desired_aspect_ratio = Size::new(9.0, 11.0)?;
                let total_size_if_width_equals_destination = Size::new(
                    self.destination_bounds.size().width(),
                    self.destination_bounds.size().width() * desired_aspect_ratio.height()
                        / desired_aspect_ratio.width(),
                )?;
                let total_size_if_height_equals_destination = Size::new(
                    self.destination_bounds.size().height() * desired_aspect_ratio.width()
                        / desired_aspect_ratio.height(),
                    self.destination_bounds.size().height(),
                )?;
                // the size of the game board plus buttons
                let total_size = if total_size_if_width_equals_destination.height()
                    > self.destination_bounds.size().height()
                {
                    total_size_if_height_equals_destination
                } else {
                    total_size_if_width_equals_destination
                };
                // the bounding rectangle in destination of the game board plus buttons
                let total_bounds =
                    Rectangle::from_centered_size(&self.destination_bounds, total_size);

                // the bounding rectangle for drawing just the game board
                let puzzle_bounds = Rectangle::from_origin_size(
                    *total_bounds.origin(),
                    Size::new(total_bounds.size().width(), total_bounds.size().width())?,
                );

                // the bounding rectangle for drawing the buttons
                let buttons_bounds = Rectangle::from_two_points(
                    &lib::graphics::Point {
                        x: total_bounds.min().x,
                        y: puzzle_bounds.max().y,
                    },
                    &lib::graphics::Point {
                        x: total_bounds.max().x,
                        y: total_bounds.max().y,
                    },
                );

                let cell_size = total_size.width() / 9.0;
                let sub_cell_size = cell_size / 3.0;
                let cell_size =
                    Size::new(cell_size, cell_size).map_err(|e| format!("bad cell size: {e:?}"))?;
                let sub_cell_size = Size::new(sub_cell_size, sub_cell_size)
                    .map_err(|e| format!("bad sub-cell size: {e:?}"))?;

                let possible_strings = vec![
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    "4".into(),
                    "5".into(),
                    "6".into(),
                    "7".into(),
                    "8".into(),
                    "9".into(),
                ];
                let cell_font_scale =
                    fit_strings_to_size(&possible_strings, &cell_size, &self.font)?
                        .ok_or("expected a font but got none".to_string())?;
                let cell_font_scale = rusttype::Scale {
                    x: cell_font_scale.x * 0.8,
                    y: cell_font_scale.y * 0.8,
                };
                let sub_cell_font_scale =
                    fit_strings_to_size(&possible_strings, &sub_cell_size, &self.font)?
                        .ok_or("expected a font but got none".to_string())?;
                let sub_cell_font_scale = rusttype::Scale {
                    x: sub_cell_font_scale.x * 0.8,
                    y: sub_cell_font_scale.y * 0.8,
                };

                Ok(Rc::new(DependentState {
                    puzzle_bounds,
                    cell_size,
                    sub_cell_size,
                    buttons_bounds,
                    cell_font_scale,
                    sub_cell_font_scale,
                }))
            })
            .clone()
    }

    fn set_selected_cell_value<F>(&self, state: &mut GameState, f: F)
    where
        F: FnOnce(&Cell) -> Cell,
    {
        if let Some(p) = self.select_location {
            let p_value = state[p];
            match p_value {
                Cell::PuzzleInput(_) => {
                    trace!("not assigning value because selected cell is puzzle input");
                }
                _ => {
                    let new = f(&p_value);
                    trace!("replacing {:?} with {:?}", p_value, new);
                    if let Cell::Solution(number) = new {
                        trace!("removing {number} from neighboring pencil marks");
                        for q in NeighborIterator::new(p) {
                            if let Cell::PencilMark(mask) = state[q] {
                                state[q] = Cell::PencilMark(mask.clear(number));
                            }
                        }
                    }
                    state[p] = new;
                }
            };
        } else {
            trace!("no selected cell, can't assign new value");
        }
    }
}
