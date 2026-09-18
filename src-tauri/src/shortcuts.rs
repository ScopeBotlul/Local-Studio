use std::collections::{BTreeMap, BTreeSet};

pub const COMMANDS: &[(&str, &str)] = &[
    ("projectNew","Ctrl+N"),("projectOpen","Ctrl+O"),("projectSaveAs","Ctrl+Shift+S"),("projectClose","Ctrl+W"),
    ("undo", "Ctrl+Z"), ("redo", "Ctrl+Y"),
    ("projectSave", "Ctrl+S"), ("generate", "Ctrl+Enter"), ("delete", "Delete"), ("rename", "F2"),
    ("selectAll", "Ctrl+A"), ("compare", "C"), ("imageFit", "F"),
    ("imageActual", "1"), ("imageReset", "0"),
    ("uiZoomIn", "Ctrl+Plus"), ("uiZoomOut", "Ctrl+Minus"), ("uiZoomReset", "Ctrl+0"),
];
pub fn defaults() -> BTreeMap<String, String> {
    COMMANDS.iter().map(|(id, key)| (id.to_string(), key.to_string())).collect()
}
pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<BTreeMap<String, String>, D::Error> {
    let provided = <BTreeMap<String,String> as serde::Deserialize>::deserialize(deserializer)?;
    let mut values = defaults(); values.extend(provided); Ok(values)
}
pub fn valid_chord(value: &str) -> bool {
    if value.is_empty() { return true; } // Explicitly unbound.
    if value.len() > 48 { return false; }
    let mut parts = value.split('+').peekable();
    let mut modifiers = Vec::new();
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            let key = part;
            let valid = (key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric() && key == key.to_uppercase())
                || ["Plus", "Minus", "Enter", "Delete", "Insert", "Home", "End", "PageUp", "PageDown"].contains(&key)
                || (1..=12).any(|n| key == format!("F{n}"));
            return valid && !(key == "Plus" && modifiers.contains(&"Shift"));
        }
        if !["Ctrl", "Alt", "Shift"].contains(&part) || modifiers.contains(&part) { return false; }
        let order = |m: &str| match m { "Ctrl" => 0, "Alt" => 1, _ => 2 };
        if modifiers.last().is_some_and(|previous| order(previous) >= order(part)) { return false; }
        modifiers.push(part);
    }
    false
}
pub fn validate(bindings: &BTreeMap<String, String>) -> Result<(), String> {
    if bindings.len() != COMMANDS.len() || COMMANDS.iter().any(|(id, _)| !bindings.contains_key(*id)) {
        return Err("Shortcut commands are incomplete or unknown.".into());
    }
    let mut seen = BTreeSet::new();
    for chord in bindings.values() {
        if !valid_chord(chord) { return Err(format!("Invalid shortcut: {chord}")); }
        if chord.is_empty() { continue; }
        if ["Alt+F4", "Ctrl+C", "Ctrl+V", "Ctrl+X", "Ctrl+Shift+Z", "Ctrl+Alt+Delete"].contains(&chord.as_str()) {
            return Err(format!("Shortcut is reserved: {chord}"));
        }
        if !seen.insert(chord) { return Err(format!("Shortcut conflict: {chord}")); }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn checks_conflicts_reserved_keys_and_unbinding() {
        let mut map = defaults(); assert!(validate(&map).is_ok());
        map.insert("generate".into(), "Ctrl+A".into()); assert!(validate(&map).is_err());
        map.insert("generate".into(), "Ctrl+S".into()); assert!(validate(&map).is_err());
        map.insert("generate".into(), "Alt+Ctrl+G".into()); assert!(validate(&map).is_err());
        map.insert("generate".into(), "Ctrl+Shift+G".into()); assert!(validate(&map).is_ok());
        map.insert("generate".into(), "".into()); assert!(validate(&map).is_ok());
        map.remove("generate"); assert!(validate(&map).is_err());
    }
}
