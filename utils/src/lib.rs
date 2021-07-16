
pub fn pretty_fs_size(size: &usize) -> String {
    let mut divisions: usize = 0;
    let mut resulting_size: f32 = size.clone() as f32;

    // Divide until we're below 1024 of whatever unit
    while resulting_size > 1024.0 {
        resulting_size /= 1024.0;
        divisions += 1;
    }

    // Then use the number of times we divided to infer the unit prefix
    match divisions {
        0 => format!("{}B", size),
        1 => format!("{0:.2}KiB", resulting_size),
        2 => format!("{0:.2}MiB", resulting_size),
        3 => format!("{0:.2}GiB", resulting_size),
        4 => format!("{0:.2}TiB", resulting_size),
        5 => format!("{0:.2}PiB", resulting_size),
        6 => format!("{0:.2}EiB", resulting_size),
        _ => format!("{} bytes", size), // I'm sure exbibytes is future proof enough
    }
}
