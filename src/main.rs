mod panel;

fn main() {
    let panel = panel::get_panel();

    if let Some(panel) = panel {
        panel::run(&panel);
    } else {
        println!("No panel found!");
        std::process::exit(1);
    }
}