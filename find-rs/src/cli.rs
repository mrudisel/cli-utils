use std::{
    fmt,
    fs::canonicalize,
    env::current_dir,
    path::PathBuf,
};

use lazy_static::lazy_static;
use regex::{Regex, Captures};
use clap::{Arg, App, ArgMatches, Values};


use utils::pretty_fs_size;


static DEFAULT_FUZZY_THRESHOLD: isize = -100;

//
static SIZE_REGEX_STR: &str = r"^[+]?(?P<digits>\d+)[^a-z|A-Z]*(?P<unit>[kmgtKMGT]?[i]?[bB]?)?$";


pub enum FindOnly {
    Files,
    Directories,
}

pub enum OrderByProperty {
    Size,
    Filename,
}


pub struct Error {
    message: String,
}

impl Error {
    fn new(messg: &str) -> Error {
        Error {message: messg.into()}
    }

    fn from_string(messg: String) -> Error {
        Error {message: messg}
    }

    fn from<'a, T: fmt::Display>(obj: &T) -> Error {
        Error {message: obj.to_string()}
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}


pub struct FindArgs {
    pub root: PathBuf,
    pub regex_matchers: Option<Vec<Regex>>,
    pub fuzzy_matchers: Option<Vec<String>>,
    pub fuzzy_thresh: isize,
    pub min_size: Option<usize>,
    pub max_size: Option<usize>,
    pub worker_threads: usize,
    pub find_only: Option<FindOnly>,
    pub order_by: Option<OrderByProperty>,
    pub desc_order: bool,
    pub print_stats: bool,
    pub verbose: bool,
}

fn build_regexes<F>(values: Values, formatter: F) -> Vec<Regex>
where F: Fn(&str) -> String
{
    values.into_iter()
        .filter_map(|val| {
            let formatted = formatter(val);
            Regex::new(&formatted).ok()
        })
        .collect()
}

impl FindArgs {
    pub fn from_arg_matches(matches: &ArgMatches) -> Result<FindArgs, Error> {
        println!("patterns: {}", match matches.values_of("patterns") {
            Some(vals) => {
                let pats: Vec<&str> = vals.into_iter().collect();
                pats.join(", ")
            },
            None => String::from("No patterns")
        });

        println!("regexes: {}", match matches.values_of("regex") {
            Some(vals) => {
                let pats: Vec<&str> = vals.into_iter().collect();
                pats.join(", ")
            },
            None => String::from("No regexes")
        });

        println!("exts: {}", match matches.values_of("exts") {
            Some(vals) => {
                let pats: Vec<&str> = vals.into_iter().collect();
                pats.join(", ")
            },
            None => String::from("No exts")
        });


        /*
        let mut patterns = vec![""];
        if matches.is_present("videos") {
            patterns.append(&mut vec!["mp4", "mkv", "mpeg", ""]);
        }
        else if matches.is_present("images") {
            patterns.append(&mut vec!["png", "gif", "jpg", "jpeg", "avif"]);
        }
        else {

        }
        */

        let pattern_values = matches.values_of("patterns");
        let regex_values = matches.values_of("regex");
        let ext_values = matches.values_of("exts");

        let mut regexes: Option<Vec<Regex>> = None;
        if let Some(patterns) = pattern_values {
            regexes = Some(build_regexes(patterns, |val| val.trim().replace("*", ".*")));
        }
        else if let Some(regex_patterns) = regex_values {
            regexes = Some(build_regexes(regex_patterns, |val| val.trim().to_string()));

        }
        else if let Some(ext_patterns) = ext_values {
            regexes = Some(build_regexes(ext_patterns, |ext| {
                format!(".*{}$", ext.trim().replace(".", ""))
            }));
        }

        let fuzzy_patterns = matches.values_of("fuzzy").map(|vals| {
            vals.into_iter()
                .map(|val| val.to_string())
                .collect()
        });


        Ok(FindArgs {
            root: get_root_path(matches.value_of("root"))?,
            regex_matchers: regexes,
            fuzzy_matchers: fuzzy_patterns,
            fuzzy_thresh: parse_fuzzy_thresh(matches.value_of("fuzzy-thresh"))?,
            min_size: try_parse_size(matches.value_of("min-size"))?,
            max_size:  try_parse_size(matches.value_of("max-size"))?,
            worker_threads: get_thread_count(matches.value_of("workers"))?,
            find_only: matches.is_present("exts")
                .then(|| FindOnly::Files) // Infered if exts was passed in
                .or_else(|| get_find_only_type(matches.value_of("type"))), // otherwise, try and match the input if it exists.
            order_by: get_order_by_prop(matches.value_of("order-by")),
            desc_order: matches.is_present("desc"),
            print_stats: matches.is_present("stats") || matches.is_present("verbose"),
            verbose: matches.is_present("verbose"),
        })

    }
}

