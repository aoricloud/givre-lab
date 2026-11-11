//! Étape G1 : la masse. Sans bruit, la masse totale `M = Σ (b + c + d)` est conservée par les sous-pas 2, 3 et 4 EXACTEMENT et par le sous-pas 1 à l'arrondi de la division par 7
//! près : à chaque pas, `|M' - M|` est au plus `3 C / 7` unités du dernier chiffre (`C` = les cases non rattachées au début du pas). Valeurs attendues :
//! `references/etape-g1-valeurs.md`, identifiants `V-G1-MASSE-xx`.

mod etape_g1_commun;

use etape_g1_commun::*;
use givre_lab::neige::Neige;

/// Le nombre de cases non rattachées d'une neige.
fn non_rattachees(neige: &Neige) -> i128 {
    (neige.cases().len() - neige.cases_rattachees()) as i128
}

/// juste : la borne de l'arrondi, à CHAQUE pas (V-G1-MASSE-01) : depuis l'état de départ (masse `1 + (cases - 1) ρ` en `brut`, exacte au temps 0), pour sept jeux de réglages et
/// des rayons de 6 à 10, 150 à 400 pas, `7 |M' - M| <= 3 C` à chaque pas où `C` est le nombre de cases non rattachées au début du pas (la division par 7 se trompe d'au plus 3/7 d'unité
/// par case, et c'est la seule perte ou création de masse de la doc). Une fonte qui oublie de décompter `c`, une part `κ` comptée deux fois, un rattachement qui perd `b`, une vapeur
/// créée par le mur ou perdue à travers lui dépassent la borne sur ces longues suites. Le test ne fixe aucune tolérance empirique : la borne vient de la formule de l'arrondi.
///
/// Chaque suite doit avoir fait grandir le cristal (un pas qui ne ferait rien tiendrait la borne à vide).
///
/// Ne couvre pas : une création ou une perte de masse plus petite que la borne (voir `conservation_exacte_si_la_division_est_exacte`, qui est exact).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn borne_de_l_arrondi_a_chaque_pas() {
    let jeux: [(u32, Jeu, u64); 7] = [
        (10, FIG9_RHO_050, 400),
        (10, FIG9_RHO_040, 400),
        (8, FIG13_MILIEU, 400),
        (8, JEU_B, 300),
        (6, JEU_C, 150),
        (8, JEU_D, 300),
        (6, JEU_E, 200),
    ];
    for (rayon, jeu, pas) in jeux {
        let reglages = p(jeu);
        let mut neige = Neige::essayer_new(rayon, reglages).expect("neige valide");
        let cases_total = neige.cases().len() as i128;
        assert_eq!(neige.masse_totale(), UN + (cases_total - 1) * i128::from(reglages.rho.0), "masse au temps 0");
        for t in 1..=pas {
            let avant = neige.masse_totale();
            let c = non_rattachees(&neige);
            neige.pas();
            let ecart = (neige.masse_totale() - avant).abs();
            assert!(
                7 * ecart <= 3 * c,
                "R={rayon} {jeu:?} pas {t} : écart {ecart} unités pour {c} cases non rattachées"
            );
        }
        assert_eq!(neige.temps(), pas, "R={rayon} {jeu:?} : le temps");
        assert!(
            neige.cases_rattachees() > 1,
            "R={rayon} {jeu:?} : le cristal n'a pas grandi, la borne serait tenue à vide"
        );
    }
}

/// juste : la conservation EXACTE (V-G1-MASSE-02) : si toutes les vapeurs sont des multiples de 7 `brut` (états tirés par un congruentiel linéaire, graines 1 à 60, rayon 4 à 6, des
/// réglages variés), la division par 7 du sous-pas 1 est exacte et le pas conserve la masse exactement : `M' = M`, en un pas (le gel, le rattachement et la fonte reprennent leurs
/// arrondis sur le reste). Une faute qui crée ou détruit la moindre unité de masse dans les sous-pas 2 à 4 échoue ici (la fonte appliquée à `c` et non à `b`, le gel d'une part `κ` comptée
/// deux fois, `b` perdu au rattachement, `c` non reçu).
///
/// Ne couvre pas : l'arrondi de la division par 7 (voir `borne_de_l_arrondi_a_chaque_pas`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn conservation_exacte_si_la_division_est_exacte() {
    let jeux = [FIG9_RHO_050, FIG13_MILIEU, JEU_B, JEU_C, JEU_D, JEU_E, ["0.5", "1", "0.5", "1", "0.5", "0.5", "0.5"]];
    let mut avec_rattachement = 0;
    let mut avec_vapeur = 0;
    for graine in 1..=60_u64 {
        let rayon = 4 + (graine % 3) as u32;
        let jeu = jeux[(graine as usize) % jeux.len()];
        let etat = etat_au_hasard(rayon as i32, graine, true);
        let (mut neige, _) = depuis_un_etat(rayon, &p(jeu), &etat);
        let avant = neige.masse_totale();
        let rattachees = neige.cases_rattachees();
        neige.pas();
        assert_eq!(neige.masse_totale(), avant, "graine {graine} R={rayon} {jeu:?}");
        avec_rattachement += usize::from(neige.cases_rattachees() > rattachees);
        avec_vapeur += usize::from(neige.cases().iter().any(|c| !c.a && c.d.0 > 0));
    }
    assert!(avec_rattachement >= 10, "trop peu d'états où un rattachement a lieu : {avec_rattachement}");
    assert!(avec_vapeur > 30, "trop peu d'états où la vapeur est non nulle : {avec_vapeur}");
}

/// juste : sur une très longue suite, la masse ne dérive pas plus que la borne (V-G1-MASSE-03) : `R = 5`, `FIG9_RHO_050` (le cristal grandit puis le mur le limite), 4000 pas : à chaque pas
/// la borne `7 |ΔM| <= 3 C`, et au total `|M - M_0|` est au plus la somme des bornes par pas ; ce n'est pas une tolérance empirique. La masse de départ est
/// celle de la doc (`1 + 90 ρ`) et le cristal a grandi à la fin (un code qui ne ferait rien, ou dont la masse serait nulle, ne passe pas à vide).
///
/// Ne couvre pas : une dérive plus petite que la borne (la dérive mesurée est bien plus petite : voir `MODELE.md`).
#[test]
#[cfg_attr(not(feature = "etape-g1"), ignore)]
fn derive_longue_sous_la_borne() {
    let mut neige = Neige::essayer_new(5, p(FIG9_RHO_050)).expect("neige valide");
    let depart = neige.masse_totale();
    assert_eq!(depart, UN + 90 * (UN / 2), "masse au temps 0 : le germe et 90 cases de vapeur 1/2");
    let mut borne_totale = 0_i128;
    for t in 1..=4000_u64 {
        let avant = neige.masse_totale();
        let c = non_rattachees(&neige);
        neige.pas();
        assert!(7 * (neige.masse_totale() - avant).abs() <= 3 * c, "pas {t}");
        borne_totale += 3 * c;
        assert!(7 * (neige.masse_totale() - depart).abs() <= borne_totale, "pas {t} : dérive totale");
    }
    assert_eq!(neige.temps(), 4000);
    assert!(neige.cases_rattachees() > 1, "le cristal n'a pas grandi : la borne serait tenue à vide");
}
