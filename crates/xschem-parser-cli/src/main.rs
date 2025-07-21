use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use colored::Colorize;

fn main() -> ExitCode {
    let start = Instant::now();

    let (count, errors) = std::env::args().skip(1).fold((0, 0), |(count, errors), a| {
        let path = Path::new(&a);
        match std::fs::read_to_string(path) {
            Ok(contents) => match xschem_parser::from_str_file(&contents, path) {
                Ok(_) => (count + 1, errors),
                Err(e) => {
                    eprintln!("{e}");
                    (count + 1, errors + 1)
                }
            },
            Err(e) => {
                eprintln!(
                    "{error}: {desc}\n\
                     {ptr}{path}",
                    error = "error".red().bold(),
                    desc = e.to_string().bold(),
                    ptr = "  --> ".blue().bold(),
                    path = path.display(),
                );
                (count + 1, errors + 1)
            }
        }
    });

    let end = Instant::now();
    let elapsed = end.duration_since(start);

    if errors == 0 {
        if count > 0 {
            eprintln!(
                "{}",
                format!(
                    "successfully parsed {count} files in {elapsed:.3}s",
                    elapsed = elapsed.as_secs_f64(),
                )
                .green()
                .bold(),
            );
        }
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "\n{}",
            format!(
                "found {errors} errors in {count} files in {elapsed:.3}s",
                elapsed = elapsed.as_secs_f64(),
            )
            .red()
            .bold(),
        );
        ExitCode::FAILURE
    }
}