impl fmt::Display for FindArgs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The root directory (try and get the absolute path, for clarity)
        match canonicalize(self.root.as_path()) {
            Ok(abs_path) => write!(f, "Root - {}\n", abs_path.display())?,
            _ => write!(f, "Root - {}\n", self.root.display())?,
        }

        match self.find_only {
            Some(FindOnly::Files) => write!(f, "Finding files only\n")?,
            Some(FindOnly::Directories) => write!(f, "Finding directories only\n")?,
            _ => () // No need to specify we're looking for everything
        }

        match self.worker_threads {
            0 | 1 => write!(f, "Using only the main thread\n")?,
            _ => write!(f, "Using {} Worker threads\n", self.worker_threads)?,
        }

        // log whichever method we're using to match
        if let Some(match_patterns) = &self.regex_matchers {
            let match_strs: Vec<String> = match_patterns.iter()
                .map(|reg| reg.to_string())
                .collect();

            match match_patterns.len() {
                0 => (),
                1 => write!(f, "Regex pattern - {}\n", match_strs[0])?,
                _ => write!(f, "Regex patterns - {}\n", match_strs.join(", "))?,
            }
        }
        else if let Some(fuzzy_patterns) = &self.fuzzy_matchers {
            write!(f, "Fuzzy matching with a score threshold of {}\n", self.fuzzy_thresh)?;

            match fuzzy_patterns.len() {
                0 => (),
                1 => write!(f, "Fuzzy match pattern - {}\n", fuzzy_patterns[0])?,
                _ => write!(f, "Fuzzy match patterns - {}\n", fuzzy_patterns.join(", "))?,
            }
        }
        else {
            write!(f, "Matching all files")?;
        }

        // Print min/max sizes if specified
        if let Some(min_size) = &self.min_size {
            write!(f, "Min file size - {}\n", pretty_fs_size(min_size))?;
        }

        if let Some(max_size) = &self.max_size {
            write!(f, "Max file size - {}\n", pretty_fs_size(max_size))?;
        }

        if let Some(order_by) = &self.order_by {
            let direction = if self.desc_order {"decending"} else {"ascending"};
            match order_by {
                OrderByProperty::Size => write!(f, "Ordering by {} file size", direction)?,
                OrderByProperty::Filename => write!(f, "Ordering by {} filename", direction)?,
            }
        }

        write!(f, "\n")
    }
}

fn parse_fuzzy_thresh(thresh_arg: Option<&str>) -> Result<isize, Error> {
    thresh_arg.map(|thresh_str| isize::from_str_radix(thresh_str, 10)) // tries parsing the arg
        .unwrap_or(Ok(DEFAULT_FUZZY_THRESHOLD)) // unwraps, using the default as a fallback
        .map_err(|err| Error::from(&err)) // Maps the error to a string
}

fn get_root_path(path_arg: Option<&str>) -> Result<PathBuf, Error> {
    if let Some(root_str) = path_arg {
        let path_buf = PathBuf::from(root_str);
        return match path_buf.exists() {
            true => Ok(path_buf),
            false => Err(Error::new("Root path specified does not exist")),
        }
    }

    match current_dir() {
        Ok(buf) => Ok(buf),
        Err(err) => Err(Error::from(&err)),
    }
}

fn get_thread_count(thread_arg: Option<&str>) -> Result<usize, Error> {
    if let Some(thread_str) = thread_arg {
        return usize::from_str_radix(thread_str, 10)
            .map_err(|err| Error::from(&err));
    }

    Ok(num_cpus::get() - 1)
}

fn try_parse_size(size_arg: Option<&str>) -> Result<Option<usize>, Error> {
    let size_str = match size_arg {
        Some(size_str) => size_str.trim(),
        None => return Ok(None),
    };

    // The below regex requires at least 1 number, and 1 character for the unit.
    if size_str.len() < 2 {
        return Ok(None);
    }

    lazy_static! {
        static ref SIZE_REGEX: Regex = match Regex::new(SIZE_REGEX_STR) {
            Ok(regex) => regex,
            Err(err) => panic!("Regex compile error: {}", err),
        };
    }

    let captures: Captures = SIZE_REGEX.captures(size_str)
        .ok_or_else(|| Error::new("Regex did not match"))?;

    let mut digits: usize = match captures.name("digits").map(|mat| mat.as_str()) {
        Some(digits_str) => usize::from_str_radix(digits_str, 10).map_err(|err| Error::from(&err))?,
        _ => return Err(Error::from_string(format!("Could not extract digits in size {}", size_str))),
    };

    if let Some(units) = captures.name("unit").map(|mat| mat.as_str()) {
        let mult = get_size_multiplier_from_units(units);

        digits *= mult;
    }

    Ok(Some(digits))
}

fn get_size_multiplier_from_units(units: &str) -> usize {
    let mut mult: usize = match units.to_lowercase().chars().nth(0) {
        Some('k') => 1024,
        Some('m') => 1024 * 1024,
        Some('g') => 1024 * 1024 * 1024,
        Some('t') => 4,
        _ => return 1,
    };

    if let Some(last_char) = units.chars().last() {
        // If we got a lower case b, assume bits not bytes, so scale by 8.
        if last_char == 'b' { mult /= 8 };
    }

    mult
}

