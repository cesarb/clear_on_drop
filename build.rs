extern crate gcc;

fn main() {
    if !cfg!(feature = "no_cc") {
        gcc::compile_library("libclear_on_drop.a", &["src/hide.c"]);
    }
}
