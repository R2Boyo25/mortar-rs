use mortar::mortar::embedded_eval;

fn main() {   
    let mortar = mortar::mortar::Mortar::new();
    let cwd: &str = "src";

    embedded_eval!(mortar, cwd, "star/mortar.star");
}
