//! Étape P3 : des fautes plausibles de la mesure, qui DOIVENT être vues. Chaque faute est une variante de la mesure naïve de
//! `etape_p3_commun` (voir `Faute` : `y` au bord de la case, la pointe lue sur la mauvaise ligne, la profondeur stricte, la règle des
//! lignes sans arrêt, le dernier croisement, les sommes brutes, le signe de la parabole, `ρ` pour `ρ²` dans `σ*`, la vitesse du
//! sommet, l'interpolation par la droite). Valeurs attendues : `references/etape-p3-valeurs.md`, identifiant `V-P3-FAUTE-01`.

mod etape_p3_commun;

use etape_p3_commun::*;
use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};
use givre_lab::pointe::{
    ErreurPointe, Pointe, constante_de_selection, contour, mesurer_pointe, parabole, vitesse_de_pointe,
};

/// La vue « à la main » : vrai si la mesure naïve fautive se trompe sur au moins un des cas exacts des tests à la main (la parabole
/// exacte à la profondeur 1,75 : 8 lignes, `(10, 4)`, position 9,9921875 ; le cercle de rayon 8 à la profondeur 2 : 11 lignes, rayon
/// 7,100 258 720 076 413 à `1e-10` ; la ligne 2 sans croisement : `TropPeuDeLignes(2)` ; la ligne 3 enfoncée : 3 lignes ; la vitesse
/// `(1 ; 2) -> (3 ; 5)` avec des sommets 7 et 9 : 1,5 ; `σ*(0,5 ; 1 ; 4 ; 0,125) = 0,5` ; la ligne `[1 ; -1 ; 1 ; -1 ; -1 ; -1]` au pas
/// 0,5 : 0,5).
fn vue_a_la_main(f: Option<Faute>) -> bool {
    let n = 24;
    let xs = xs_parabole_exacte(n);
    let mesure = |xs: &[f64], h: f64| {
        mesure_naive(&champ_par_lignes(n, 0.5, xs, 2.0), n, 0.5, 0.0, false, h, f)
            .map(|p| (p.lignes, p.sommet, p.rayon, p.position))
    };
    let mut fausse = mesure(&xs, 1.75) != Ok((8, 10.0, 4.0, 9.9921875));
    let cercle: Vec<f64> = (0..n)
        .map(|i| {
            let y = (i as f64 + 0.5) * 0.5;
            if y < 8.0 { (64.0 - y * y).sqrt() } else { f64::NAN }
        })
        .collect();
    fausse |= !matches!(mesure(&cercle, 2.0), Ok((11, _, r, _)) if ((r - 7.100_258_720_076_413) / 7.1).abs() < 1e-10);
    let mut a = xs.clone();
    a[2] = f64::NAN;
    fausse |= mesure(&a, 1.75) != Err(ErreurPointe::TropPeuDeLignes(2));
    let mut b = xs.clone();
    b[3] -= 5.0;
    fausse |= !matches!(mesure(&b, 1.75), Ok((3, _, _, _)));
    let p = |temps, position, sommet| Pointe { temps, position, lignes: 5, sommet, rayon: 3.0 };
    fausse |= vitesse_naive(&p(1.0, 2.0, 7.0), &p(3.0, 5.0, 9.0), f) != Ok(1.5);
    fausse |= selection_naive(0.5, 1.0, 4.0, 0.125, f) != Ok(0.5);
    fausse |= croisement_naif(&[1.0, -1.0, 1.0, -1.0, -1.0, -1.0], 0.5, f) != Some(0.5);
    fausse
}

