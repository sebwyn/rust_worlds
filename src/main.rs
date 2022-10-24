use rust_worlds::App;

fn main() {
    pollster::block_on(App::run());   
}
