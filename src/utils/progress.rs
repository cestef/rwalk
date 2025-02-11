const BRAILLE_CHARS: [char; 9] = ['⠀', '⠁', '⠃', '⠇', '⡇', '⡏', '⡟', '⡿', '⣿'];

pub fn get_braille_char(current: usize, total: usize) -> char {
    let index = (current as f64 / total as f64 * BRAILLE_CHARS.len() as f64).floor() as usize;
    let index = index.min(BRAILLE_CHARS.len() - 1);
    BRAILLE_CHARS[index]
}
