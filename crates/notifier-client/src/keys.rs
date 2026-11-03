use rand::seq::IndexedRandom;
use rdev::Key;

pub static KEYS: &[Key] = &[
    Key::KeyQ,
    //Key::KeyW,
    Key::KeyE,
    //Key::KeyR,
    Key::KeyT,
    Key::KeyY,
    //Key::KeyU,
    Key::KeyI,
    //Key::KeyO,
    Key::KeyP,
    //Key::KeyA,
    Key::KeyS,
    Key::KeyD,
    //Key::KeyF,
    Key::KeyG,
    Key::KeyH,
    Key::KeyJ,
    Key::KeyK,
    Key::KeyL,
    Key::KeyZ,
    Key::KeyX,
    Key::KeyC,
    Key::KeyV,
    Key::KeyB,
    Key::KeyN,
    Key::KeyM,
];

pub fn get_random_puppy_key() -> Key {
    [
        Key::KeyA,
        Key::KeyR,
        Key::KeyU,
        Key::KeyF,
        Key::KeyW,
        Key::KeyO,
    ]
    .choose(&mut rand::rng())
    .unwrap()
    .clone()
}

pub fn char_to_keys(character: char) -> &'static [Key] {
    match character {
        'q' => &[Key::KeyQ],
        'w' => &[Key::KeyW],
        'e' => &[Key::KeyE],
        'r' => &[Key::KeyR],
        't' => &[Key::KeyT],
        'y' => &[Key::KeyY],
        'u' => &[Key::KeyU],
        'i' => &[Key::KeyI],
        'o' => &[Key::KeyO],
        'p' => &[Key::KeyP],
        'a' => &[Key::KeyA],
        's' => &[Key::KeyS],
        'd' => &[Key::KeyD],
        'f' => &[Key::KeyF],
        'g' => &[Key::KeyG],
        'h' => &[Key::KeyH],
        'j' => &[Key::KeyJ],
        'k' => &[Key::KeyK],
        'l' => &[Key::KeyL],
        'z' => &[Key::KeyZ],
        'x' => &[Key::KeyX],
        'c' => &[Key::KeyC],
        'v' => &[Key::KeyV],
        'b' => &[Key::KeyB],
        'n' => &[Key::KeyN],
        'm' => &[Key::KeyM],
        ':' => &[Key::ShiftLeft, Key::SemiColon],
        _ => &[Key::Space],
    }
}
