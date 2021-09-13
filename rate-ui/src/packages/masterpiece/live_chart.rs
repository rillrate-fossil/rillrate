use crate::packages::masterpiece::scale::{LinearScale, Range};
use crate::packages::or_fail::{Fail, Fasten};
use anyhow::Error;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d as Context2d, HtmlCanvasElement, Window};
use yew::NodeRef;

const SCALE: f64 = 2.0;

const RETAIN: f64 = 6_000.0;

const LINE_WIDTH: f64 = 1.0;

/// Pixels per second
const PPS: f64 = 50.0;

pub struct Point {
    pub value: f64,
    pub timestamp: f64,
}

pub struct LiveChart {
    last_timestamp: f64,
    canvas_ref: NodeRef,
    canvas: Option<HtmlCanvasElement>,
    ctx2d: Option<Context2d>,
    scale: f64,
    real_width: f64,
    real_height: f64,
    retain: f64,
}

impl Default for LiveChart {
    fn default() -> Self {
        Self {
            last_timestamp: 0.0,
            canvas_ref: NodeRef::default(),
            canvas: None,
            ctx2d: None,
            scale: SCALE,
            real_width: 0.0,
            real_height: 0.0,
            retain: RETAIN,
        }
    }
}

impl LiveChart {
    pub fn canvas_ref(&self) -> &NodeRef {
        &self.canvas_ref
    }

    pub fn link_to_canvas(&mut self) -> Result<(), Error> {
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
        // TODO: Do I have to `resize` here?!
        self.resize()
    }

    pub fn resize(&mut self) -> Result<(), Error> {
        let canvas = self
            .canvas
            .as_ref()
            .ok_or_else(|| Error::msg("Canvas 2D is not available!"))?;
        let rect = canvas.get_bounding_client_rect();
        self.real_width = rect.width();
        self.real_height = rect.height();
        self.retain = self.real_width / PPS * 1_000.0;
        self.scale = web_sys::window()
            .as_ref()
            .map(Window::device_pixel_ratio)
            .unwrap_or(SCALE);
        let width = self.real_width * self.scale;
        let height = self.real_height * self.scale;
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        Ok(())
    }

    pub fn render<I>(
        &mut self,
        timestamp: f64,
        points: I,
        mut y_domain: Range,
        _x_domain: Range,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = Point>,
    {
        self.last_timestamp = timestamp;
        let ctx = self
            .ctx2d
            .as_ref()
            .ok_or_else(|| Error::msg("Canvas 2D Context not initialized!"))?;

        ctx.set_transform(self.scale, 0.0, 0.0, self.scale, 0.0, 0.0)
            .map_err(|_| {
                Error::msg("Can't set transformation parameter to the Canvas 2D Context!")
            })?;
        let width = self.real_width;
        let height = self.real_height;

        // VALUE RANGE
        if y_domain.is_flat() {
            y_domain.spread(0.1);
        }
        let y_ticks = y_domain.ticks(5);
        let mut y_range = Range::new(height, 0.0);
        y_range.with_padding(LINE_WIDTH);
        let y_scale = LinearScale::new(y_domain, y_range);
        let translate_y = |value: f64| -> f64 { y_scale.rescale(value) };

        // TIMESTAMP RANGE
        let ts_domain = Range::new(timestamp - self.retain, timestamp);
        let ts_range = Range::new(0.0, width);
        let x_scale = LinearScale::new(ts_domain, ts_range.clone());
        let translate_x = |value: f64| -> f64 { x_scale.rescale(value) };

        ctx.clear_rect(0.0, 0.0, width, height);

        // DRAW TICKS
        ctx.set_stroke_style(&JsValue::from("#EEEEEE"));
        ctx.set_line_width(LINE_WIDTH);
        for y_tick in y_ticks {
            let y = translate_y(y_tick);
            ctx.begin_path();
            ctx.move_to(ts_range.min(), y);
            ctx.line_to(ts_range.max(), y);
            ctx.stroke();
        }

        // DRAW LINES
        ctx.set_fill_style(&JsValue::from("#EEEEEE"));
        ctx.set_stroke_style(&JsValue::from("#5d2f86"));
        ctx.set_line_width(LINE_WIDTH);

        let mut points_iter = points;
        if let Some(mut last_point) = points_iter.next() {
            ctx.move_to(ts_range.min(), translate_y(last_point.value));
            ctx.begin_path();
            while let Some(next_point) = points_iter.next() {
                ctx.line_to(
                    translate_x(next_point.timestamp),
                    translate_y(last_point.value),
                );
                ctx.line_to(
                    translate_x(next_point.timestamp),
                    translate_y(next_point.value),
                );
                last_point = next_point;
            }
            ctx.line_to(ts_range.max(), translate_y(last_point.value));
            ctx.stroke();
        }
        Ok(())
    }
}
