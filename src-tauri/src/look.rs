//! Cor e acessórios do pato. As listas e as regras de espaço vivem aqui;
//! o TypeScript (src/sprites/colors.ts e accessories.ts) só desenha.

pub const DEFAULT_COLOR: &str = "amarelo";

pub const COLORS: [&str; 15] = ["amarelo", "laranja", "branco", "cinza", "creme", "rosa", "menta", "azul", "marrom", "lavanda", "preto", "vermelho", "verde", "roxo", "turquesa"];

/// (id, espaço). No máximo um acessório por espaço.
pub const ACCESSORIES: [(&str, &str); 22] = [
    ("peruca-ruiva", "cabeca"),
    ("peruca-curta", "cabeca"),
    ("chapeu-festa", "cabeca"),
    ("coroa", "cabeca"),
    ("cartola", "cabeca"),
    ("chapeu-seletor", "cabeca"),
    ("chapeu-capivara", "cabeca"),
    ("oculos", "rosto"),
    ("oculos-escuros", "rosto"),
    ("gorro-natal", "cabeca"),
    ("chapeu-bruxa", "cabeca"),
    ("chapeu-pirata", "cabeca"),
    ("tapa-olho", "rosto"),
    ("capa", "pescoco"),
    ("mochila", "costas"),
    ("fones", "orelha"),
    ("laco", "pescoco"),
    ("flor", "enfeite"),
    ("varinha", "mao"),
    ("faca", "mao"),
    ("faca-sangue", "mao"),
    ("arminha", "mao"),
];

pub fn slot(id: &str) -> Option<&'static str> {
    ACCESSORIES.iter().find(|(a, _)| *a == id).map(|(_, s)| *s)
}

pub fn sanitize_color(c: &str) -> String {
    if COLORS.contains(&c) {
        c.to_string()
    } else {
        DEFAULT_COLOR.to_string()
    }
}

/// Descarta ids desconhecidos e, em cada espaço, fica com o último escolhido.
pub fn sanitize_accessories(list: &[String]) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for id in list {
        let Some(sl) = slot(id) else { continue };
        out.retain(|o| slot(o) != Some(sl));
        out.push(id.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn unknown_ids_are_dropped() {
        assert_eq!(sanitize_accessories(&v(&["coroa", "banana"])), v(&["coroa"]));
        assert_eq!(sanitize_color("roxo-neon"), "amarelo");
        assert_eq!(sanitize_color("rosa"), "rosa");
    }

    #[test]
    fn one_per_slot_last_wins() {
        assert_eq!(sanitize_accessories(&v(&["peruca-ruiva", "coroa"])), v(&["coroa"]));
        assert_eq!(sanitize_accessories(&v(&["oculos", "oculos-escuros"])), v(&["oculos-escuros"]));
    }

    #[test]
    fn capybara_hat_takes_the_head_slot() {
        assert_eq!(slot("chapeu-capivara"), Some("cabeca"));
        assert_eq!(sanitize_accessories(&v(&["coroa", "chapeu-capivara"])), v(&["chapeu-capivara"]));
        assert_eq!(sanitize_accessories(&v(&["chapeu-capivara", "oculos"])), v(&["chapeu-capivara", "oculos"]));
    }

    #[test]
    fn hand_items_replace_each_other() {
        for id in ["faca", "faca-sangue", "arminha", "varinha"] {
            assert_eq!(slot(id), Some("mao"), "{id}");
        }
        assert_eq!(sanitize_accessories(&v(&["faca", "faca-sangue"])), v(&["faca-sangue"]));
        assert_eq!(sanitize_accessories(&v(&["varinha", "arminha"])), v(&["arminha"]));
        assert_eq!(sanitize_accessories(&v(&["arminha", "faca", "coroa"])), v(&["faca", "coroa"]));
        assert_eq!(sanitize_accessories(&v(&["arminha", "coroa"])), v(&["arminha", "coroa"]));
    }

    #[test]
    fn different_slots_combine_and_dupes_collapse() {
        let r = sanitize_accessories(&v(&["peruca-curta", "oculos", "fones", "flor", "flor", "laco"]));
        assert_eq!(r, v(&["peruca-curta", "oculos", "fones", "flor", "laco"]));
    }
}
