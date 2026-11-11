//! Étape G1 : le modèle de Gravner et Griffeath sans bruit, EXACT : le simulateur NAÏF du test (`etape_g1_commun::Naif`, écrit d'après
//! la doc de `neige` : un dictionnaire ordonné, des `i128`, des arrondis écrits à la main) donne exactement les états de `Neige`, à
//! chaque pas ; des états et des pas calculés à la main ; des valeurs calculées par le code Python indépendant (`g1_ref.py`) ; les
//! refus et les valeurs extrêmes. Valeurs attendues : `references/etape-g1-valeurs.md`, identifiants `V-G1-xx`.

mod etape_g1_commun;

use etape_g1_commun::valeurs::TRAJECTOIRES;
use etape_g1_commun::*;
use givre_lab::empreinte::sha256_hex;
use givre_lab::math::Fixe;
use givre_lab::neige::{Case, ErreurNeige, Neige, Parametres, RAYON_MAX};
use std::collections::BTreeMap;

/// Fait avancer `Neige` et le naïf pas à pas depuis le même état et vérifie l'égalité de toutes les cases (`a`, `b`, `c`, `d`), de la
/// masse totale, du temps et de l'empreinte à chaque pas.
fn comparer(mut neige: Neige, mut naif: Naif, pas: u64, etiquette: &str) -> Compteurs {
    assert_eq!(etat_de(&neige), etat_du_naif(&naif), "{etiquette} : état de départ");
    for t in 1..=pas {
        neige.pas();
        naif.pas();
        assert_eq!(neige.temps(), t);
        assert_eq!(etat_de(&neige), etat_du_naif(&naif), "{etiquette} : pas {t}");
        assert_eq!(neige.masse_totale(), naif.masse_totale(), "{etiquette} : masse au pas {t}");
    }
    assert_eq!(neige.cases_rattachees(), naif.rattachees(), "{etiquette} : cases rattachées");
    assert_eq!(neige.masse_du_cristal(), naif.masse_du_cristal(), "{etiquette} : masse du cristal");
    assert_eq!(neige.rayon_cristal() as i32, naif.rayon_cristal(), "{etiquette} : rayon du cristal");
    naif.compteurs
}

/// Un pas de la bibliothèque (et du naïf, dont l'état doit être le même) depuis un état donné à la main.
fn un_pas(rayon: u32, p: &Parametres, etat: &BTreeMap<(i32, i32), Brut>, etiquette: &str) -> Neige {
    let (mut neige, mut naif) = depuis_un_etat(rayon, p, etat);
    neige.pas();
    naif.pas();
    assert_eq!(etat_de(&neige), etat_du_naif(&naif), "{etiquette} : la bibliothèque et le naïf diffèrent");
    neige
}

/// juste : depuis l'état de départ de la doc, pour six jeux de réglages (dont trois de l'article : `FIG9_RHO_050`, `FIG9_RHO_040`,
/// `FIG13_MILIEU`) et des rayons de 3 à 12 (un petit rayon pour que le cristal touche le mur, un grand pour qu'il ne le touche pas),
/// 100 à 300 pas : toutes les cases (`a`, `b`, `c`, `d`), la masse totale, le temps, les mesures et l'empreinte sont exactement ceux du
/// simulateur naïf écrit dans ce test, à chaque pas (V-G1-10). Chaque branche de la règle de rattachement est exercée (une, deux, trois
/// voisines et plus ; `b >= 1` ; la lame de couteau franchie et non franchie sur `α` ou sur `θ`) : le test compte les branches prises
/// par le naïf et exige que chacune l'ait été (un jeu qui ne les exercerait pas passerait à vide).
///
/// Ne couvre pas : les états sans symétrie (voir `naif_depuis_des_etats_sans_symetrie`), les égalités exactes des seuils (voir
/// `regles_de_rattachement_une_a_une` et `theta_est_strict`) ; le naïf et la bibliothèque ont la même lecture de la doc (l'article n'est
/// pas ce test).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn naif_depuis_l_etat_de_depart() {
    let jeux: [(u32, Jeu, u64); 9] = [
        (12, FIG9_RHO_050, 200),
        (12, FIG9_RHO_040, 300),
        (12, FIG13_MILIEU, 300),
        (12, JEU_B, 300),
        (10, JEU_C, 100),
        (10, JEU_D, 200),
        (10, JEU_E, 100),
        (4, JEU_C, 100),
        (3, FIG9_RHO_050, 200),
    ];
    let mut total = Compteurs::default();
    for (rayon, jeu, pas) in jeux {
        let p = p(jeu);
        let k = comparer(
            Neige::essayer_new(rayon, p).expect("neige valide"),
            Naif::initial(rayon as i32, &p),
            pas,
            &format!("R={rayon} {jeu:?}"),
        );
        total.n12_oui += k.n12_oui;
        total.n12_non += k.n12_non;
        total.n3_seuil_un += k.n3_seuil_un;
        total.n3_couteau += k.n3_couteau;
        total.n3_theta_ok_alpha_non += k.n3_theta_ok_alpha_non;
        total.n3_alpha_ok_theta_non += k.n3_alpha_ok_theta_non;
        total.n4_plus += k.n4_plus;
    }
    for (nom, nombre) in [
        ("n = 1 ou 2, rattachée", total.n12_oui),
        ("n = 1 ou 2, pas rattachée", total.n12_non),
        ("n = 3, b >= 1", total.n3_seuil_un),
        ("n = 3, lame de couteau", total.n3_couteau),
        ("n = 3, θ franchi mais pas α", total.n3_theta_ok_alpha_non),
        ("n = 3, α franchi mais pas θ", total.n3_alpha_ok_theta_non),
        ("n >= 4", total.n4_plus),
    ] {
        assert!(nombre > 0, "la branche « {nom} » n'est jamais exercée par ces jeux");
    }
}

