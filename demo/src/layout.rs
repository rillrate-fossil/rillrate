use rillrate::basis::*;

pub fn add() {
    let mut tab = Layout::new(["Prog Layout", "First Tab"]);
    tab.set_container(Align {
        alignment: Alignment::TOP_CENTER,
        child: LLabel {
            text: "Text".into(),
        }
        .into(),
    });
    tab.register();
}
