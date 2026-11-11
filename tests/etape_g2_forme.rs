//! Étape G2 : l'étiquette `Forme` (`balayage::classer`) est une TABLE DE RÈGLES à seuils entiers (voir « L'étiquette » dans la doc de `src/balayage.rs`) : c'est un modèle à nous, ses seuils sont à nous,
//! ils ne viennent pas de l'article. Les tests vérifient que le code applique la table à la lettre : chaque règle des deux côtés de son seuil, l'ordre des règles, des formes synthétiques construites à la main.
//! Aucun test ici ne dit qu'une forme « ressemble » à une figure. Valeurs attendues : `references/etape-g2-valeurs.md`, identifiants `V-G2-FORME-xx`.

mod etape_g2_commun;

use etape_g2_commun::*;
use givre_lab::balayage::{
    Forme, Mesures, RAYON_MIN_FORME, SEUIL_FOUGERE, SEUIL_PLAQUE, SEUIL_SECTEURS, classer, mesurer,
};

/// Des mesures dont seuls `rayon`, `compacite` et `aiguilles` sont donnés ; `rattachees`, `pointes` et `hexagonalite` prennent des valeurs quelconques que l'étiquette ne doit pas lire.
fn m(rayon: u32, compacite: u32, aiguilles: u32) -> Mesures {
    Mesures {
        rattachees: 17 * rayon.min(1000) + 3,
        rayon,
        compacite,
        pointes: (3 * aiguilles.min(1000)).wrapping_add(rayon.min(1000)) % 1234,
        aiguilles,
        hexagonalite: (7 * compacite.min(1000) + 11) % 1001,
    }
}

/// juste : les constantes de la doc (V-G2-FORME-01) : `RAYON_MIN_FORME = 10`, `SEUIL_PLAQUE = 950`, `SEUIL_SECTEURS = 700`, `SEUIL_FOUGERE = 6`. Elles sont écrites en toutes lettres dans la table de la doc ;
/// les tests qui suivent emploient les nombres, pas les constantes (une constante fausse y est donc vue).
///
/// Ne couvre pas : l'usage des constantes dans `classer` (voir `chaque_regle_de_la_table_des_deux_cotes_de_son_seuil`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn les_constantes_de_la_table() {
    assert_eq!(RAYON_MIN_FORME, 10);
    assert_eq!(SEUIL_PLAQUE, 950);
    assert_eq!(SEUIL_SECTEURS, 700);
    assert_eq!(SEUIL_FOUGERE, 6);
}

