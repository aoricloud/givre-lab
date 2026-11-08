//! Étape 3 : l'état d'un `Flocon` (état initial, `depuis_masses`, ordre des cases) et la classification des cellules
//! (gelée, de bord, non réceptive). Valeurs attendues : `references/etape-3-valeurs.md`, identifiants `V-ETAT-xx`.

mod etape_3_commun;

use etape_3_commun::{Naif, brut_de, cases, masses_formule, parametres};
use givre_lab::flocon::{Etat, Flocon, Parametres};
use givre_lab::math::Fixe;

fn paniquer<T>(f: impl FnOnce() -> T + std::panic::UnwindSafe) -> bool {
    std::panic::catch_unwind(f).is_err()
}

/// juste : l'état initial (V-ETAT-01), d'après la doc de `Flocon::new` : toutes les masses valent `β`, sauf la case `(0, 0)`
/// qui vaut `Fixe::UN` ; temps 0 ; rayon et paramètres rendus tels quels ; une seule case gelée (le centre), les six voisins du
/// centre de bord, tous les autres non réceptifs ; rayon du cristal 0, aucune pointe, compacité `UN` (1 case sur 1).
/// Vérifié pour trois rayons (1, 3, 12) et trois `β` (0, 0.3, 0.9).
///
/// Ne couvre pas : le comportement d'un pas (voir `etape_3_pas`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn etat_initial() {
    for rayon in [1_u32, 3, 12] {
        for beta in ["0", "0.3", "0.9"] {
            let p = parametres("1", beta, "0.01");
            let f = Flocon::new(rayon, p);
            assert_eq!(f.rayon(), rayon);
            assert_eq!(f.parametres(), p);
            assert_eq!(f.temps(), 0);
            let toutes = cases(rayon as i32);
            assert_eq!(f.masses().len(), toutes.len());
            for &(q, r) in &toutes {
                let attendu = if (q, r) == (0, 0) { Fixe::UN } else { p.beta };
                assert_eq!(f.masse(q, r), attendu, "R={rayon} beta={beta} case ({q}, {r})");
            }
            assert_eq!(f.gelees(), 1);
            assert_eq!(f.rayon_cristal(), 0);
            assert_eq!(f.pointes(), 0);
            assert_eq!(f.compacite(), Fixe::UN);
            assert_eq!(f.defaut_rotation(), 0);
            assert_eq!(f.defaut_reflexion(), 0);
            let voisins_du_centre = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];
            for &(q, r) in &toutes {
                let attendu = if (q, r) == (0, 0) {
                    Etat::Gelee
                } else if voisins_du_centre.contains(&(q, r)) {
                    Etat::DeBord
                } else {
                    Etat::NonReceptive
                };
                assert_eq!(f.etat(q, r), attendu, "R={rayon} case ({q}, {r})");
            }
        }
    }
}

/// juste : `depuis_masses` range les masses dans l'ordre canonique (V-ETAT-02) : la masse de la i-ième case de l'ordre `r`
/// croissant puis `q` croissant est `masses[i]`, `masses()` rend la même liste, le temps est 0, rayon et paramètres sont ceux
/// donnés ; les masses de la couronne sont gardées telles quelles (pas de remise à `β` avant le premier pas). L'énumération
/// de référence est réécrite dans le test, pour `R` de 1 à 4 ; chaque masse est distincte (`i + 1` fois un `brut` fixe).
///
/// Ne couvre pas : les masses refusées (voir `depuis_masses_refuse`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn depuis_masses_ordre_canonique() {
    let p = parametres("0.8", "0.25", "0.05");
    for rayon in 1_u32..=4 {
        let toutes = cases(rayon as i32);
        let masses: Vec<Fixe> = (0..toutes.len() as i64).map(|i| Fixe(1_000_003 * (i + 1))).collect();
        let f = Flocon::depuis_masses(rayon, p, &masses);
        assert_eq!(f.rayon(), rayon);
        assert_eq!(f.parametres(), p);
        assert_eq!(f.temps(), 0);
        for (i, &(q, r)) in toutes.iter().enumerate() {
            assert_eq!(f.masse(q, r), masses[i], "R={rayon} case ({q}, {r}) d'indice {i}");
        }
        assert_eq!(f.masses(), masses);
    }
}

