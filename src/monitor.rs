/// Enables use of `static`s as global variables.
///
/// Usage:
/// ```
/// let mut inner = unsafe { SINGLETON.take() };
/// inner.method();
/// unsafe { SINGLETON.give(inner); }
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

    /// Returns taken value from singleton to the singleton
    /// for next use.
    pub fn give(&mut self, inner: T) {
        let _ = core::mem::replace(&mut self.inner, Some(inner));
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
            buffer: 0xb8000 as *mut u16,
            background_color: Color::Black,
            foreground_color: Color::White,
        }),
    };

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
        x: u8,
        /// Current row position in vga display.
        /// [`ROWS`] is the maximum value. When reached [`scroll()`](self::Monitor::scroll) function is called
        /// and cursor gets set to begginning of last row.
        y: u8,
    }

    impl Cursor {
        /// Moves cursor by one character.
        fn update_position(&mut self) {
        }

        /// Returns cursor position as array offset.
        fn to_array_index(&self) -> u16{
            (self.y as u16 * COLUMNS as u16 + self.x as u16) as u16
        }
    }

    /// Structure used to write to vga display.
    #[derive(Debug)]
    pub struct Monitor {
        cursor: Cursor,
        /// The buffer of the vga device. All writes and reads should be `volatile`.
        buffer: *mut u16,
        background_color: Color,
        foreground_color: Color,
    }

    impl Monitor {
        /// Fills the full screen (`ROWS * COLUMNS`) with blank (`' '`) character
        /// and sets cursor position to top left corner.
        pub fn clear() {
            let mut monitor: Self = unsafe { BUFFER.take() };
            let cursor = &mut monitor.cursor;

            let blank_character = vga_char(' ' as u8,
                                           monitor.background_color,
                                           monitor.foreground_color);
            let mut i = 0 as isize;
            const MAX_OFFSET: isize = (ROWS as isize * COLUMNS as isize) as isize;
            while i < MAX_OFFSET {
                unsafe {
                    monitor.buffer.offset(i).write_volatile(blank_character);
                }

                i += 1;
            }

            cursor.x = 0;
            cursor.y = 0;

            unsafe { BUFFER.give(monitor); }
        }

        /// Prints `byte` to current position on vga display
        /// includes special characters: `'\n'`, `'\r'`, `'\b'` and `'\t'`.
        pub fn write_byte(byte: u8) {
            let mut monitor: Self = unsafe { BUFFER.take() };
            let cursor = &mut monitor.cursor;

            let character = vga_char(byte,
                                     monitor.background_color,
                                     monitor.foreground_color);
            let position  = cursor.to_array_index() as isize;

            // Handle special characters
            match byte {
                b'\n' => {
                    cursor.x = 0;
                    cursor.y += 1;
                },
                b'\r' => {
                    cursor.x = 0;
                },
                0x08 =>{
                    cursor.x -= 1;
                },
                b'\t' => {
                    // TODO:
                },
                _ => {}
            }

            // MAYBE_REMOVE: it may be wanted to print not printable chars
            if !is_printable(byte) {
                unsafe { BUFFER.give(monitor); }
                return;
            }

            unsafe {
                monitor.buffer.offset(position).write_volatile(character);
            }

            cursor.x += 1;
            if cursor.x >= COLUMNS {
                cursor.x = 0;
                cursor.y += 1;
            }
            cursor.update_position();

            if cursor.y >= ROWS {
                monitor.scroll();
            }

            unsafe { BUFFER.give(monitor); }
        }

        pub fn write_str(string: &[u8]) {
            string.iter().for_each(|byte| Self::write_byte(*byte));
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
            const MAX_OFFSET: isize = ((ROWS as isize - 1) * COLUMNS as isize) as isize;
            let mut i = 0;
            while i < MAX_OFFSET {
                // move every row one row up
                // Since vga display is one array, COLUMNS == one row
                unsafe {
                    self.buffer.offset(i).write_volatile(
                        self.buffer.offset(i + COLUMNS as isize).read_volatile()
                    );
                }
                i += 1;
            }

            let blank_character: u16 = vga_char(' ' as u8,
                                                self.background_color,
                                                self.foreground_color);
            i = ((ROWS as isize - 1) * COLUMNS as isize) as isize;
            while i < MAX_OFFSET {
                unsafe {
                    self.buffer.offset(i).write_volatile(blank_character);
                }

                i += 1;
            }
        }
    }
}