/// juste : chaque règle de la table, des DEUX côtés de son seuil (V-G2-FORME-02). Règle 1 : `rayon < 10` donne `Aucune` (9 : `Aucune`, 10 : la suite) et passe avant toutes les autres (un disque plein de rayon 9 est
/// `Aucune`, pas `Plaque`). Règle 2 : `compacite >= 950` donne `Plaque` (950 : `Plaque`, 949 : `Secteurs`) et passe avant la règle 4 (une compacité de 1000 avec cinq cents aiguilles reste `Plaque`). Règle 3 :
/// `compacite >= 700` donne `Secteurs` (700 : `Secteurs`, 699 : la suite) et passe avant la règle 4. Règle 4 : `aiguilles >= 6` donne `Fougere` (6 aiguilles : `Fougere`, 5 : `Dendrite`, à tous les rayons : le
/// seuil ne dépend pas du rayon). Règle 5 : sinon `Dendrite`. `rattachees`, `pointes` et `hexagonalite` ne sont pas lus : l'étiquette ne dépend que de `rayon`, `compacite` et `aiguilles`.
///
/// Ne couvre pas : les seuils eux-mêmes (ils sont à nous : `MODELE.md`, « Les seuils de l'étiquette ») ; des mesures d'un vrai cristal (voir `des_formes_synthetiques_construites_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn chaque_regle_de_la_table_des_deux_cotes_de_son_seuil() {
    // (rayon, compacité, aiguilles, étiquette attendue, pourquoi)
    let cas: [(u32, u32, u32, Forme, &str); 30] = [
        (9, 1000, 0, Forme::Aucune, "règle 1 : rayon 9 < 10, même un disque plein"),
        (9, 0, 100_000, Forme::Aucune, "règle 1 passe avant la règle 4"),
        (0, 1000, 1, Forme::Aucune, "le germe seul"),
        (10, 1000, 0, Forme::Plaque, "rayon 10 : la règle 1 ne s'applique plus"),
        (10, 950, 5, Forme::Plaque, "règle 2 : compacité 950"),
        (20, 950, 0, Forme::Plaque, "règle 2 : compacité 950"),
        (20, 949, 0, Forme::Secteurs, "règle 2 : compacité 949, règle 3"),
        (20, 1000, 500, Forme::Plaque, "règle 2 passe avant la règle 4"),
        (20, 950, 6, Forme::Plaque, "règle 2 passe avant la règle 4 au seuil exact"),
        (20, 700, 0, Forme::Secteurs, "règle 3 : compacité 700"),
        (20, 699, 0, Forme::Dendrite, "règle 3 : compacité 699, règle 5"),
        (20, 800, 500, Forme::Secteurs, "règle 3 passe avant la règle 4"),
        (20, 700, 6, Forme::Secteurs, "règle 3 passe avant la règle 4 au seuil exact"),
        (20, 699, 6, Forme::Fougere, "règle 4 : 6 aiguilles"),
        (20, 699, 5, Forme::Dendrite, "règle 4 : 5 aiguilles ne suffisent pas"),
        (20, 0, 7, Forme::Fougere, "règle 4 : 7 aiguilles"),
        (40, 600, 6, Forme::Fougere, "règle 4 : le seuil ne dépend pas du rayon (40)"),
        (40, 600, 5, Forme::Dendrite, "règle 4 : le seuil ne dépend pas du rayon (40)"),
        (10, 100, 6, Forme::Fougere, "règle 4 au rayon 10"),
        (10, 100, 5, Forme::Dendrite, "règle 4 au rayon 10"),
        (400, 300, 6, Forme::Fougere, "règle 4 au rayon 400"),
        (400, 300, 5, Forme::Dendrite, "règle 4 au rayon 400"),
        (20, 400, 0, Forme::Dendrite, "règle 5"),
        (20, 0, 0, Forme::Dendrite, "règle 5 : une compacité nulle n'est pas `Aucune`"),
        (10, 0, 0, Forme::Dendrite, "règle 5 au rayon 10"),
        (10, 699, 5, Forme::Dendrite, "règle 5 au rayon 10"),
        (10, 4, u32::MAX, Forme::Fougere, "règle 4 : un nombre d'aiguilles énorme"),
        (u32::MAX, 0, 0, Forme::Dendrite, "règle 5 pour un rayon énorme"),
        (u32::MAX, 1000, 0, Forme::Plaque, "règle 2 pour un rayon énorme"),
        (u32::MAX, 0, 6, Forme::Fougere, "règle 4 pour un rayon énorme"),
    ];
    for (rayon, compacite, aiguilles, attendu, pourquoi) in cas {
        assert_eq!(
            classer(&m(rayon, compacite, aiguilles)),
            attendu,
            "rayon {rayon}, compacité {compacite}, aiguilles {aiguilles} : {pourquoi}"
        );
    }
    // `rattachees`, `pointes` et `hexagonalite` ne sont pas lus : mêmes étiquettes pour d'autres valeurs.
    for (rayon, compacite, aiguilles, attendu, pourquoi) in cas {
        for (rattachees, pointes, hexagonalite) in [(0_u32, 0_u32, 0_u32), (u32::MAX, u32::MAX, 1000), (12345, 7, 77)] {
            let mesures = Mesures { rattachees, rayon, compacite, pointes, aiguilles, hexagonalite };
            assert_eq!(classer(&mesures), attendu, "{mesures:?} : {pourquoi}");
        }
    }
}

