use sdl2::keyboard::Keycode;

pub struct Keyboard {
    keys: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard { keys: [false; 16] }
    }

    pub fn pressed(&self, key: usize) -> bool {
        self.keys[key]
    }

    pub fn press(&mut self, key: Keycode, state: bool) {
        let index = self.key_to_index(key);
        if index <= 0xf {
            println!("Key changed: {} - {}", index, state);
            self.keys[index] = state;
        }
    }

    /**
     * Maps the following keyboard configuration
     *  *---------------*    *---------------*
     *  | 1 | 2 | 3 | 4 |    | 1 | 2 | 3 | C |
     *  | Q | W | E | R |    | 4 | 5 | 6 | D |
     *  | A | S | D | F | -> | 7 | 8 | 9 | E |
     *  | Z | X | C | V |    | A | 0 | B | F |
     *  *---------------*    *---------------*
     */
    fn key_to_index(&self, key: Keycode) -> usize {
        match key {
            Keycode::Num1 => 0x1,
            Keycode::Num2 => 0x2,
            Keycode::Num3 => 0x3,
            Keycode::Num4 => 0xc,
            Keycode::Q => 0x4,
            Keycode::W => 0x5,
            Keycode::E => 0x6,
            Keycode::R => 0xd,
            Keycode::A => 0x7,
            Keycode::S => 0x8,
            Keycode::D => 0x9,
            Keycode::F => 0xe,
            Keycode::Z => 0xa,
            Keycode::X => 0x0,
            Keycode::C => 0xb,
            Keycode::V => 0xf,
            _ => 99,
        }
    }
}
