use rillrate::basis::*;

pub fn add() {
    let mut tab = Layout::new(["Prog Layout", "First Tab"]);
    tab.set_container(Align {
        alignment: Alignment::TOP_CENTER,
        child: Text {
            text: "Text".into(),
            align: TextAlign::Center,
        }
        .boxed(),
    });
    tab.register();
}
