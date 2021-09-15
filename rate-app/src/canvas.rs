use anyhow::Error;
use derive_more::{Deref, DerefMut};
// TODO: Remove it or use
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use rate_ui::packages::or_fail::{Fail, Fasten};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d as Context2d, HtmlCanvasElement};
use yew::NodeRef;

const SCALE: f64 = 2.0;

pub struct SmartCanvas {
    canvas_ref: NodeRef,
    canvas: Option<HtmlCanvasElement>,
    ctx2d: Option<Context2d>,
    scale: f64,
    real_width: f64,
    real_height: f64,
    was_width: f64,
    was_height: f64,
}

impl Default for SmartCanvas {
    fn default() -> Self {
        Self {
            canvas_ref: NodeRef::default(),
            canvas: None,
            ctx2d: None,
            scale: SCALE,
            real_width: 0.0,
            real_height: 0.0,
            was_width: 0.0,
            was_height: 0.0,
        }
    }
}

impl SmartCanvas {
    pub fn node_ref(&self) -> &NodeRef {
        &self.canvas_ref
    }

    pub fn canvas(&self) -> Result<&HtmlCanvasElement, Error> {
        self.canvas.as_ref().ok_or_else(|| Error::msg("no canvas"))
    }

    pub fn bind(&mut self) -> Result<(), Error> {
        let canvas = self
            .canvas_ref
            .cast::<HtmlCanvasElement>()
            .or_fail("can't cast canvas")?;
        let ctx2d: Context2d = canvas
            .get_context("2d")
            .fasten()?
            .or_fail("no canvas context")?
            .dyn_into()
            .fasten()?;
        self.canvas = Some(canvas);
        self.ctx2d = Some(ctx2d);
        Ok(())
    }

    pub fn resize(&mut self) -> Result<(), Error> {
        let canvas = self
            .canvas
            .as_ref()
            .ok_or_else(|| Error::msg("Canvas 2D is not available!"))?;
        let rect = canvas.get_bounding_client_rect();
        self.scale = 2.0;
        self.real_height = rect.height();
        if self.was_height != self.real_height {
            let height = self.real_height * self.scale;
            canvas.set_height(height as u32);
            self.was_height = self.real_height;
        }
        self.real_width = rect.width();
        if self.was_width != self.real_width {
            let width = self.real_width * self.scale;
            canvas.set_width(width as u32);
            self.was_width = self.real_width;
        }
        /*
        log::info!("RATIO: {}", web_sys::window().unwrap().device_pixel_ratio());
        self.scale = web_sys::window()
            .as_ref()
            .map(Window::device_pixel_ratio)
            .unwrap_or(SCALE);
        self.scale = 2.0;
        */
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        let ctx = self
            .ctx2d
            .as_ref()
            .ok_or_else(|| Error::msg("Canvas 2D Context not initialized!"))?;

        /*
        ctx.set_transform(self.scale, 0.0, 0.0, self.scale, 0.0, 0.0)
            .map_err(|_| {
                Error::msg("Can't set transformation parameter to the Canvas 2D Context!")
            })?;
        */

        ctx.clear_rect(
            0.0,
            0.0,
            self.real_width * self.scale,
            self.real_height * self.scale,
        );
        Ok(())
    }
}

#[derive(Deref, DerefMut, Default)]
pub struct DrawCanvas {
    canvas: SmartCanvas,
}

