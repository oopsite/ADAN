mod lexer;
mod parser;
mod std;

fn main() {
    let user_input = std::io::Terminal::input();

    std::io::Terminal::output(&user_input, true);
}
