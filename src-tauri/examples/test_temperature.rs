use sysinfo::Components;

fn main() {
    let components = Components::new_with_refreshed_list();

    println!("Found {} components:", components.len());

    for component in &components {
        println!("  Label: {}", component.label());
        println!("  Temperature: {}°C", component.temperature());
        println!("  Max: {}°C", component.max());
        println!("  Critical: {:?}°C", component.critical());
        println!("---");
    }

    if components.is_empty() {
        println!("No temperature sensors found!");
        println!("This is common on macOS - sysinfo cannot access SMC sensors.");
    }
}
