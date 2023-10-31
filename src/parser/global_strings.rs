static mut STATIC_STRINGS: Vec<String> = Vec::new(); // it's Vec
pub fn add_static_string(s: String) {
    unsafe {
        // SAFETY: no race conditions in this single threaded program.
        if !STATIC_STRINGS.contains(&s) {
            STATIC_STRINGS.push(s);
        }
    }
}

pub fn get_string_label(s: &String) -> String {
    unsafe {
        // SAFETY: no race conditions in this single threaded program.
        for (index, string) in STATIC_STRINGS.iter().enumerate() {
            if string == s {
                return format!(".globl_str_{}", index.to_string());
            }
        }
    }
    // the get string label should only be called if the caller is sure that
    // s is indeed a global string
    unreachable!()
}

pub fn generate_global_strings_asm() -> String {
    unsafe {
        let mut result = String::new();

        // SAFETY: no race conditions in this single threaded program.
        for (index, string_val) in STATIC_STRINGS.iter().enumerate() {
            let string_label = format!(".globl_str_{}", index.to_string());
            let mut string_bytes = String::new();
            for b in string_val.as_bytes() {
                string_bytes.push_str(&b.to_string());
                string_bytes.push_str(", ");
            }
            result.push_str(&format!("{} db {} 0 \n", string_label, string_bytes))
        }

        result
    }
}
