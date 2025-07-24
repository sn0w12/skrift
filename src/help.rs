use crate::config::{Config, Binding};

pub fn print_help() {
    println!("Skrift - Minimal FLTK Text Editor");
    println!();
    println!("Usage:");
    println!("  skript [file]           Open file for editing (default: out.txt)");
    println!("  skript -h | --help      Show this help message");
    println!();
    println!("Shortcuts:");
    let cfg = Config::load();
    for binding in [
        Binding::Save,
        Binding::Quit,
        Binding::Reload,
        Binding::MoveLineUp,
        Binding::MoveLineDown,
    ] {
        if let Some(shortcut) = cfg.bindings.get(&binding) {
            println!("  {:<14} {}", format!("{:?}:", binding), shortcut);
        }
    }
    println!();
    println!("Config:");
    println!("  ~/.config/skrift/config.skrift");
}