/// juste : depuis des états de départ SANS symétrie, valides et tirés par un congruentiel linéaire (graines 1 à 40, écrites ici : un
/// amas rattaché au hasard, des masses de bord et de vapeur quelconques ; rayon 4, 5 ou 6 : le mur compte), pour six jeux de
/// réglages, 8 pas chacun, avec des vapeurs quelconques (graines impaires) ou multiples de 7 `brut` (graines paires) (V-G1-11) :
/// l'état de la bibliothèque est celui du naïf, case par case, à chaque pas. Une voisine oubliée ou mal orientée, une diffusion en
/// place, un rattachement lu après mise à jour, l'ordre des sous-pas, un mur qui ne réfléchit pas, un germe décalé échouent ici.
///
/// Ne couvre pas : un pas long (voir `naif_depuis_l_etat_de_depart`) ; les états qui ne sont pas valides (voir `refus_des_etats`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn naif_depuis_des_etats_sans_symetrie() {
    let jeux = [FIG9_RHO_050, FIG13_MILIEU, JEU_B, JEU_C, JEU_D, JEU_E];
    for graine in 1..=40_u64 {
        let rayon = 4 + (graine % 3) as u32;
        let jeu = jeux[(graine as usize) % jeux.len()];
        let p = p(jeu);
        let etat = etat_au_hasard(rayon as i32, graine, graine % 2 == 0);
        let (neige, naif) = depuis_un_etat(rayon, &p, &etat);
        comparer(neige, naif, 8, &format!("graine {graine}, R={rayon}, {jeu:?}"));
    }
}

/// juste : le premier pas, calculé à la main (V-G1-12), `R = 1` (le germe et ses six voisines, dont trois cases absentes autour de
/// chaque voisine : le mur), `ρ = 1/2`, `β = 2`, `α = 1/4`, `θ = 1/8`, `κ = 1/4`, `μ = γ = 1/2`. Sous-pas 1 : chaque voisine a pour somme
/// `7 ρ` (le germe rattaché, et les trois cases absentes, sont remplacés par sa propre vapeur) donc `d1 = ρ = 2^31` ; sous-pas 2 :
/// `g = κ ρ = 2^29`, `c2 = 2^29`, `b2 = 2^31 - 2^29 = 3 * 2^29` ; sous-pas 3 : `n = 1` et `b2 = 0,375 < β` : rien ne se rattache ;
/// sous-pas 4 : `fb = μ b2 = 3 * 2^28`, `fc = γ c2 = 2^28`, donc `b = 3 * 2^28`, `c = 2^28`, `d = 2^30`. Le germe ne change pas. La masse
/// de chaque voisine est restée `2^31` (conservée), et un deuxième pas redonne le même état (l'anneau uniforme est stationnaire).
/// Si le mur comptait pour 0 au lieu de la vapeur de la case, `d1` serait `4/7 ρ`.
///
/// Ne couvre pas : un rayon plus grand (voir les autres tests) ; l'arrondi de la division par 7 (ici elle est exacte).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn premier_pas_a_la_main() {
    let p = parametres("0.5", "2", "0.25", "0.125", "0.25", "0.5", "0.5");
    let mut neige = Neige::essayer_new(1, p).expect("neige valide");
    let rho = 1_i128 << 31;
    for &(q, r) in &cases(1) {
        let attendu = if (q, r) == (0, 0) { [1, 0, UN, 0] } else { [0, 0, 0, rho] };
        assert_eq!(etat_de(&neige)[cases(1).iter().position(|&c| c == (q, r)).unwrap()], attendu);
    }
    assert_eq!(neige.masse_totale(), UN + 6 * rho);
    neige.pas();
    for &(q, r) in &cases(1) {
        let c = neige.case(q, r).expect("case");
        if (q, r) == (0, 0) {
            assert_eq!(c, case_de([1, 0, UN, 0]));
        } else {
            assert_eq!((c.a, c.b.0, c.c.0, c.d.0), (false, 3 << 28, 1 << 28, 1 << 30), "case ({q}, {r})");
        }
    }
    assert_eq!(neige.masse_totale(), UN + 6 * rho);
    let apres_un = neige.cases();
    neige.pas();
    assert_eq!(neige.cases(), apres_un, "l'anneau uniforme est stationnaire");
    assert_eq!(neige.temps(), 2);
}

/// juste : le mur calculé à la main (V-G1-13), `R = 1`, aucune case rattachée, une seule vapeur de 7000 `brut` en `(1, 0)` (une case du
/// mur : trois voisines présentes, trois absentes). Sous-pas 1 : `(1, 0)` a pour somme `7000 + 3 * 0 + 3 * 7000 = 28000` donc 4000 ; le
/// centre, `(0, 1)` et `(1, -1)` (qui voient `(1, 0)` une fois) ont pour somme 7000 donc 1000 ; les autres 0. La masse (7000) est
/// conservée exactement. Aucune case n'est de bord : les sous-pas 2 à 4 ne changent rien. Un mur qui perdrait la vapeur (voisin absent
/// compté pour 0) donnerait 1000 à `(1, 0)` ; un mur qui diffuserait vers l'extérieur ne conserverait plus la masse.
///
/// Ne couvre pas : un mur avec un cristal (voir `premier_pas_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn le_mur_reflechit_a_la_main() {
    let p = parametres("0.5", "2", "0.25", "0.125", "0.25", "0.5", "0.5");
    let mut etat = etat_vide(1);
    etat.insert((1, 0), [0, 0, 0, 7000]);
    let neige = un_pas(1, &p, &etat, "mur");
    let attendu = |q, r| match (q, r) {
        (1, 0) => 4000,
        (0, 0) | (0, 1) | (1, -1) => 1000,
        _ => 0,
    };
    for &(q, r) in &cases(1) {
        let c = neige.case(q, r).expect("case");
        assert_eq!((c.a, c.b.0, c.c.0, c.d.0), (false, 0, 0, attendu(q, r)), "case ({q}, {r})");
    }
    assert_eq!(neige.masse_totale(), 7000);
}

