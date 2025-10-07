pub fn color_map(id: u64) -> colored::Color {
    match id % 6 {
        0 => colored::Color::Blue,
        1 => colored::Color::Green,
        2 => colored::Color::Yellow,
        3 => colored::Color::Magenta,
        4 => colored::Color::Cyan,
        _ => colored::Color::White,
    }
}

#[macro_export]
macro_rules! color_id {
    ([$id:expr]) => {
        {
            use colored::*;
            format!("[{}]", $id).color($crate::logger::color_map($id))
        }
    };
    ($id:expr => $($arg:tt)*) => {
        {
            use colored::*;
            format!("{}", format!($($arg)*).color($crate::logger::color_map($id)))
        }
    };
}

#[macro_export]
macro_rules! record {
    ([$id:expr], $($arg:tt)*) => {
        println!("{} {} {}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string().dimmed(), $crate::color_id!([$id]), format!($($arg)*))
    };
    ($($arg:tt)*) => {
        println!("{} {}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string().dimmed(), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ([$id:expr], $($arg:tt)*) => {
        {
            use colored::*;
            eprintln!("{} {} {}", $crate::color_id!([$id]), "warning:".yellow().bold(), format!($($arg)*))
        }
    };
    ($($arg:tt)*) => {
        {
            use colored::*;
            eprintln!("{} {}", "warning:".yellow().bold(), format!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! error {
    ([$id:expr], $($arg:tt)*) => {
        {
            use colored::*;
            eprintln!("{} {} {}", $crate::color_id!([$id]), "error:".red().bold(), format!($($arg)*))
        }
    };
    ($($arg:tt)*) => {
        {
            use colored::*;
            eprintln!("{} {}", "error:".red().bold(), format!($($arg)*))
        }
    };
}
