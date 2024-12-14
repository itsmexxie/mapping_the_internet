use std::{env, process::ExitCode};

#[inline]
fn run(endpoint: &str) -> Result<minreq::Response, minreq::Error> {
    minreq::get(endpoint).send()
}

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<String>>();
    match args.len() {
        l if l < 2 => panic!("Too few arguments!"),
        l if l > 2 => panic!("Too many arguments!"),
        _ => match run(&args[1]) {
            Ok(res) => {
                let code = res.status_code;

                if code > 299 {
                    println!("{}", code);
                    return ExitCode::from(1);
                } else {
                    return ExitCode::from(0);
                }
            }
            Err(err) => {
                println!("{}", err);
                return ExitCode::from(1);
            }
        },
    }
}
