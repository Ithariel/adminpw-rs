use passwords::PasswordGenerator;
use std::io::Read;
use std::fmt;

mod froxlor;
mod plesk;
mod confixx;
mod liveconfig;

#[derive(Debug, Clone, Copy)]
pub enum Panel {
    Froxlor,
    Plesk,
    Confixx,
    LiveConfig
}

impl fmt::Display for Panel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Panel::Froxlor => write!(f, "Froxlor"),
            Panel::Plesk => write!(f, "Plesk"),
            Panel::Confixx => write!(f, "Confixx"),
            Panel::LiveConfig => write!(f, "LiveConfig"),
        }
    }
}

/// Prompt user to ENTER to continue
fn wait() {
    println!("Press ENTER to continue...");
    let buffer = &mut [0u8];
    std::io::stdin().read_exact(buffer).unwrap();
}

fn generate_password() -> String {
    let generator = PasswordGenerator {
        length: 24,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: false,
        strict: true
    };

    generator.generate_one().unwrap()
}

pub fn get_panel() -> Option<Panel> {
    if froxlor::is_froxlor() {
        Some(Panel::Froxlor)
    } else if plesk::is_plesk() {
        Some(Panel::Plesk)
    } else if confixx::is_confixx() {
        Some(Panel::Confixx)
    } else if liveconfig::is_liveconfig() {
        Some(Panel::LiveConfig)
    } else {
        None
    }
}

pub fn run(panel: &Panel) {
    match panel {
        Panel::Froxlor => return froxlor::run(&generate_password()),
        Panel::Plesk => return plesk::run(),
        Panel::Confixx => return confixx::run(&generate_password()),
        Panel::LiveConfig => return liveconfig::run(&generate_password()),
    }
}