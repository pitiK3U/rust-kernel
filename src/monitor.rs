/// Enables use of `static`s as global variables.
///
/// Usage:
/// ```
/// let mut inner = unsafe { SINGLETON.take() };
/// inner.method();
/// ```
pub struct Singleton<T> {
    pub inner: Option<T>,
}

impl<T> Singleton<T> {
    /// Takes ownership of singleton's inner value.
    /// If the inner value is already taken, program crashes.
    pub fn take(&mut self) -> T {
        let p = core::mem::replace(&mut self.inner, None);
        // TODO: better double take handling
        p.unwrap()
    }
}

/// Module that implements basic VGA functions.
pub mod VGA {
    use crate::monitor::Singleton;

    /// VGA display width in number of characters.
    const COLUMNS: u8 = 80;
    /// VGA display height in number of characters.
    const ROWS: u8    = 25;

    /// Main singleton for writing to vga display.
    pub static mut BUFFER: Singleton<Monitor> = Singleton::<Monitor> {
        inner: Some(Monitor {
            cursor: Cursor {
                x: 0,
                y: 0,
            },
        }),
    };

    /// Enum used to represent background and foreground color
    /// in vga display.
    #[repr(u8)]
    #[allow(dead_code)]
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
    #[derive(Copy, Clone, Default, Debug)]
    struct Cursor {
        /// Current column position in given row.
        /// [`COLUMNS`] is the maximum value. When reached `x` is reset to `0`
        /// and `y` is increased.
        x: u8,
        /// Current row position in vga display.
        /// [`ROWS`] is the maximum value. When reached [`scroll()`](self::Monitor::scroll) function is called
        /// and cursor gets set to begginning of last row.
        y: u8,
    }

    impl Cursor {
        /// Moves cursor by one character.
        fn update_position(&mut self) {
            self.x += 1;

            if self.x >= COLUMNS {
                self.x = 0;
                self.y += 1;

                if self.y >= ROWS {
                    // handle
                }
            }
        }

        /// Returns cursor position as array offset.
        fn to_pos(&self) -> u16{
            (self.y * COLUMNS + self.x) as u16
        }
    }

    /// Structure used to write to vga display.
    #[derive(Copy, Clone)]
    pub struct Monitor {
        cursor: Cursor,
    }

    impl Monitor {
        /// Fills the full screen (`ROWS * COLUMNS`) with blank (`' '`) character
        /// and sets cursor position to top left corner.
        pub fn clear(&mut self) {
            // FIXME: cheat to avoid static
            let vga_buffer = 0xb8000 as *mut u16;

            let blank_character = vga_char(' ' as u8, Color::Black, Color::White);
            let mut i = 0 as isize;
            const MAX_OFFSET: isize = (ROWS as isize * COLUMNS as isize) as isize;
            while i < MAX_OFFSET {
                unsafe {
                    vga_buffer.offset(i).write_volatile(blank_character);
                }

                i += 1;
            }

            self.cursor.x = 0;
            self.cursor.y = 0;
        }

        /// Prints `byte` to current position on vga display
        /// includes special characters: `'\n'`, `'\r'`, `'\b'` and `'\t'`.
        pub fn write_byte(&mut self, byte: u8) {
            // FIXME: cheat to avoid static
            let vga_buffer = 0xb8000 as *mut u16;

            let character  = vga_char(byte, Color::Black, Color::White);
            let position   = self.cursor.to_pos() as isize;

            // Handle special characters
            match byte {
                b'\n' => {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                },
                b'\r' => {
                    self.cursor.x = 0;
                },
                0x08 =>{
                    self.cursor.x -= 1;
                },
                b'\t' => {
                    // TODO:
                },
                _ => {}
            }

            // MAYBE_REMOVE: it may be wanted to print not printable chars
            if !is_printable(byte) {
                return;
            }

            unsafe {
                vga_buffer.offset(position).write_volatile(character);
            }
            self.cursor.update_position();

            if self.cursor.y >= ROWS {
                self.scroll();
            }
        }

        /// Move all rows one row up. First row gets lost
        /// and last row gets filled with spaces.
        fn scroll(&mut self) {
            // FIXME: cheat to avoid static
            let vga_buffer = 0xb8000 as *mut u16;

            const MAX_OFFSET: isize = ((ROWS as isize - 1) * COLUMNS as isize) as isize;
            let mut i = 0;
            while i < MAX_OFFSET {
                // move every row one row up
                // Since vga display is one array, COLUMNS == one row
                unsafe {
                    vga_buffer.offset(i).write_volatile(
                        vga_buffer.offset(i + COLUMNS as isize).read_volatile()
                    );
                }
                i += 1;
            }

            let blank_character: u16 = vga_char(' ' as u8, Color::Black, Color::White);
            i = ((ROWS as isize - 1) * COLUMNS as isize) as isize;
            while i < MAX_OFFSET {
                unsafe {
                    vga_buffer.offset(i).write_volatile(blank_character);
                }

                i += 1;
            }
        }
    }
}