impl DrawCanvas {
    pub fn draw_charts(
        &mut self,
        secs: i64,
        mut from_color: usize,
        min: f32,
        max: f32,
        x_formatter: &dyn Fn(&i64) -> String,
        y_formatter: &dyn Fn(&f32) -> String,
        data: &[Vec<(i64, f32)>],
    ) -> Result<(), Error> {
        from_color += 5;

        let canvas = self.canvas.canvas()?.clone();

        let root_area = CanvasBackend::with_canvas_object(canvas)
            .ok_or_else(|| Error::msg("no canvas backend created"))?
            .into_drawing_area();

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin(60)
            //.caption("Scatter Demo", ("Jost", 40))
            .build_cartesian_2d((-secs * 1_000)..0, min..max)?;

        ctx.configure_mesh()
            //.light_line_style(&WHITE)
            .light_line_style(&RGBColor(0xF8, 0xF9, 0xFA))
            .label_style(("Jost", 26))
            .x_label_formatter(x_formatter)
            .y_label_formatter(y_formatter)
            .draw()?;

        let single = data.len() == 1;

        for (col, line) in data.into_iter().enumerate() {
            let area_color;
            let line_color;
            if single {
                area_color = RGBColor(0xD2, 0x09, 0x09).mix(0.2).to_rgba();
                line_color = RGBColor(0x42, 0x11, 0xCC).mix(1.0).to_rgba();
            } else {
                line_color = Palette99::pick(from_color + col).to_rgba();
                area_color = line_color.mix(0.2).to_rgba();
            }
            let line = line.iter().cloned();
            let series = AreaSeries::new(line, 0.0, &area_color).border_style(&line_color);
            ctx.draw_series(series)?;
        }

        Ok(())
    }
}

pub fn sustain<Y: Copy>(mut iter: impl Iterator<Item = (i64, Y)>, last_x: i64) -> Vec<(i64, Y)> {
    let mut result = Vec::new();
    if let Some((mut prev_x, mut prev_y)) = iter.next() {
        result.push((prev_x, prev_y));
        for (next_x, next_y) in iter {
            let diff = next_x - prev_x;
            let shift = (diff as f32 * 0.1) as i64;
            result.push((next_x - shift, prev_y));
            result.push((next_x, next_y));
            prev_x = next_x;
            prev_y = next_y;
        }
        result.push((last_x, prev_y));
    }
    result
}

/*
pub fn sustain_soft<Y: Copy>(
    mut iter: impl Iterator<Item = (i64, Y)>,
    last: Option<i64>,
) -> Vec<(i64, Y)> {
    let mut result = Vec::new();
    if let Some((mut prev_x, mut prev_y)) = iter.next() {
        result.push((prev_x, prev_y));
        for (next_x, next_y) in iter {
            let diff = ((next_x - prev_x) as f32 * 0.2) as i64;
            result.push((next_x - diff, prev_y));
            result.push((next_x, next_y));
            prev_x = next_x;
            prev_y = next_y;
        }
        if let Some(last_x) = last {
            result.push((last_x, prev_y));
        }
    }
    result
}

pub fn sustain_sharp<X: Copy, Y: Copy>(
    mut iter: impl Iterator<Item = (X, Y)>,
    last: Option<X>,
) -> Vec<(X, Y)> {
    let mut result = Vec::new();
    if let Some((prev_x, mut prev_y)) = iter.next() {
        result.push((prev_x, prev_y));
        for (next_x, next_y) in iter {
            result.push((next_x, prev_y));
            result.push((next_x, next_y));
            //prev_x = next_x;
            prev_y = next_y;
        }
        if let Some(last_x) = last {
            result.push((last_x, prev_y));
        }
    }
    result
}
*/

pub fn formatter_plain(input: &f32) -> String {
    input.to_string()
}

/*
pub fn formatter_pct(input: &f32) -> String {
    format!("{:.0} %", input)
}
*/

/*
pub fn formatter_kib(input: &f32) -> String {
    format!("{:.0} KiB/s", input / 1_024.0)
}

pub fn formatter_gb(input: &f32) -> String {
    format!("{:.0} Gb", input / 1_000_000.0)
}
*/

pub fn formatter_sec(input: &i64) -> String {
    let input = input.abs();
    format!("{} sec", input / 1_000)
    /*
    if input % 60_000 == 0 {
        format!("{} min", input / 60_000)
    } else {
        format!("{} sec", input / 1_000)
    }
    */
}
