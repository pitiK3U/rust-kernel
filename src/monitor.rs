/// Module that implements basic VGA functions.
#[allow(non_snake_case)]
pub mod VGA {

    #[macro_export]
    macro_rules! print {
        ($($arg:tt)*) => ($crate::monitor::VGA::_print(format_args!($($arg)*)));
    }

    #[macro_export]
    macro_rules! println {
        () => ($crate::print!("\n"));
        ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
    }

    #[doc(hidden)]
    pub fn _print(args: fmt::Arguments) {
        use core::fmt::Write;
        BUFFER.lock().write_fmt(args).unwrap();
    }

    /// VGA display width in number of characters.
    const COLUMNS: usize = 80;
    /// VGA display height in number of characters.
    const ROWS: usize    = 25;
    const TAB_WIDTH: usize = 8;

    use crate::essentials::Mutex;
    use lazy_static::lazy_static;

    lazy_static!{
        /// Main singleton for writing to vga display.
        pub static ref BUFFER: Mutex<Monitor> = Mutex::new(Monitor {
            cursor: Cursor {
                x: 0,
                y: 0,
            },
            buffer: unsafe { &mut *(0xb8000 as *mut [[u16; COLUMNS];ROWS]) },
            background_color: Color::Black,
            foreground_color: Color::White,
        });
    }

    /// Enum used to represent background and foreground color
    /// in vga display.
    #[allow(dead_code)]
    #[repr(u8)]
    #[derive(Copy, Clone, Debug)]
    pub enum Color {
        Black       = 0,
        Blue        = 1,
        Green       = 2,
        Cyan        = 3,
        Red         = 4,
        Purple      = 5,
        Brown       = 6,
        Grey        = 7,
        DarkGrey    = 8,
        LightBlue   = 9,
        LightGreen  = 10,
        LightCyan   = 11,
        LightRed    = 12,
        LightPurple = 13,
        Yellow      = 14,
        White       = 15,
    }

    /// Constructs vga character (`u16`) from `byte`, foreground and background color.
    ///
    /// `byte` is stored in `0x00ff`
    ///
    /// `foreground_color` in `0x0f00`
    ///
    /// `background_color` in `0xf000`
    const fn vga_char(byte: u8, background_color: Color, foreground_color: Color) -> u16 {
        let attribute_byte: u8 = ( ( (background_color as u8) << 4)
                                   | ( (foreground_color as u8) & 0x0f) ) as u8;
        let character: u16 = ( (byte as u16) | ( (attribute_byte as u16) << 8) ) as u16;
        character
    }

    /// Returns whether `byte` is a printable character.
    const fn is_printable(byte: u8) -> bool {
        32 <= byte && byte <= 126
    }

    /// Inner display cursor representation.
    #[derive(Default, Debug)]
    struct Cursor {
        /// Current column position in given row.
        /// [`COLUMNS`] is the maximum value. When reached `x` is reset to `0`
        /// and `y` is increased.
        x: usize,
        /// Current row position in vga display.
        /// [`ROWS`] is the maximum value. When reached [`scroll()`](self::Monitor::scroll) function is called
        /// and cursor gets set to begginning of last row.
        y: usize,
    }

    impl Cursor {
        /// Moves cursor by one character.
        fn update_position(&self) {
            use crate::port::Port;

            let a = Port::new(0x3d4);
            let b = Port::new(0x3d5);
            let pos = self.to_array_index();

            a.write_byte(0x0f);
            b.write_byte((pos & 0xff) as u8);
            a.write_byte(0x0e);
            b.write_byte( ( (pos >> 8) & 0xff) as u8);
        }

        /// Returns cursor position as array offset.
        fn to_array_index(&self) -> usize{
            self.y * COLUMNS + self.x
        }
    }

    /// Structure used to write to vga display.
    #[derive(Debug)]
    pub struct Monitor {
        cursor: Cursor,
        /// The buffer of the vga device. All writes and reads should be `volatile`.
        buffer: &'static mut [[u16; COLUMNS as usize]; ROWS as usize],
        background_color: Color,
        foreground_color: Color,
    }

    use core::fmt;
    impl fmt::Write for Monitor {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.write_str(s);
            Ok(())
        }
    }

    impl Monitor {
        /// Sets backgound color for writes to vga.
        #[inline(always)]
        pub fn set_background_color(&mut self, color: &Color) {
            self.background_color = color.clone();
        }

        /// Sets foreground color for writes to vga.
        #[inline(always)]
        pub fn set_foreground_color(&mut self, color: &Color) {
            self.foreground_color = color.clone();
        }

        /// Fills the full screen (`ROWS * COLUMNS`) with blank (`' '`) character
        /// and sets cursor position to top left corner.
        pub fn clear(&mut self) {
            let cursor = &mut self.cursor;

            let blank_character = vga_char(' ' as u8,
                                           self.background_color,
                                           self.foreground_color);
            let mut row = 0;
            while row < ROWS {
                let mut column = 0;
                while column < COLUMNS {
                    self.buffer[row][column] = blank_character;
                    column += 1;
                }

                row += 1;
            }

            cursor.x = 0;
            cursor.y = 0;
        }

        /// Prints `byte` to current position on vga display
        /// includes special characters: `'\n'`, `'\r'`, `'\b'` and `'\t'`.
        pub fn write_byte(&mut self, byte: u8) {
            let cursor = &mut self.cursor;

            let mut character = vga_char(byte,
                                     self.background_color,
                                     self.foreground_color);

            // Handle special characters
            match byte {
                b'\n' => {
                    cursor.x = 0;
                    cursor.y += 1;
                    return;
                },
                b'\r' => {
                    cursor.x = 0;
                    return;
                },
                0x08 =>{
                    cursor.x -= 1;
                    return;
                },
                b'\t' => {
                    cursor.x = ( (cursor.x % TAB_WIDTH) + 1 ) * TAB_WIDTH;
                    return;
                },
                _ => {}
            }

            // MAYBE_REMOVE: it may be wanted to print not printable chars
            if !is_printable(byte) {
                // Print '■' instead of invalid byte
                character = vga_char(0xfe,
                                     self.background_color,
                                     self.foreground_color);
                //return;
            }

            self.buffer[cursor.y][cursor.x] = character;

            cursor.x += 1;
            if cursor.x >= COLUMNS {
                cursor.x = 0;
                cursor.y += 1;
            }
            cursor.update_position();

            if cursor.y >= ROWS {
                self.scroll();
            }
        }

        pub fn write_str(&mut self, string: &str) {
            string.bytes().for_each(|byte| self.write_byte(byte));
            /*
            let mut i: usize = 0;
            while i < string.len() {
                Self::write_byte(string[i]);

                i += 1;
            }
            */
        }

        /// Move all rows one row up. First row gets lost
        /// and last row gets filled with spaces.
        fn scroll(&mut self) {
            let mut i = 0;
            while i < ROWS - 1 {
                let mut column = 0;
                while column < COLUMNS {
                    // move every row one row up
                    // Since vga display is one array, COLUMNS == one row
                    self.buffer[i][column] = self.buffer[i + 1][column];
                    column += 1;
                }
                i += 1;
            }

            let blank_character: u16 = vga_char(' ' as u8,
                                                self.background_color,
                                                self.foreground_color);
            i = 0;
            while i < COLUMNS {
                self.buffer[ROWS - 1][i] = blank_character;
                i += 1;
            }
        }
    }
}