fn get_find_only_type(find_arg: Option<&str>) -> Option<FindOnly> {
    let find_only_str = find_arg.unwrap_or("").trim().to_lowercase();

    match find_only_str.as_str() {
       "f" | "file" | "files" => Some(FindOnly::Files),
       "d" | "dir" | "dirs" | "directory" | "directories" => Some(FindOnly::Directories),
       _ => None,
    }
}

fn get_order_by_prop(order_arg: Option<&str>) -> Option<OrderByProperty> {
    let order_str = order_arg.unwrap_or("").trim().to_lowercase();

    match order_str.to_lowercase().as_str() {
        "s" | "sz" | "size" => Some(OrderByProperty::Size),
        "f" | "fn" | "filename" => Some(OrderByProperty::Filename),
        _ => None,
    }
}


pub fn parse_cli() -> Result<FindArgs, Error> {
    let root_arg = Arg::with_name("root")
        .help("The root directory to begin searching from. If not specified, will default to $PWD")
        .required(true);

    let pattern_arg = Arg::with_name("patterns")
        .help("The pattern(s) to match file/directory names against.")
        .multiple(true);

    let exts_arg = Arg::with_name("exts")
        .help("File extensions to search for. Implies '--type file'")
        .long("exts")
        .short("e")
        .conflicts_with_all(&["regex", "fuzzy"]);

    let regex_arg = Arg::with_name("regex")
        .help("Regex patten to match")
        .long("regex")
        .short("r")
        .conflicts_with_all(&["exts", "fuzzy"]);

    let fuzzy_arg = Arg::with_name("fuzzy")
        .help("Attempts to fuzzy match the file or directory name")
        .long("fuzzy")
        .short("f")
        .takes_value(true)
        .conflicts_with_all(&["exts", "regex"]);

    let fuzzy_thresh_arg = Arg::with_name("fuzzy-thresh")
        .help("Set a fuzzy match scoring threshold to ignore beyond. Defaults to ") // TODO
        .short("s")
        .long("fuzzy-score")
        .required(false)
        .takes_value(true);

    let min_size_arg = Arg::with_name("min-size")
        .help("Minimum file size threshold. Can be a number in bytes, or a human readable string (ex. '4MiB', '4k', '5MB', etc)")
        .long("min-size")
        .takes_value(true)
        .required(false);

    let max_size_arg = Arg::with_name("max-size")
        .help("Maximum file size threshold. Can be a number in bytes, or a human readable string (ex. '4MiB', '4k', '5MB', etc)")
        .long("max-size")
        .takes_value(true)
        .required(false);

    let workers_arg = Arg::with_name("workers")
        .help("Maximum number of worker threads to use. Defaults to 'num_cpus - 1'")
        .short("w")
        .long("max-workers")
        .takes_value(true);

    let type_arg = Arg::with_name("type")
        .help("Whether to search only for files, or directories.")
        .long("type")
        .short("t")
        .takes_value(true)
        .possible_values(&["f", "d", "file", "dir", "files", "dirs", "directory", "directories"])
        .required(false);

    let order_by_arg = Arg::with_name("order-by")
        .help("Instead of printing as we find them, we'll collect all results and order by the specified metric")
        .long("order-by")
        .short("o")
        .required(false)
        .takes_value(true)
        .possible_values(&["s", "f", "sz", "fn", "size", "filename"]);

    let desc_arg = Arg::with_name("desc")
        .help("Use decending order when --order-by is specified (defaults to ascending)")
        .long("desc")
        .short("d")
        .required(false)
        .takes_value(false);

    let stats_arg = Arg::with_name("stats")
        .help("After searching, print some simple stats about the search performed.")
        .long("stats")
        .required(false)
        .takes_value(false);

    let verbose_arg = Arg::with_name("verbose")
        .help("Prints the parsed arguments, and the current search directory for each thread. Infers '--stats' as well.")
        .short("v")
        .long("verbose")
        .required(false)
        .takes_value(false);

    let images_arg = Arg::with_name("images")
        .help("Shortcuts to search for image file types")
        .long("images")
        .visible_aliases(&["image", "img", "imgs"])
        .required(false)
        .takes_value(false);

    let videos_arg = Arg::with_name("videos")
        .help("Shortcuts to search for video file types")
        .long("videos")
        .visible_aliases(&["video", "vid", "vids"])
        .required(false)
        .takes_value(false);

    let matches = App::new("find-rs")
        .author("mrudisel")
        .about("Simple find tool with multithreading")
        .version("0.1")
        .arg(root_arg)
        .arg(pattern_arg)
        .arg(exts_arg)
        .arg(regex_arg)
        .arg(fuzzy_arg)
        .arg(fuzzy_thresh_arg)
        .arg(min_size_arg)
        .arg(max_size_arg)
        .arg(workers_arg)
        .arg(type_arg)
        .arg(order_by_arg)
        .arg(desc_arg)
        .arg(images_arg)
        .arg(videos_arg)
        .arg(stats_arg)
        .arg(verbose_arg)
        .get_matches();

    FindArgs::from_arg_matches(&matches)
}
