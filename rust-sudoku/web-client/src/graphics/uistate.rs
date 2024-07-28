use std::rc::Rc;

use log::*;
use web_sys::CanvasRenderingContext2d;

use super::text::{
    fill_string, fit_strings_to_size, Font, HorizontalStringAlign, VerticalStringAlign,
};
use lib::{
    graphics::{Rectangle, Size},
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

struct Button {
    location: ButtonLocation,
    on_draw: Box<
        dyn Fn(
            &ButtonLocation,
            &UIState,
            &DependentState,
            &CanvasRenderingContext2d,
            &GameState,
        ) -> Result<()>,
    >,
    on_click: Box<dyn Fn(&ButtonLocation, &mut UIState, &mut GameState)>,
}

struct DependentState {
    puzzle_bounds: Rectangle,
    cell_size: Size,
    sub_cell_size: Size,

    buttons_bounds: Rectangle,

    cell_font: Font,
    sub_cell_font: Font,
}

impl DependentState {
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

    fn draw_button(
        &self,
        ui: &UIState,
        context: &CanvasRenderingContext2d,
        location: &ButtonLocation,
        s: &str,
        selected: bool,
    ) -> Result<()> {
        let button_bounds = self.button_bounds(location)?;

        let bg_color = if selected {
            &ui.button_selected_color
        } else {
            &ui.button_deselected_color
        };
        context.set_fill_style(&bg_color.clone().into());
        context.begin_path();
        add_rect_to_context(context, &button_bounds);
        context.fill();

        context.set_stroke_style(&ui.button_border_color.clone().into());
        context.begin_path();
        add_rect_to_context(context, &button_bounds);
        context.stroke();

        context.set_fill_style(&ui.button_text_color.clone().into());
        fill_string(
            context,
            s,
            &button_bounds,
            &self.cell_font,
            HorizontalStringAlign::Center,
            VerticalStringAlign::Center,
        )?;

        Ok(())
    }
}

pub struct UIState {
    destination_bounds: Rectangle,

    background_color: String,
    puzzle_color: String,
    hover_color: String,
    select_color: String,
    grid_line_color: String,
    puzzle_input_text_color: String,
    solution_text_color: String,
    conflict_text_color: String,
    buttons_background_color: String,
    button_deselected_color: String,
    button_selected_color: String,
    button_border_color: String,
    button_text_color: String,

    font_family: String,

    buttons: Vec<Rc<Button>>,

    dependent_state: Option<Result<Rc<DependentState>>>,

    hover_location: Option<Point>,
    select_location: Option<Point>,
    is_penciling: bool,
    clipboard: Option<Cell>,
}

impl UIState {
    pub fn new(destination_bounds: Rectangle) -> Result<Self> {
        let mut buttons = Vec::new();
        for number in Number::all() {
            let row = ButtonRow::Top;
            buttons.push(Rc::new(Button {
                location: ButtonLocation {
                    row,
                    column: number,
                },
                on_draw: Box::new(|location, ui, ds, context, state| {
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
                    ds.draw_button(
                        ui,
                        context,
                        &location,
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
            on_draw: Box::new(|location, ui, ds, context, _state| {
                ds.draw_button(ui, context, &location, "P", ui.is_penciling)?;
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
            on_draw: Box::new(|location, ui, ds, context, _state| {
                ds.draw_button(ui, context, &location, "⎀", false)?;
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
            on_draw: Box::new(|location, ui, ds, context, _state| {
                ds.draw_button(ui, context, &location, "📋", false)?;
                Ok(())
            }),
            on_click: Box::new(|_location, _ui, _state| {
                todo!("paste");
            }),
        }));
        buttons.push(Rc::new(Button {
            location: ButtonLocation {
                row: ButtonRow::Bottom,
                column: 3.try_into()?,
            },
            on_draw: Box::new(|location, ui, ds, context, _state| {
                ds.draw_button(ui, context, &location, "📋", false)?;
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
            on_draw: Box::new(|location, ui, ds, context, _state| {
                ds.draw_button(ui, context, &location, "🗑", false)?;
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
            on_draw: Box::new(|location, ui, ds, context, _state| {
                ds.draw_button(ui, context, &location, "⎌", false)?;
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
            on_draw: Box::new(|location, ui, ds, context, _state| {
                ds.draw_button(ui, context, &location, "⟳", false)?;
                Ok(())
            }),
            on_click: Box::new(|_location, _ui, _state| {
                todo!("redo");
            }),
        }));

        Ok(Self {
            destination_bounds,

            background_color: "#222222".into(),
            puzzle_color: "white".into(),
            hover_color: "#ddddff".into(),
            select_color: "#6666ff".into(),
            grid_line_color: "black".into(),
            puzzle_input_text_color: "#555555".into(),
            solution_text_color: "black".into(),
            conflict_text_color: "red".into(),
            buttons_background_color: "#ff9999".into(),
            button_deselected_color: "#888888".into(),
            button_selected_color: "#ddddff".into(),
            button_border_color: "black".into(),
            button_text_color: "black".into(),

            font_family: "monospace".into(),

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

    pub fn font_family(&self) -> &String {
        &self.font_family
    }

    pub fn set_font_family(&mut self, s: String) {
        self.font_family = s;
        self.dependent_state = None;
    }

    pub fn selected(&self) -> Option<Point> {
        self.select_location
    }

    pub fn hover(&mut self, p: &lib::graphics::Point) -> Result<()> {
        for sp in AllPointsIterator::new() {
            if self.cell_bounds(sp)?.contains(p) {
                self.hover_location = Some(sp);
                return Ok(());
            }
        }
        self.hover_location = None;

        let ds = self.refresh_dependent_state()?;
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
        match p {
            Some(p) => {
                for sp in AllPointsIterator::new() {
                    if self.cell_bounds(sp)?.contains(p) {
                        self.select_location = Some(sp);
                        trace!("selecting puzzle cell {sp:?}");
                        return Ok(());
                    }
                }

                let ds = self.refresh_dependent_state()?;
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

    pub fn draw_to_context(
        &mut self,
        context: &CanvasRenderingContext2d,
        state: &GameState,
    ) -> Result<()> {
        let ds = self.refresh_dependent_state()?;

        context.set_fill_style(&self.background_color.clone().into());
        context.begin_path();
        add_rect_to_context(context, &self.destination_bounds);
        context.fill();

        context.set_fill_style(&self.puzzle_color.clone().into());
        context.begin_path();
        add_rect_to_context(context, &ds.puzzle_bounds);
        context.fill();

        context.set_fill_style(&self.buttons_background_color.clone().into());
        context.begin_path();
        add_rect_to_context(context, &ds.buttons_bounds);
        context.fill();

        for p in AllPointsIterator::new() {
            let cell_bounds = self.cell_bounds(p)?;

            if self.select_location == Some(p) {
                context.set_fill_style(&self.select_color.clone().into());
                context.begin_path();
                add_rect_to_context(context, &cell_bounds);
                context.fill();
            } else if self.hover_location == Some(p) {
                context.set_fill_style(&self.hover_color.clone().into());
                context.begin_path();
                add_rect_to_context(context, &cell_bounds);
                context.fill();
            }

            let (cell_value, cell_status) = state.status_at(&p);
            match cell_value {
                Cell::Empty => (),
                Cell::PuzzleInput(value) => {
                    context.set_fill_style(&self.puzzle_input_text_color.clone().into());
                    fill_string(
                        context,
                        &format!("{}", value),
                        &cell_bounds,
                        &ds.cell_font,
                        HorizontalStringAlign::Center,
                        VerticalStringAlign::Center,
                    )?;
                }
                Cell::Solution(value) => {
                    let color = match cell_status {
                        CellStatus::Conflict => &self.conflict_text_color,
                        CellStatus::NoConflict => &self.solution_text_color,
                    };
                    context.set_fill_style(&color.clone().into());
                    fill_string(
                        context,
                        &format!("{}", value),
                        &cell_bounds,
                        &ds.cell_font,
                        HorizontalStringAlign::Center,
                        VerticalStringAlign::Center,
                    )?;
                }
                Cell::PencilMark(value) => {
                    for x_sub in 0..3 {
                        for y_sub in 0..3 {
                            let number = (y_sub * 3 + x_sub + 1).try_into()?;
                            let sub_cell_bounds = self.sub_cell_bounds(p, x_sub, y_sub)?;

                            if value.is_set(number) {
                                context.set_fill_style(&self.solution_text_color.clone().into());
                                fill_string(
                                    context,
                                    &format!("{}", number),
                                    &sub_cell_bounds,
                                    &ds.sub_cell_font,
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
            context.set_stroke_style(&self.grid_line_color.clone().into());
            context.set_line_width(if i % 3 == 0 { 5.0 } else { 1.0 });
            context.begin_path();
            context.move_to(x, ds.puzzle_bounds.min().y);
            context.line_to(x, ds.puzzle_bounds.max().y);
            context.stroke();
            context.begin_path();
            context.move_to(ds.puzzle_bounds.min().x, y);
            context.line_to(ds.puzzle_bounds.max().x, y);
            context.stroke();
        }

        for button in self.buttons.iter() {
            (*button.on_draw)(&button.location, self, &ds, context, state)?;
        }

        Ok(())
    }

    fn refresh_dependent_state(&mut self) -> Result<Rc<DependentState>> {
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

                let font_family = self.font_family.clone();
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
                let cell_font = fit_strings_to_size(&possible_strings, &cell_size, &font_family)?
                    .ok_or("expected a font but got none".to_string())?
                    .scaled_by(0.8);
                let sub_cell_font =
                    fit_strings_to_size(&possible_strings, &sub_cell_size, &font_family)?
                        .ok_or("expected a font but got none".to_string())?
                        .scaled_by(0.8);

                Ok(Rc::new(DependentState {
                    puzzle_bounds,
                    cell_size,
                    sub_cell_size,
                    buttons_bounds,
                    cell_font,
                    sub_cell_font,
                }))
            })
            .clone()
    }

    fn cell_bounds(&mut self, p: Point) -> Result<Rectangle> {
        //TODO cache
        let ds = self.refresh_dependent_state()?;
        Ok(Rectangle::from_origin_size(
            lib::graphics::Point {
                x: ds.puzzle_bounds.origin().x + (p.column.0 as f64) * ds.cell_size.width(),
                y: ds.puzzle_bounds.origin().y + (p.row.0 as f64) * ds.cell_size.height(),
            },
            ds.cell_size,
        ))
    }

    fn sub_cell_bounds(&mut self, p: Point, x_sub: i8, y_sub: i8) -> Result<Rectangle> {
        //TODO cache
        let cell_bounds = self.cell_bounds(p)?;
        let ds = self.refresh_dependent_state()?;
        Ok(Rectangle::from_origin_size(
            lib::graphics::Point {
                x: cell_bounds.origin().x + (x_sub as f64) * ds.sub_cell_size.width(),
                y: cell_bounds.origin().y + (y_sub as f64) * ds.sub_cell_size.height(),
            },
            ds.sub_cell_size,
        ))
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

fn add_rect_to_context(context: &CanvasRenderingContext2d, r: &Rectangle) {
    context.rect(
        r.origin().x,
        r.origin().y,
        r.size().width(),
        r.size().height(),
    );
}
