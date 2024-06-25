use clap::Parser;

#[derive(clap::Parser, Debug)]
struct Args {
    message: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Some(msg) = args.message {
        println!("{}", msg);
    } else {
        let msg = lib::get_message();
        println!("{}", msg);
    }
}