/// La vue « naïf » : vrai si, sur trois graines rondes (`ε` = 0,02 ; 0,05 ; 0,0665, `dx` = 0,8 ; 0,6 ; 0,5, quart de 36, rayon 6,
/// `Delta = 0,65`, relevés après 100, 300 et 600 pas, profondeurs 1 ; 2 ; 3,7), la mesure naïve fautive diffère au bit de la
/// bibliothèque sur au moins un contour, une mesure, une vitesse ou un `σ*` (c'est ce que voit `etape_p3_naif.rs`).
fn vue_naive(f: Option<Faute>) -> bool {
    let mut differe = false;
    for (epsilon, dx) in [(0.02, 0.8), (0.05, 0.6), (0.0665, 0.5)] {
        let r0 = Reglages::depuis_capillarite(1.0, 0.5, 1.0, epsilon, dx, 1.0);
        let r = Reglages { dt: r0.dt_borne(0.65, ModeU::Joue), ..r0 };
        let n = (36.0 / dx) as usize;
        let mut d = Dendrite::graine_ronde(r, n, Domaine::Quart, 0.65, 6.0, ModeU::Joue).unwrap();
        let mut avant: [Option<(Pointe, Pointe)>; 3] = [None; 3];
        let mut fait = 0;
        for pas in [100_u64, 300, 600] {
            d.avancer(pas - fait).unwrap();
            fait = pas;
            let c: Vec<Option<u64>> = contour(&d).unwrap().iter().map(|x| x.map(f64::to_bits)).collect();
            let cn: Vec<Option<u64>> = contour_naif(d.phi(), n, dx, f).iter().map(|x| x.map(f64::to_bits)).collect();
            differe |= c != cn;
            for (k, h) in [1.0, 2.0, 3.7].into_iter().enumerate() {
                let (lib, naif) = (mesurer_pointe(&d, h), mesure_naive_de(&d, h, f));
                differe |= bits_mesure(&lib) != bits_mesure(&naif);
                if let (Ok(a), Ok(b)) = (lib, naif) {
                    if let Some((pa, pb)) = avant[k] {
                        let (va, vb) = (vitesse_de_pointe(&pa, &a), vitesse_naive(&pb, &b, f));
                        differe |= va.clone().map(f64::to_bits) != vb.clone().map(f64::to_bits);
                        if let (Ok(va), Ok(vb)) = (va, vb)
                            && va > 0.0
                            && vb > 0.0
                        {
                            let sa = constante_de_selection(0.5, 1.0, a.rayon, va).map(f64::to_bits);
                            differe |= sa != selection_naive(0.5, 1.0, b.rayon, vb, f).map(f64::to_bits);
                        }
                    }
                    avant[k] = Some((a, b));
                }
            }
        }
    }
    differe
}

/// La table attendue (mesurée sur le prototype) : `(faute, [à la main, naïf])`.
fn attendue() -> Vec<(Faute, [bool; 2])> {
    vec![
        (Faute::YAuBord, [true, true]),
        (Faute::MauvaiseLigne, [true, true]),
        // une ligne d'une dendrite réelle n'est jamais à la profondeur exacte : seuls les cas à la main (dyadiques) la voient
        (Faute::ProfondeurStricte, [true, false]),
        // sur une dendrite réelle, après la première ligne qui manque, aucune ligne ne revient dans la règle
        (Faute::SansArret, [true, false]),
        // la pointe d'une dendrite en temps court n'a qu'un croisement par ligne
        (Faute::DernierCroisement, [true, false]),
        // la même formule en réels : seuls les bits la voient
        (Faute::SommesBrutes, [false, true]),
        (Faute::SigneDeLaParabole, [true, true]),
        (Faute::RayonSimple, [true, true]),
        (Faute::VitesseDuSommet, [true, true]),
        // la même formule en réels, exacte sur les cas dyadiques : seuls les bits la voient
        (Faute::InterpolationParLaDroite, [false, true]),
    ]
}

/// juste : (V-P3-FAUTE-01) la table des fautes AFFIRMÉE EN ENTIER : pour chacune des dix fautes de `TOUTES_LES_FAUTES`, dans l'ordre, ses
/// deux vues (`[à la main, naïf]`, voir `vue_a_la_main` et `vue_naive`) sont exactement celles de `attendue()` (`assert_eq!` sur toute la
/// table), et la mesure naïve SANS faute n'est vue par aucune des deux (sinon les fautes ne prouveraient rien ; seul
/// `parabole` des huit points exacts touche ici la bibliothèque, et rend `(10, 4)`). Chaque
/// faute est vue par au moins une vue.
///
/// Ne couvre pas : les fautes que personne n'a imaginées ; des fautes combinées ; les fautes de la dendrite (les tests de P2).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn la_table_des_fautes_en_entier() {
    let points: Vec<(f64, f64)> = (0..8)
        .map(|i| {
            let y = (i as f64 + 0.5) * 0.5;
            (y, 10.0 - (y * y) / 8.0)
        })
        .collect();
    assert_eq!(parabole(&points), Ok((10.0, 4.0)));
    assert_eq!(
        [vue_a_la_main(None), vue_naive(None)],
        [false, false],
        "la mesure naïve sans faute devait passer les deux vues"
    );
    let vue: Vec<(Faute, [bool; 2])> =
        TOUTES_LES_FAUTES.iter().map(|&f| (f, [vue_a_la_main(Some(f)), vue_naive(Some(f))])).collect();
    assert_eq!(vue, attendue());
    assert!(vue.iter().all(|(_, v)| v[0] || v[1]));
}
