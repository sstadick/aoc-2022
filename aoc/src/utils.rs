use std::{
    error::Error,
    fmt::{self, Debug},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct SlurpError {
    line: usize,
    msg: String,
}

impl fmt::Display for SlurpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error at line {}: {}", self.line, self.msg)
    }
}

impl Error for SlurpError {}

#[allow(clippy::missing_errors_doc)]
pub fn slurp_file<P, T>(path: P) -> Result<Vec<T>, SlurpError>
where
    P: AsRef<Path>,
    T: FromStr,
    <T as FromStr>::Err: Error,
{
    let reader = File::open(&path).map(BufReader::new).expect("Failed to open file");
    let mut result = vec![];
    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| SlurpError { line: i, msg: e.to_string() })?;
        result.push(line.parse::<T>().map_err(|e| SlurpError { line: i, msg: e.to_string() })?);
    }
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct ParseError {
    msg: String,
}
impl ParseError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error for command: {}", self.msg)
    }
}