/// juste : chaque règle de rattachement prise une à une, sur un état à la main (V-G1-14) : `R = 3`, une case cible `(0, 0)` de masse de
/// bord `b` (le reste : aucune vapeur, `d = 0` partout, donc `T = 0`), un nombre `n` de voisines rattachées (de 1 à 6, dans des
/// directions variées), `β = 1,5`, `α = 1/4`, `θ` variable, `μ = 1/2`. Résultats attendus, écrits à la main : la cible se rattache
/// (`a = 1`, `c = b`, `b = 0`, `d = 0`) ou non ; si elle ne se rattache pas, elle fond (`fb = (b + 1) >> 1` : `b' = b - fb`, `d' = fb`).
/// Pour 1 ou 2 voisines : `b >= β` (égalité comprise) ; pour 3 : `b >= 1` (égalité comprise) ou (`T < θ` STRICT et `b >= α`, égalité
/// comprise) : avec `θ = 0` et `T = 0` la lame ne s'ouvre jamais ; pour 4 à 6 : toujours, même avec `b = 0`. Un `>` pour un `>=`, un `<=`
/// pour un `<`, `β` pris pour 1, `α` et `θ` échangés, un seuil `n >= 5` au lieu de 4, une fonte appliquée à une case qui vient de se
/// rattacher échouent ici.
///
/// Ne couvre pas : une vapeur voisine non nulle (voir `theta_est_strict`) ; l'égalité `T = θ` avec `T > 0`.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn regles_de_rattachement_une_a_une() {
    let beta = 3_i128 << 31;
    let alpha = 1_i128 << 30;
    // (voisines rattachées par direction, b de la cible, θ, la cible se rattache ?)
    let lignes: Vec<(&[usize], i128, &str, bool)> = vec![
        (&[0], beta, "0", true),
        (&[3], beta - 1, "0", false),
        (&[0, 2], beta, "0.125", true),
        (&[1, 4], beta - 1, "0.125", false),
        (&[0, 3], beta, "8", true),
        (&[2, 5], UN, "0", false),
        (&[0, 2, 4], UN, "0", true),
        (&[0, 2, 4], UN - 1, "0", false),
        (&[0, 1, 2], alpha, "0.125", true),
        (&[0, 1, 2], alpha - 1, "0.125", false),
        (&[1, 3, 5], UN - 1, "0.125", true),
        (&[0, 1, 2], alpha - 1, "8", false),
        (&[0, 2, 4], beta, "0.125", true),
        (&[0, 1, 2], 0, "0.125", false),
        (&[0, 1, 2, 3], 0, "0", true),
        (&[0, 1, 3, 4], 0, "0.125", true),
        (&[0, 1, 2, 3, 4], 0, "0", true),
        (&[0, 1, 2, 3, 4, 5], 0, "0", true),
        (&[1, 2, 4, 5], 7, "0.125", true),
    ];
    for (i, &(dirs, b, theta, attendue)) in lignes.iter().enumerate() {
        let p = parametres("0.5", "1.5", "0.25", theta, "0.5", "0.5", "0.5");
        let mut etat = etat_vide(3);
        for &k in dirs {
            let (dq, dr) = DIRECTIONS[k];
            etat.insert((dq, dr), [1, 0, UN, 0]);
        }
        etat.insert((0, 0), [0, b, 0, 0]);
        let neige = un_pas(3, &p, &etat, &format!("ligne {i}"));
        let cible = neige.case(0, 0).expect("cible");
        let fb = (b + 1) >> 1;
        let attendu = if attendue { [1, 0, b, 0] } else { [0, b - fb, 0, fb] };
        let obtenu = [i128::from(cible.a), i128::from(cible.b.0), i128::from(cible.c.0), i128::from(cible.d.0)];
        assert_eq!(obtenu, attendu, "ligne {i} : n = {}, b = {b}, θ = {theta}", dirs.len());
        // les voisines rattachées ne changent pas (plus de dynamique aux cases rattachées)
        for &k in dirs {
            let (dq, dr) = DIRECTIONS[k];
            assert_eq!(neige.case(dq, dr), Some(case_de([1, 0, UN, 0])), "ligne {i} : voisine rattachée {k}");
        }
    }
}