/// juste : `depuis_masses` panique si le nombre de masses n'est pas celui des cases (trop peu, trop), si une masse est négative
/// (`Fixe(-1)`), si une masse vaut `2^60` en `brut` ou plus, si le rayon est 0 ou si les paramètres sont invalides ; et il accepte
/// `2^60 - 1` et `0` (V-ETAT-03), comme le dit sa doc.
///
/// Ne couvre pas : `Flocon::new` (voir `new_refuse_les_parametres_invalides`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn depuis_masses_refuse() {
    let p = parametres("1", "0.3", "0.01");
    let bonnes = vec![Fixe::ZERO; 19]; // R = 2 : 19 cases
    assert!(!paniquer(|| Flocon::depuis_masses(2, p, &bonnes)));
    assert!(paniquer(|| Flocon::depuis_masses(2, p, &bonnes[..18])));
    let trop = vec![Fixe::ZERO; 20];
    assert!(paniquer(|| Flocon::depuis_masses(2, p, &trop)));
    let mut negative = bonnes.clone();
    negative[5] = Fixe(-1);
    assert!(paniquer(|| Flocon::depuis_masses(2, p, &negative)));
    let mut enorme = bonnes.clone();
    enorme[5] = Fixe(1 << 60);
    assert!(paniquer(|| Flocon::depuis_masses(2, p, &enorme)));
    let mut limite = bonnes.clone();
    limite[5] = Fixe((1 << 60) - 1);
    assert!(!paniquer(|| Flocon::depuis_masses(2, p, &limite)));
    assert!(paniquer(|| Flocon::depuis_masses(0, p, &[Fixe::ZERO])));
    let invalides = Parametres { beta: Fixe::UN, ..p };
    assert!(paniquer(|| Flocon::depuis_masses(2, invalides, &bonnes)));
}

/// juste : une configuration à la main sur l'hexagone de rayon 2 (19 cases), toutes les masses à 0.2 sauf `(0,0) = 1` (gelée :
/// égalité comprise), `(2,-2) = 1 + 5 ulp` (gelée, coin de la couronne : elle n'a que 3 voisins), `(-2,1) = 1 - 1 ulp` (PAS gelée,
/// et ses voisines n'en deviennent pas de bord). Classes attendues, écrites en toutes lettres : gelées `(0,0)` et `(2,-2)` ; de
/// bord les six voisins du centre `(1,0) (0,1) (-1,1) (-1,0) (0,-1) (1,-1)` et les voisins de `(2,-2)` dans l'hexagone `(2,-1)`
/// `(1,-1)` `(1,-2)` ; non réceptives les 9 autres cases (V-ETAT-04). Un `>` pour `>=`, un voisin de plus ou de moins, un voisin
/// hors de l'hexagone traité comme gelé, ou une masse `UN - 1` traitée comme gelée échouent ici.
///
/// Ne couvre pas : les grands états (voir `classes_comme_le_naif`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn classes_a_la_main() {
    let p = parametres("1", "0.3", "0.01");
    let toutes = cases(2);
    let mut masses = vec![Fixe::depuis_f64(0.2); toutes.len()];
    let place = |masses: &mut Vec<Fixe>, c: (i32, i32), v: Fixe| {
        let i = toutes.iter().position(|&x| x == c).unwrap();
        masses[i] = v;
    };
    place(&mut masses, (0, 0), Fixe::UN);
    place(&mut masses, (2, -2), Fixe(Fixe::UN.0 + 5));
    place(&mut masses, (-2, 1), Fixe(Fixe::UN.0 - 1));
    let f = Flocon::depuis_masses(2, p, &masses);
    let gelees = [(0, 0), (2, -2)];
    let de_bord = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1), (2, -1), (1, -2)];
    let mut non_receptives = 0;
    for &c in &toutes {
        let attendu = if gelees.contains(&c) {
            Etat::Gelee
        } else if de_bord.contains(&c) {
            Etat::DeBord
        } else {
            non_receptives += 1;
            Etat::NonReceptive
        };
        assert_eq!(f.etat(c.0, c.1), attendu, "case {c:?}");
    }
    assert_eq!(non_receptives, 9);
    assert_eq!(f.gelees(), 2);
    assert_eq!(f.rayon_cristal(), 2);
    // les deux cases gelées ne sont pas voisines : chacune est une pointe seulement si elle a un voisin gelé : aucune ici
    assert_eq!(f.pointes(), 0);
}

