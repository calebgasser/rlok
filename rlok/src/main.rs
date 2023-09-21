use rlok_lib::interpreter::Interpreter;
use std::env;

fn main() {
    Interpreter::build().start(env::args().collect());
}