/// juste : le rattachement est SYNCHRONE (V-G1-15) : `R = 3`, deux cases voisines de bord `x = (0, 0)` et `y = (0, 1)`, chacune avec les
/// MÊMES deux voisines rattachées `(1, 0)` et `(-1, 1)` (donc `n = 2` pour toutes deux), `β = 1,5` : l'une a `b = β` (elle se rattache),
/// l'autre `b = 1 + 1000 unités` (elle ne se rattache pas : avec `n = 2` il lui faudrait `β`). Un rattachement lu après mise à jour (la
/// voisine qui vient de se rattacher compterait : `n = 3`, et `b >= 1` suffirait) rattacherait la seconde ; on essaie les deux rôles, de
/// sorte que la case traitée la première dans l'ordre canonique soit tantôt l'une tantôt l'autre.
///
/// Ne couvre pas : l'ordre d'autres sous-pas (voir le naïf).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn le_rattachement_est_synchrone() {
    let p = parametres("0.5", "1.5", "0.25", "0", "0.5", "0", "0");
    let beta = 3_i128 << 31;
    let moyen = UN + 1000;
    for (attaquant, victime) in [((0, 0), (0, 1)), ((0, 1), (0, 0))] {
        let mut etat = etat_vide(3);
        etat.insert((1, 0), [1, 0, UN, 0]);
        etat.insert((-1, 1), [1, 0, UN, 0]);
        etat.insert(attaquant, [0, beta, 0, 0]);
        etat.insert(victime, [0, moyen, 0, 0]);
        let neige = un_pas(3, &p, &etat, "synchrone");
        let a = neige.case(attaquant.0, attaquant.1).expect("attaquant");
        let v = neige.case(victime.0, victime.1).expect("victime");
        assert_eq!((a.a, a.b.0, a.c.0), (true, 0, beta as i64), "l'attaquante se rattache");
        assert_eq!(
            (v.a, v.b.0, v.c.0),
            (false, moyen as i64, 0),
            "la victime ne se rattache pas : n = 2 au début du pas"
        );
    }
}

/// juste : `θ` est un seuil STRICT sur la vapeur du voisinage APRÈS le gel (V-G1-16), `R = 3`, `κ = μ = γ = 0` : trois voisines
/// rattachées contiguës `(1, 0)`, `(0, 1)`, `(-1, 1)` autour de la cible `(0, 0)` ; une vapeur de `7 m` (`m = 2^28`) sur la case lointaine
/// `(0, -1)`. Après le sous-pas 1, la cible, `(−1, 0)`, `(1, −1)` et `(0, −1)` valent `m` ; après le gel (`κ = 0` : toute la vapeur d'une case
/// de bord devient masse de bord), seule `(0, -1)` (lointaine) garde `m` : `T = m` exactement, et `b2(cible) = b + m`. On pose `b = α - m` :
/// `b2 = α` pile. Avec `θ = m` : `T < θ` est faux, la cible ne se rattache pas ; avec `θ = m + 1` elle se rattache (`c = α`) ; avec `θ = m + 1` et
/// `b = α - m - 1` elle ne se rattache pas (`b2 = α - 1`). Un `<=` pour un `<`, une vapeur lue avant le gel (alors `T = 4 m`), une vapeur de la
/// cible oubliée ou comptée deux fois, `α` et `θ` échangés échouent ici.
///
/// Ne couvre pas : `T` quand le voisinage compte plusieurs cases lointaines (voir le naïf).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn theta_est_strict() {
    let m: i128 = 1 << 28;
    let alpha = 1_i128 << 30;
    // (θ en `brut`, b de la cible, la cible se rattache ?)
    let lignes: [(i128, i128, bool); 3] =
        [(m, alpha - m, false), (m + 1, alpha - m, true), (m + 1, alpha - m - 1, false)];
    for (i, &(theta, b, attendue)) in lignes.iter().enumerate() {
        let mut p = parametres("0.5", "1.5", "0.25", "0", "0", "0", "0");
        p.theta = Fixe(theta as i64);
        let mut etat = etat_vide(3);
        for k in [0, 1, 2] {
            etat.insert(DIRECTIONS[k], [1, 0, UN, 0]);
        }
        etat.insert((0, 0), [0, b, 0, 0]);
        etat.insert((0, -1), [0, 0, 0, 7 * m]);
        let neige = un_pas(3, &p, &etat, &format!("θ ligne {i}"));
        let cible = neige.case(0, 0).expect("cible");
        // b2 = b + m (κ = 0 : toute la vapeur de la case de bord devient masse de bord) ; rien ne fond (μ = γ = 0)
        let attendu = if attendue { [1, 0, b + m, 0] } else { [0, b + m, 0, 0] };
        let obtenu = [i128::from(cible.a), i128::from(cible.b.0), i128::from(cible.c.0), i128::from(cible.d.0)];
        assert_eq!(obtenu, attendu, "ligne {i} : θ = {theta} brut, b = {b}");
    }
}

/// juste : des trajectoires dont l'empreinte (SHA-256 de l'écriture canonique de la doc), le nombre de cases rattachées, le rayon du
/// cristal, la masse totale et la masse du cristal ont été calculés par le code Python indépendant `g1_ref.py` (écrit d'après la
/// doc seule, sans lire le Rust), depuis l'état de départ (V-G1-17) : 8 trajectoires (jeux de l'article et jeux d'essai, rayons 3 à 12,
/// 25 à 300 pas, dont un cristal qui touche le mur). Deux implémentations écrites d'après la même doc donnent les mêmes bits.
///
/// Ne couvre pas : les états sans symétrie ; le Python et la bibliothèque ont la même lecture de la doc.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn trajectoires_calculees_par_python() {
    for &(nom, rayon, pas, jeu, empreinte, rattachees, rayon_cristal, masse, masse_cristal) in TRAJECTOIRES {
        let mut neige = Neige::essayer_new(rayon, p(jeu)).expect("neige valide");
        neige.avancer(pas);
        let etiquette = format!("{nom} R={rayon} {pas} pas");
        assert_eq!(neige.temps(), pas, "{etiquette}");
        assert_eq!(neige.empreinte(), empreinte, "{etiquette}");
        assert_eq!(neige.cases_rattachees(), rattachees, "{etiquette}");
        assert_eq!(neige.rayon_cristal(), rayon_cristal, "{etiquette}");
        assert_eq!(neige.masse_totale(), masse, "{etiquette}");
        assert_eq!(neige.masse_du_cristal(), masse_cristal, "{etiquette}");
    }
}

