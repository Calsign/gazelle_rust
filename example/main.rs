use clap::Parser;

#[derive(clap::Parser)]
struct Args {
    message: Option<String>,
}

macro_lib::macro_msg!();

fn main() {
    let args = Args::parse();

    if let Some(msg) = args.message {
        println!("{}", msg);
    } else {
        let msg = lib::get_message();
        println!("{}", msg);
    }
}
