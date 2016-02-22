extern crate unifind;
use std::io::Write;

fn main() {
    let mut args: Vec<String> = std::env::args().map(|l| l.to_uppercase()).collect();
    args.remove(0);
    let results = unifind::search_and(args.as_ref());

    // By using this instead of println!'ing everyting, we
    // won't panic if the parent process / shell  / console
    // does not read all our results :)
    let mut out = std::io::stdout();
    for result in results {
        let _ = write!(&mut out, "{}\n", result);
    }
}