/// juste : l'écriture canonique écrite à la main (V-G1-18), `R = 1`, `ρ = 1/2`, `β = 2`, `α = 1/4`, `θ = 1/8`, `κ = 1/4`, `μ = γ = 1/2` : l'étiquette
/// `GIVREG01`, le rayon en `u32`, les sept réglages en `i64` dans l'ordre `ρ β α θ κ μ γ`, le temps en `u64`, puis, case par case dans l'ordre
/// canonique (`(0,-1) (1,-1) (-1,0) (0,0) (1,0) (-1,1) (0,1)`), `a` sur un octet puis `b`, `c`, `d` en `i64` : `76 + 25 * 7 = 251` octets, petit-boutistes. Au
/// temps 0 puis au temps 1 (états de `premier_pas_a_la_main`). L'empreinte est le SHA-256 de ces octets.
///
/// Ne couvre pas : un état sans symétrie (l'ordre des cases n'y est vu que par le centre au milieu de la liste).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn ecriture_canonique_a_la_main() {
    let p = parametres("0.5", "2", "0.25", "0.125", "0.25", "0.5", "0.5");
    let ecrire = |temps: u64, cellule: Brut, centre: Brut| -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"GIVREG01");
        o.extend_from_slice(&1_u32.to_le_bytes());
        for brut in [1_i64 << 31, 2 << 32, 1 << 30, 1 << 29, 1 << 30, 1 << 31, 1 << 31] {
            o.extend_from_slice(&brut.to_le_bytes());
        }
        o.extend_from_slice(&temps.to_le_bytes());
        for &c in &[(0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1)] {
            let v = if c == (0, 0) { centre } else { cellule };
            o.push(v[0] as u8);
            for &masse in &v[1..4] {
                o.extend_from_slice(&(masse as i64).to_le_bytes());
            }
        }
        o
    };
    let mut neige = Neige::essayer_new(1, p).expect("neige valide");
    let a_zero = ecrire(0, [0, 0, 0, 1 << 31], [1, 0, UN, 0]);
    assert_eq!(a_zero.len(), 251);
    assert_eq!(neige.ecriture_canonique(), a_zero);
    assert_eq!(neige.empreinte(), sha256_hex(&a_zero));
    neige.pas();
    let a_un = ecrire(1, [0, 3 << 28, 1 << 28, 1 << 30], [1, 0, UN, 0]);
    assert_eq!(neige.ecriture_canonique(), a_un);
    assert_eq!(neige.empreinte(), sha256_hex(&a_un));
    assert_ne!(sha256_hex(&a_zero), sha256_hex(&a_un));
}

/// juste : les accesseurs et l'état de départ (V-G1-19), pour les rayons 1, 2, 5, 10 : `cases()` a `3 R (R + 1) + 1` cases dans l'ordre canonique
/// (écrit dans le test), le centre vaut `(a = 1, b = 0, c = 1, d = 0)`, toute autre case `(0, 0, 0, ρ)` ; `temps() = 0`, `rayon()`, `parametres()`
/// rendent ce qu'on a donné ; `masse_totale() = 1 + (cases - 1) ρ` en `brut` ; `cases_rattachees() = 1`, `rayon_cristal() = 0`, `masse_du_cristal() = UN` ;
/// `case(q, r)` rend `None` hors de l'hexagone (`R + 1`, `i32::MAX`, `i32::MIN`) sans paniquer ; `avancer(7)` est sept `pas()`.
///
/// Ne couvre pas : l'état après un pas (voir les autres tests).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn etat_de_depart_et_accesseurs() {
    let p = p(FIG9_RHO_050);
    for rayon in [1_u32, 2, 5, 10] {
        let mut neige = Neige::essayer_new(rayon, p).expect("neige valide");
        let ordre = cases(rayon as i32);
        let nombre = 3 * (rayon as usize) * (rayon as usize + 1) + 1;
        assert_eq!(ordre.len(), nombre);
        assert_eq!(neige.cases().len(), nombre);
        assert_eq!((neige.rayon(), neige.temps(), neige.parametres()), (rayon, 0, p));
        for (i, &(q, r)) in ordre.iter().enumerate() {
            let attendu =
                if (q, r) == (0, 0) { case_de([1, 0, UN, 0]) } else { case_de([0, 0, 0, i128::from(p.rho.0)]) };
            assert_eq!(neige.cases()[i], attendu, "case {i} = ({q}, {r})");
            assert_eq!(neige.case(q, r), Some(attendu));
        }
        assert_eq!(neige.masse_totale(), UN + (nombre as i128 - 1) * i128::from(p.rho.0));
        assert_eq!((neige.cases_rattachees(), neige.rayon_cristal(), neige.masse_du_cristal()), (1, 0, UN));
        let r = rayon as i32;
        for (q, r) in [
            (r + 1, 0),
            (0, r + 1),
            (-r - 1, 0),
            (r, r),
            (i32::MAX, 0),
            (0, i32::MIN),
            (i32::MAX, i32::MAX),
            (i32::MIN, i32::MIN),
        ] {
            assert_eq!(neige.case(q, r), None, "({q}, {r}) n'est pas dans l'hexagone de rayon {rayon}");
        }
        if rayon == 5 {
            let mut autre = neige.clone();
            neige.avancer(7);
            for _ in 0..7 {
                autre.pas();
            }
            assert_eq!(neige.temps(), 7);
            assert_eq!(neige.cases(), autre.cases());
            assert_eq!(neige.empreinte(), autre.empreinte());
        }
    }
}

