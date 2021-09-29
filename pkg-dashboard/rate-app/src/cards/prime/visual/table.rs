use crate::blocks;
use rate_ui::utils;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::Path;
use rrpack_prime::visual::table::{Col, ColRecord, Row, RowRecord, TableEvent, TableState};
use std::collections::{BTreeMap, HashMap};
use yew::{html, Html, NodeRef};

pub type TableCard = WidgetRuntime<TableCardWidget>;

static EMPTY_CELL: String = String::new();

#[derive(Default)]
pub struct TableCardWidget {
    node_refs: HashMap<Row, HashMap<Col, NodeRef>>,
}

impl Widget for TableCardWidget {
    type Event = ();
    type Tag = Option<Path>;
    type Properties = SingleFlowProps;
    type Meta = SingleFlowMeta<Self>;

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.on_props(ctx);
    }

    fn on_props(&mut self, ctx: &mut Context<Self>) {
        let path = ctx.properties().path.clone().of_server();
        ctx.rewire(path);
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let body = {
            if let Some(state) = ctx.meta().state() {
                let cols = &state.spec.columns;
                html! {
                    <div class="flex-grow-1 overflow-auto">
                        <table class="table table-hover">
                            <thead>
                                <tr>
                                    { for cols.iter().map(|col| self.render_col(col)) }
                                </tr>
                            </thead>
                            <tbody>
                                { for state.rows.iter().map(|row| self.render_row(row, cols)) }
                            </tbody>
                        </table>
                    </div>
                }
            } else {
                blocks::spinner("Connecting...")
            }
        };
        html! {
            <div yew=module_path!() class="overflow-auto pe-3" style="height: 100%; width: 100%;">
                { body }
            </div>
        }
    }
}

impl TableCardWidget {
    fn render_col(&self, (_col_id, col): (&Col, &ColRecord)) -> Html {
        html! {
            <th class="col-1" scope="col">{ &col.title }</th>
        }
    }

    fn render_row(
        &self,
        (row_id, row): (&Row, &RowRecord),
        columns: &BTreeMap<Col, ColRecord>,
    ) -> Html {
        let cols = &row.cols;
        html! {
            <tr>
                { for columns.keys().map(|col_id| self.render_cell(row_id, col_id, cols.get(col_id))) }
            </tr>
        }
    }

    fn render_cell(&self, row: &Row, col: &Col, cell: Option<&String>) -> Html {
        let node_ref = self
            .node_refs
            .get(row)
            .and_then(|cols| cols.get(col))
            .cloned()
            .unwrap_or_else(|| {
                log::error!(
                    "NodeRef not exists for the table cell R-{:?} C-{:?}",
                    row,
                    col
                );
                NodeRef::default()
            });
        html! {
            <td ref=node_ref>{ cell.unwrap_or(&EMPTY_CELL) }</td>
        }
    }
}

/*
impl WiredWidget<SingleFlowMeta<Self>> for TableCardWidget {
    type Flow = TableState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        log::error!("TABLE: {:?}", ctx.meta().state());
        ctx.redraw();
    }
}
*/

impl WiredWidget<SingleFlowMeta<Self>> for TableCardWidget {
    type Flow = TableState;

    fn state_changed(&mut self, reloaded: bool, ctx: &mut Context<Self>) {
        if reloaded {
            self.node_refs.clear();
            if let Some(state) = ctx.meta().state() {
                let cols = &state.spec.columns;
                for row in state.rows.keys() {
                    let row = self.node_refs.entry(*row).or_default();
                    for (col, _) in cols.iter() {
                        row.entry(*col).or_default();
                    }
                }
            }
            ctx.redraw();
        }
    }

    fn state_update(
        &mut self,
        _tag: &Path,
        event: &<Self::Flow as Flow>::Event,
        reloaded: &mut bool,
        _ctx: &mut Context<Self>,
    ) {
        match &event {
            TableEvent::SetCell { row, col, value } => {
                let node = self
                    .node_refs
                    .get_mut(row)
                    .and_then(|cols| cols.get_mut(col));
                if let Some(node_ref) = node {
                    utils::set_node(node_ref, value);
                } else {
                    *reloaded = true;
                }
            }
            TableEvent::AddRow { .. } | TableEvent::DelRow { .. } => {
                *reloaded = true;
            }
        }
    }
}
