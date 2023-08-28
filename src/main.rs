use mortar::mortar::embedded_eval;

fn main() {
    let mortar = mortar::mortar::Mortar::new();
    let cwd: &str = "test_dir";

    embedded_eval!(mortar, cwd, "star/mortar.star");
}