/// juste : les refus de `essayer_new` et de `Parametres::verifier` (V-G1-20) : un rayon de 0, de `RAYON_MAX + 1` ou de `u32::MAX` rend
/// `Err(RayonHorsBorne)` (testé AVANT les réglages) ; `RAYON_MAX` vaut celui d'une directive `hexagone` (2047) ; chaque réglage est accepté à ses deux
/// bornes (incluses : `ρ`, `α`, `κ`, `μ`, `γ` dans `[0, 1]`, `β` dans `[1, 8]`, `θ` dans `[0, 8]`) et refusé à un `brut` au-delà, avec son nom ; si plusieurs
/// sont hors plage, c'est le premier dans l'ordre `rho, beta, alpha, theta, kappa, mu, gamma` ; jamais de panique.
///
/// Ne couvre pas : les refus de `essayer_depuis_cases` (voir `refus_des_etats`) ni ceux de `Parametres::lire`.
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn refus_de_la_construction() {
    let bon = p(FIG9_RHO_050);
    assert_eq!(RAYON_MAX, givre_lab::scenario::RAYON_MAX);
    assert_eq!(RAYON_MAX, 2047);
    for rayon in [0, RAYON_MAX + 1, 5000, u32::MAX] {
        assert_eq!(Neige::essayer_new(rayon, bon).err(), Some(ErreurNeige::RayonHorsBorne), "rayon {rayon}");
    }
    let mut tout_faux = bon;
    tout_faux.rho = Fixe(UN as i64 + 1);
    assert_eq!(
        Neige::essayer_new(0, tout_faux).err(),
        Some(ErreurNeige::RayonHorsBorne),
        "le rayon est testé avant les réglages"
    );
    assert_eq!(bon.verifier(), Ok(()));

    let un = UN as i64;
    // (nom, borne basse, borne haute, le champ)
    type Champ = fn(&mut Parametres) -> &mut Fixe;
    let champs: [(&str, i64, i64, Champ); 7] = [
        ("rho", 0, un, |p| &mut p.rho),
        ("beta", un, 8 * un, |p| &mut p.beta),
        ("alpha", 0, un, |p| &mut p.alpha),
        ("theta", 0, 8 * un, |p| &mut p.theta),
        ("kappa", 0, un, |p| &mut p.kappa),
        ("mu", 0, un, |p| &mut p.mu),
        ("gamma", 0, un, |p| &mut p.gamma),
    ];
    for (nom, bas, haut, champ) in champs {
        for valeur in [bas, haut] {
            let mut p = bon;
            *champ(&mut p) = Fixe(valeur);
            assert_eq!(p.verifier(), Ok(()), "{nom} = {valeur} brut est permis");
            assert!(Neige::essayer_new(3, p).is_ok(), "{nom} = {valeur} brut est permis");
        }
        for valeur in [bas - 1, haut + 1, i64::MIN, i64::MAX] {
            let mut p = bon;
            *champ(&mut p) = Fixe(valeur);
            let attendu = Err(ErreurNeige::ParametreHorsPlage(nom));
            assert_eq!(p.verifier(), attendu, "{nom} = {valeur} brut est refusé");
            assert_eq!(Neige::essayer_new(3, p).err(), Some(ErreurNeige::ParametreHorsPlage(nom)));
        }
    }
    // le premier dans l'ordre
    let mut deux = bon;
    deux.gamma = Fixe(-1);
    deux.theta = Fixe(9 * un);
    assert_eq!(deux.verifier(), Err(ErreurNeige::ParametreHorsPlage("theta")));
    deux.rho = Fixe(2 * un);
    assert_eq!(deux.verifier(), Err(ErreurNeige::ParametreHorsPlage("rho")));
}

