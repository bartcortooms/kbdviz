use crate::keyboard::XkbManager;
use std::collections::HashMap;

/// Represents a single compose sequence result
#[derive(Clone, Debug)]
pub struct ComposeEntry {
    pub character: String,
    pub key_sequence: String,
}

/// Index of base characters to their variants
pub struct ComposeIndex {
    // Maps base character (e.g., 'e') to all its variants
    index: HashMap<char, Vec<ComposeEntry>>,
}

impl ComposeIndex {
    /// Count of base characters with variants
    pub fn count(&self) -> usize {
        self.index.len()
    }

    /// Build the compose index from XKB manager
    pub fn build(_xkb: &XkbManager) -> Result<Self, String> {
        let mut index: HashMap<char, Vec<ComposeEntry>> = HashMap::new();

        // For now, manually define common compose sequences
        // TODO: Parse from /usr/share/X11/locale/*/Compose or use libxkbcommon compose API

        // Letter E variants
        add_entry(&mut index, 'e', "é", "AltGr+' e");
        add_entry(&mut index, 'e', "è", "AltGr+` e");
        add_entry(&mut index, 'e', "ë", "AltGr+\" e");
        add_entry(&mut index, 'e', "ê", "AltGr+^ e");
        add_entry(&mut index, 'e', "ẽ", "AltGr+~ e");
        add_entry(&mut index, 'e', "ē", "AltGr+- e");
        add_entry(&mut index, 'e', "ė", "AltGr+. e");
        add_entry(&mut index, 'e', "ę", "AltGr+; e");

        // Letter A variants
        add_entry(&mut index, 'a', "á", "AltGr+' a");
        add_entry(&mut index, 'a', "à", "AltGr+` a");
        add_entry(&mut index, 'a', "ä", "AltGr+\" a");
        add_entry(&mut index, 'a', "â", "AltGr+^ a");
        add_entry(&mut index, 'a', "ã", "AltGr+~ a");
        add_entry(&mut index, 'a', "ā", "AltGr+- a");
        add_entry(&mut index, 'a', "ą", "AltGr+; a");
        add_entry(&mut index, 'a', "å", "AltGr+o a");

        // Letter O variants
        add_entry(&mut index, 'o', "ó", "AltGr+' o");
        add_entry(&mut index, 'o', "ò", "AltGr+` o");
        add_entry(&mut index, 'o', "ö", "AltGr+\" o");
        add_entry(&mut index, 'o', "ô", "AltGr+^ o");
        add_entry(&mut index, 'o', "õ", "AltGr+~ o");
        add_entry(&mut index, 'o', "ō", "AltGr+- o");
        add_entry(&mut index, 'o', "ø", "AltGr+/ o");

        // Letter U variants
        add_entry(&mut index, 'u', "ú", "AltGr+' u");
        add_entry(&mut index, 'u', "ù", "AltGr+` u");
        add_entry(&mut index, 'u', "ü", "AltGr+\" u");
        add_entry(&mut index, 'u', "û", "AltGr+^ u");
        add_entry(&mut index, 'u', "ũ", "AltGr+~ u");
        add_entry(&mut index, 'u', "ū", "AltGr+- u");
        add_entry(&mut index, 'u', "ų", "AltGr+; u");

        // Letter I variants
        add_entry(&mut index, 'i', "í", "AltGr+' i");
        add_entry(&mut index, 'i', "ì", "AltGr+` i");
        add_entry(&mut index, 'i', "ï", "AltGr+\" i");
        add_entry(&mut index, 'i', "î", "AltGr+^ i");
        add_entry(&mut index, 'i', "ĩ", "AltGr+~ i");
        add_entry(&mut index, 'i', "ī", "AltGr+- i");
        add_entry(&mut index, 'i', "į", "AltGr+; i");

        // Letter N variants
        add_entry(&mut index, 'n', "ñ", "AltGr+~ n");
        add_entry(&mut index, 'n', "ń", "AltGr+' n");

        // Letter C variants
        add_entry(&mut index, 'c', "ç", "AltGr+, c");
        add_entry(&mut index, 'c', "ć", "AltGr+' c");
        add_entry(&mut index, 'c', "č", "AltGr+v c");

        // Letter S variants
        add_entry(&mut index, 's', "ś", "AltGr+' s");
        add_entry(&mut index, 's', "š", "AltGr+v s");
        add_entry(&mut index, 's', "ş", "AltGr+, s");
        add_entry(&mut index, 's', "ß", "AltGr+s");

        // Letter Y variants
        add_entry(&mut index, 'y', "ý", "AltGr+' y");
        add_entry(&mut index, 'y', "ÿ", "AltGr+\" y");

        // Letter Z variants
        add_entry(&mut index, 'z', "ź", "AltGr+' z");
        add_entry(&mut index, 'z', "ž", "AltGr+v z");
        add_entry(&mut index, 'z', "ż", "AltGr+. z");

        // Letter L variants
        add_entry(&mut index, 'l', "ł", "AltGr+/ l");

        // Special characters accessible via base characters
        add_entry(&mut index, 'a', "æ", "AltGr+z");
        add_entry(&mut index, 'o', "œ", "AltGr+x");

        // Currency symbols
        add_entry(&mut index, 'e', "€", "AltGr+5");
        add_entry(&mut index, 'c', "¢", "AltGr+c");
        add_entry(&mut index, 'l', "£", "AltGr+3");
        add_entry(&mut index, 'y', "¥", "AltGr+y");

        Ok(Self { index })
    }

    /// Find all character variants for a given base character
    pub fn find_variants(&self, input: &str) -> Vec<ComposeEntry> {
        // Get the first character from input
        let base_char = input.chars().next();

        if let Some(ch) = base_char {
            // Try both lowercase and uppercase
            let mut results = Vec::new();

            if let Some(variants) = self.index.get(&ch.to_lowercase().next().unwrap()) {
                results.extend(variants.clone());
            }

            if ch.is_uppercase() {
                if let Some(variants) = self.index.get(&ch.to_uppercase().next().unwrap()) {
                    results.extend(variants.clone());
                }
            }

            results
        } else {
            Vec::new()
        }
    }
}

/// Helper function to add an entry to the index
fn add_entry(
    index: &mut HashMap<char, Vec<ComposeEntry>>,
    base: char,
    character: &str,
    key_sequence: &str,
) {
    let entry = ComposeEntry {
        character: character.to_string(),
        key_sequence: key_sequence.to_string(),
    };

    index
        .entry(base)
        .or_insert_with(Vec::new)
        .push(entry);
}