/// juste : la classification de `etat` égale celle du simulateur naïf du test sur 30 états (V-ETAT-05) : des masses de formule
/// `rapport((a i + b) % c, d)` (valeurs de 0 à 1 par pas de 1/c) où le seuil `UN` est atteint ou frôlé (des masses `UN - 1`,
/// `UN`, `UN + 1` sont posées en plusieurs cases, couronne comprise), pour `R` de 1 à 6. Le naïf cherche les voisins dans un
/// dictionnaire de cases (une case hors de l'hexagone n'y est pas).
///
/// Ne couvre pas : l'effet de la classe sur un pas.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn classes_comme_le_naif() {
    let p = parametres("1", "0.3", "0.01");
    let mut verifies = 0;
    for rayon in 1_i32..=6 {
        for (a, b, c) in [(37, 11, 61), (53, 7, 97), (29, 5, 59), (11, 3, 41), (7, 1, 23)] {
            let toutes = cases(rayon);
            let mut masses = masses_formule(rayon, a, b, c, c, &[]);
            // quelques masses sur le seuil, à des cases qui dépendent de la formule
            for (k, valeur) in [
                (2_usize, Fixe::UN),
                (7, Fixe(Fixe::UN.0 - 1)),
                (11, Fixe(Fixe::UN.0 + 1)),
                (toutes.len() - 1, Fixe::UN),
            ] {
                if k < toutes.len() {
                    masses[k] = valeur;
                }
            }
            let f = Flocon::depuis_masses(rayon as u32, p, &masses);
            let naif = Naif::depuis_brut(rayon, p, &brut_de(&masses));
            for &(q, r) in &toutes {
                let attendu = match naif.classe((q, r)) {
                    'G' => Etat::Gelee,
                    'B' => Etat::DeBord,
                    _ => Etat::NonReceptive,
                };
                assert_eq!(f.etat(q, r), attendu, "R={rayon} formule ({a}, {b}, {c}) case ({q}, {r})");
            }
            let gelees_naif = toutes.iter().filter(|&&c| naif.classe(c) == 'G').count();
            assert_eq!(f.gelees(), gelees_naif);
            verifies += 1;
        }
    }
    assert!(verifies >= 30);
}

/// juste : `masse` et `etat` paniquent hors de l'hexagone (V-ETAT-06), comme `Hexagone::index` : `(3, 0)` et `(0, -3)` et `(2, 1)`
/// pour `R = 2` ; et acceptent le bord `(2, 0)`.
///
/// Ne couvre pas : le contenu des cases.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn hors_de_l_hexagone() {
    let f = Flocon::new(2, parametres("1", "0.3", "0.01"));
    for (q, r) in [(3, 0), (0, -3), (2, 1), (-1, -2)] {
        assert!(paniquer(|| f.masse(q, r)), "masse({q}, {r})");
        assert!(paniquer(|| f.etat(q, r)), "etat({q}, {r})");
    }
    assert!(!paniquer(|| f.masse(2, 0)));
    assert!(!paniquer(|| f.etat(2, 0)));
}