/// juste : les refus de `essayer_depuis_cases` (V-G1-21), dans l'ordre de la doc : rayon, réglages, nombre de cases (`NombreDeCases`), puis la PREMIÈRE
/// case (ordre canonique) qui viole un invariant (`EtatInvalide(indice)`) : une masse négative (`b`, `c` ou `d`), une case rattachée avec `b` ou `d` non nul
/// (mais `c` quelconque est permis), une case non rattachée sans voisine rattachée avec `b` ou `c` non nul (mais `d` quelconque est permis ; une case
/// de bord peut avoir `b`, `c` et `d`) ; enfin `MasseTropGrande` si la somme des `brut` est `2^56` ou plus (et un état de masse `2^56 - 1` est permis, et un pas dessus ne
/// panique pas). Jamais de panique.
///
/// Ne couvre pas : les refus de la construction depuis l'état de départ (voir `refus_de_la_construction`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn refus_des_etats() {
    let p = p(FIG9_RHO_050);
    let rayon = 2_u32;
    let n = cases(2).len();
    let vide: Vec<Case> = (0..n).map(|_| case_de([0, 0, 0, 0])).collect();
    let indice = |c: (i32, i32)| cases(2).iter().position(|&x| x == c).unwrap();
    let refuse = |liste: &[Case]| Neige::essayer_depuis_cases(rayon, p, liste).err();
    assert!(Neige::essayer_depuis_cases(rayon, p, &vide).is_ok(), "l'état vide est permis (ni germe ni vapeur)");
    // l'ordre des refus
    assert_eq!(Neige::essayer_depuis_cases(0, p, &vide).err(), Some(ErreurNeige::RayonHorsBorne));
    let mut faux = p;
    faux.kappa = Fixe(-1);
    assert_eq!(
        Neige::essayer_depuis_cases(rayon, faux, &vide[1..]).err(),
        Some(ErreurNeige::ParametreHorsPlage("kappa"))
    );
    assert_eq!(refuse(&vide[1..]), Some(ErreurNeige::NombreDeCases));
    let mut trop = vide.clone();
    trop.push(case_de([0, 0, 0, 0]));
    assert_eq!(refuse(&trop), Some(ErreurNeige::NombreDeCases));
    // les invariants
    for (nom, champ) in [("b", 1), ("c", 2), ("d", 3)] {
        let mut liste = vide.clone();
        let mut v = [0, 0, 0, 0];
        v[champ] = -1;
        liste[7] = case_de(v);
        assert_eq!(refuse(&liste), Some(ErreurNeige::EtatInvalide(7)), "{nom} négatif");
    }
    let centre = indice((0, 0));
    let voisine = indice((1, 0));
    let loin = indice((-2, 0));
    let mut liste = vide.clone();
    liste[centre] = case_de([1, 0, UN, 0]);
    assert!(refuse(&liste).is_none(), "un germe seul est permis");
    liste[centre] = case_de([1, 5, UN, 0]);
    assert_eq!(refuse(&liste), Some(ErreurNeige::EtatInvalide(centre)), "rattachée avec b");
    liste[centre] = case_de([1, 0, UN, 5]);
    assert_eq!(refuse(&liste), Some(ErreurNeige::EtatInvalide(centre)), "rattachée avec d");
    liste[centre] = case_de([1, 0, 0, 0]);
    assert!(refuse(&liste).is_none(), "une case rattachée de masse cristalline nulle est permise");
    liste[centre] = case_de([1, 0, UN, 0]);
    liste[voisine] = case_de([0, 5, 6, 7]);
    assert!(refuse(&liste).is_none(), "une case de bord peut avoir b, c et d");
    liste[loin] = case_de([0, 5, 0, 0]);
    assert_eq!(refuse(&liste), Some(ErreurNeige::EtatInvalide(loin)), "lointaine avec b");
    liste[loin] = case_de([0, 0, 5, 0]);
    assert_eq!(refuse(&liste), Some(ErreurNeige::EtatInvalide(loin)), "lointaine avec c");
    liste[loin] = case_de([0, 0, 0, 5]);
    assert!(refuse(&liste).is_none(), "une lointaine peut avoir de la vapeur");
    // la première case fautive dans l'ordre canonique
    liste[indice((0, -2))] = case_de([0, 3, 0, 0]);
    liste[loin] = case_de([0, 0, 5, 0]);
    assert_eq!(refuse(&liste), Some(ErreurNeige::EtatInvalide(indice((0, -2)).min(loin))));
    // la masse
    let mut grosse = vide.clone();
    grosse[loin] = case_de([0, 0, 0, 1_i128 << 56]);
    assert_eq!(refuse(&grosse), Some(ErreurNeige::MasseTropGrande));
    grosse[loin] = case_de([0, 0, 0, (1_i128 << 56) - 1]);
    let mut neige = Neige::essayer_depuis_cases(rayon, p, &grosse).expect("2^56 - 1 est permis");
    let avant = neige.masse_totale();
    neige.avancer(5);
    assert!(
        (neige.masse_totale() - avant).abs() <= 5 * 3 * 19 / 7 + 5,
        "la masse d'un état extrême reste conservée à l'arrondi près"
    );
    // plusieurs cases : la somme compte
    let mut deux = vide.clone();
    deux[loin] = case_de([0, 0, 0, 1_i128 << 55]);
    deux[indice((2, 0))] = case_de([0, 0, 0, 1_i128 << 55]);
    assert_eq!(refuse(&deux), Some(ErreurNeige::MasseTropGrande));
    // une masse tout juste permise répartie
    deux[indice((2, 0))] = case_de([0, 0, 0, (1_i128 << 55) - 1]);
    assert!(refuse(&deux).is_none());
}

/// juste : les réglages extrêmes PERMIS ne font jamais paniquer et restent exacts (V-G1-22) : les sept bornes basses, les sept bornes hautes, et quatre
/// mélanges (par exemple `α = 0` et `θ = 8` : toute case à 3 voisines se rattache ; `κ = 1` : tout gèle d'un coup ; `μ = γ = 1` : tout fond), sur
/// `R = 1` à 6, 120 pas, depuis l'état de départ : jamais de panique (l'arithmétique de `Fixe` panique au débordement), toutes les masses `>= 0`, et l'état est celui
/// du naïf à chaque pas.
///
/// Ne couvre pas : des états de départ extrêmes (voir `refus_des_etats`) ni de très grands rayons (le coût).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn reglages_extremes_sans_panique() {
    let jeux: [Jeu; 6] = [
        ["0", "1", "0", "0", "0", "0", "0"],
        ["1", "8", "1", "8", "1", "1", "1"],
        ["1", "1", "0", "8", "1", "0", "0"],
        ["1", "1", "1", "8", "0", "1", "1"],
        ["1", "1", "0", "0", "0", "1", "0"],
        ["0.5", "1", "0.5", "8", "0.5", "0.5", "1"],
    ];
    for jeu in jeux {
        for rayon in [1_u32, 2, 4, 6] {
            let p = p(jeu);
            let mut neige = Neige::essayer_new(rayon, p).expect("valide");
            let mut naif = Naif::initial(rayon as i32, &p);
            for t in 1..=120 {
                neige.pas();
                naif.pas();
                assert_eq!(etat_de(&neige), etat_du_naif(&naif), "{jeu:?} R={rayon} pas {t}");
                for c in neige.cases() {
                    assert!(c.b.0 >= 0 && c.c.0 >= 0 && c.d.0 >= 0, "{jeu:?} R={rayon} pas {t} : masse négative");
                }
            }
        }
    }
}

