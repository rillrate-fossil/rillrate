use yew::services::render::{RenderService, RenderTask};
use yew::Callback;

pub struct Reframer {
    callback: Callback<f64>,
    task: Option<RenderTask>,
}

impl Reframer {
    pub fn new(callback: Callback<f64>) -> Self {
        Self {
            callback,
            task: None,
        }
    }

    pub fn request_next_frame(&mut self) {
        let task = RenderService::request_animation_frame(self.callback.clone());
        self.task = Some(task);
    }

    pub fn interrupt(&mut self) {
        self.task.take();
    }
}
