use std::fs;

fn main() -> Result<(), String> {
    let invocation_configuration = parse_args(std::env::args())?;
    let log = open_and_parse_log(invocation_configuration.target_logfile)?;

    for line in log.lines() {
        println!("{line}");
    }
    Ok(())
}

fn parse_args(mut args: std::env::Args) -> Result<InvocationConfiguration, String> {
    args.next().expect("invocation name must be present");
    match args.next() {
        None => Err("no log file path was provided as the first argument".to_string()),
        Some(target_logfile) => Ok(InvocationConfiguration { target_logfile }),
    }
}

fn open_and_parse_log(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|_| format!("Unable to read file: {0}", path))
}

struct InvocationConfiguration {
    target_logfile: String,
}