/// juste : des FORMES SYNTHÉTIQUES construites à la main, mesurées par la bibliothèque puis rangées (V-G2-FORME-03), les mesures étant calculées à la main (et recomptées par `mesures_g2.py`) : un disque plein de rayon
/// 12 (469 cases, compacité 1000, 6 pointes, aucune aiguille) est `Plaque` ; le même disque de rayon 15 privé d'un coin (720 cases, compacité 998, 7 pointes) reste `Plaque` ; six bras de quinze cases de large (l'étoile
/// épaissie de 7 autour des demi-droites de longueur 20 : 1801 cases, rayon 27, compacité 793, 18 pointes, aucune aiguille) sont des `Secteurs` ; six bras de trois cases de large (361 cases, rayon 21, compacité 260, 18 pointes,
/// aucune aiguille) sont une `Dendrite` ; six lignes d'une case de longueur 20 (121 cases, rayon 20, compacité 95, 114 pointes, 114 aiguilles) sont une `Fougere` ; un seul bras d'une case de large et de 30 cases de long (31 cases,
/// compacité 11, 31 aiguilles) aussi (l'étiquette est un seuil sur nos mesures, pas une définition de la forme) ; le germe, un bras de 7 cases et un disque de rayon 9 sont `Aucune` (rayon de moins de 10).
///
/// Ne couvre pas : des cristaux du modèle (voir `etape_g2_ordre`, qui est une régression).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn des_formes_synthetiques_construites_a_la_main() {
    let mut disque_prive_d_un_coin = disque(15);
    disque_prive_d_un_coin.remove(&(15, 0));
    // (nom, domaine, cases, mesures attendues (rattachees, rayon, compacite, pointes, aiguilles), étiquette)
    type Cas = (&'static str, u32, Ensemble, (u32, u32, u32, u32, u32), Forme);
    let cas: Vec<Cas> = vec![
        ("disque de rayon 12", 20, disque(12), (469, 12, 1000, 6, 0), Forme::Plaque),
        ("disque de rayon 15 privé d'un coin", 20, disque_prive_d_un_coin, (720, 15, 998, 7, 0), Forme::Plaque),
        ("six bras de 15 cases de large", 40, etoile_epaisse(20, 7), (1801, 27, 793, 18, 0), Forme::Secteurs),
        ("six bras de 3 cases de large", 40, etoile_epaisse(20, 1), (361, 21, 260, 18, 0), Forme::Dendrite),
        ("six lignes d'une case", 40, etoile(20), (121, 20, 95, 114, 114), Forme::Fougere),
        ("un bras de 30 cases", 40, bras(30), (31, 30, 11, 31, 31), Forme::Fougere),
        ("germe seul", 40, disque(0), (1, 0, 1000, 1, 1), Forme::Aucune),
        ("bras de 7 cases", 40, bras(7), (8, 7, 47, 8, 8), Forme::Aucune),
        ("disque de rayon 9", 40, disque(9), (271, 9, 1000, 6, 0), Forme::Aucune),
        ("disque de rayon 10", 40, disque(10), (331, 10, 1000, 6, 0), Forme::Plaque),
    ];
    for (nom, domaine, ensemble, (rattachees, rayon, compacite, pointes, aiguilles), attendu) in cas {
        let mesures = mesurer(&neige_de(domaine, &ensemble, jeu_dense()));
        assert_eq!(
            (mesures.rattachees, mesures.rayon, mesures.compacite, mesures.pointes, mesures.aiguilles),
            (rattachees, rayon, compacite, pointes, aiguilles),
            "{nom} : mesures"
        );
        assert_eq!(classer(&mesures), attendu, "{nom} : étiquette");
    }
}

/// juste : l'étiquette ne dépend que des mesures (V-G2-FORME-04) : une même `Mesures` donne la même `Forme` (deux appels) et les formes se rangent comme la table le dit sur 3000 mesures tirées (le test réécrit la
/// table : `rayon < 10`, puis `compacite >= 950`, puis `>= 700`, puis `aiguilles >= 6`, sinon `Dendrite`) ; les cinq formes sont toutes atteintes par le tirage.
///
/// Ne couvre pas : que la table soit un bon classement (c'est un modèle à nous).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn la_table_reecrite_dans_le_test_donne_les_memes_etiquettes() {
    let table = |mesures: &Mesures| {
        if mesures.rayon < 10 {
            Forme::Aucune
        } else if mesures.compacite >= 950 {
            Forme::Plaque
        } else if mesures.compacite >= 700 {
            Forme::Secteurs
        } else if mesures.aiguilles >= 6 {
            Forme::Fougere
        } else {
            Forme::Dendrite
        }
    };
    let mut g = Lcg(2024);
    let mut vues = Vec::new();
    for _ in 0..3000 {
        let mesures = Mesures {
            rattachees: g.jusqu_a(5000) as u32,
            rayon: g.jusqu_a(60) as u32,
            compacite: g.jusqu_a(1001) as u32,
            pointes: g.jusqu_a(300) as u32,
            aiguilles: g.jusqu_a(12) as u32,
            hexagonalite: g.jusqu_a(1001) as u32,
        };
        let forme = classer(&mesures);
        assert_eq!(forme, table(&mesures), "{mesures:?}");
        assert_eq!(forme, classer(&mesures), "{mesures:?}");
        if !vues.contains(&forme) {
            vues.push(forme);
        }
    }
    assert_eq!(vues.len(), 5, "le tirage doit atteindre les cinq formes : {vues:?}");
}
