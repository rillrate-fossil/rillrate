use meio::prelude::Actor;

pub struct RillSupervisor {}

impl Actor for RillSupervisor {
    type GroupBy = ();
}