/// juste : `Parametres::lire` (V-G1-23) : les sept décimaux sont lus comme `flocon::lire_decimal` (exactement, arrondi une fois) : `"0.5"` donne
/// `brut = 2^31`, `"1"` donne `UN`, `"0.00005"` donne `round(0.00005 * 2^32) = 214748` ; le jeu de la figure 9 est celui de `FIG9_RHO_050` ; les bornes
/// incluses sont lues (`"1"` pour `ρ`, `"8"` pour `β`), et un seul réglage mal écrit (`".5"`, `"1e-3"`, `""`, `" 0.5"`, `"+0.5"`, `"-0.1"`, `"0.5."`) ou hors plage (`ρ = 1.01`,
/// `β = 0.99` ou `8.01`, `α = 1.01`, `θ = 8.01`, `κ = 1.01`, `μ = 2`, `γ = 1.0000001`) rend `None`, à n'importe laquelle des sept places.
///
/// Ne couvre pas : la directive `neige` (voir `etape_g1_scenario`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn lire_des_reglages() {
    let lire = |j: [&str; 7]| Parametres::lire(j[0], j[1], j[2], j[3], j[4], j[5], j[6]);
    assert_eq!(lire(FIG9_RHO_050), Some(p(FIG9_RHO_050)));
    assert_eq!(lire(FIG13_MILIEU), Some(p(FIG13_MILIEU)));
    let p9 = lire(FIG9_RHO_050).expect("valide");
    assert_eq!((p9.rho.0, p9.beta.0, p9.gamma.0), (1 << 31, 5583457485, 214748));
    let hautes = lire(["1", "8", "1", "8", "1", "1", "1"]).expect("les bornes hautes sont permises");
    assert_eq!((hautes.rho, hautes.beta, hautes.theta), (Fixe::UN, Fixe::entier(8), Fixe::entier(8)));
    let basses = lire(["0", "1", "0", "0", "0", "0", "0"]).expect("les bornes basses sont permises");
    assert_eq!((basses.rho, basses.beta, basses.gamma), (Fixe::ZERO, Fixe::UN, Fixe::ZERO));
    // mal écrit, à n'importe laquelle des sept places
    for place in 0..7 {
        for valeur in [".5", "1e-3", "", " 0.5", "+0.5", "-0.1", "0.5.", "0,5", "0.99999999999999999999"] {
            let mut j = FIG9_RHO_050;
            j[place] = valeur;
            assert_eq!(lire(j), None, "position {place} = {valeur:?}");
        }
    }
    // bien écrit mais hors plage
    let hors_plage: [(usize, &str); 12] = [
        (0, "1.01"),
        (0, "2"),
        (1, "0.99"),
        (1, "8.01"),
        (1, "0"),
        (2, "1.01"),
        (3, "8.01"),
        (3, "9"),
        (4, "1.01"),
        (5, "2"),
        (6, "1.0000001"),
        (6, "3"),
    ];
    for (place, valeur) in hors_plage {
        let mut j = FIG9_RHO_050;
        j[place] = valeur;
        assert_eq!(lire(j), None, "position {place} = {valeur}");
    }
}

/// juste : de GRANDES masses, contre le naïf (V-G1-24, ajouté par le juge) : `R = 2`, chaque case porte une vapeur de `2^51` plus un tirage (congruentiel linéaire, graines 1 à 20, cinq
/// jeux de réglages, dont un à `β = 8`, `κ = 0,9999999` et `γ = 0,3` : les voisines du germe gèlent presque toute leur vapeur sans se rattacher, et les produits `κ d1` et `γ c` sont de l'ordre de `2^83`) ; pour les graines paires un germe est posé au centre et ses six voisines reçoivent une petite masse de bord et une glace de l'ordre de `2^50`. La masse totale est entre
/// `2^55` et `2^56` (le plus grand état permis) : la somme de sept vapeurs dépasse `2^53`, donc une division ou un produit qui passerait par un `f64` (53 bits) perd des unités, et un entier de 64 bits
/// mal employé déborde. Six pas : l'état est celui du naïf (qui calcule en `i128`), à chaque pas, sans panique.
///
/// Ne couvre pas : un rayon plus grand ; les masses ordinaires (voir les autres tests).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn naif_avec_de_grandes_masses() {
    let jeux = [FIG9_RHO_050, JEU_B, JEU_C, JEU_E, ["0.9", "8", "0.5", "0.5", "0.9999999", "0.3", "0.3"]];
    for graine in 1..=20_u64 {
        let jeu = jeux[(graine as usize) % jeux.len()];
        let mut g = Lcg(graine);
        let mut etat = etat_vide(2);
        for v in etat.values_mut() {
            v[3] = (1_i128 << 51) + g.jusqu_a(1 << 40) as i128;
        }
        if graine % 2 == 0 {
            etat.insert((0, 0), [1, 0, UN, 0]);
            for (dq, dr) in DIRECTIONS {
                let b = g.jusqu_a(1 << 20) as i128;
                let c = g.jusqu_a(1 << 50) as i128;
                let d = etat[&(dq, dr)][3];
                etat.insert((dq, dr), [0, b, c, d]);
            }
        }
        let (neige, naif) = depuis_un_etat(2, &p(jeu), &etat);
        let masse = neige.masse_totale();
        assert!(masse > 1 << 55 && masse < 1 << 56, "graine {graine} : masse {masse}");
        comparer(neige, naif, 6, &format!("grandes masses, graine {graine}, {jeu:?}"));
    }
}
