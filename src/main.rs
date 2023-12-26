fn main() {
    let mortar = mortar::mortar::Mortar::new();

    let a = mortar.eval_file("src/test.rhai");

    println!("{}", a);
}
