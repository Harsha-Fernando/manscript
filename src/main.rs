fn main() {
    if let Err(err) = manscript::cli::dispatch() {
        err.print();
        std::process::exit(1);
    }
}
