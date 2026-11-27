//! Étape P3 : le contour `φ = 0` d'un quart, ligne par ligne (`givre_lab::pointe::contour`), sur des champs faits à la main dont la
//! réponse est exacte. Valeurs attendues : `references/etape-p3-valeurs.md`, identifiants `V-P3-CONTOUR-xx`.

mod etape_p3_commun;

use etape_p3_commun::*;
use givre_lab::dendrite::Domaine;
use givre_lab::pointe::{ErreurPointe, contour};

/// juste : (V-P3-CONTOUR-01) le contour de la doc à la main. (a) La parabole exacte `x = 10 - y² / 8` (`xs_parabole_exacte`, quart de
/// 24 × 24 cases au pas `dx = 0,5`, rampes de demi-largeur `w = 2` : tout est dyadique, l'interpolation est exacte) : la ligne `i` croise
/// en `10 - ((i + 0,5) · 0,5)² / 8` exactement (bits égaux) pour `i` de 0 à 17, et `None` pour les lignes 18 à 23 (aucune case positive) ;
/// le résultat a 24 valeurs ; la ligne 0 est, bit pour bit, `Dendrite::position_pointe()` (9,9921875). (b) Un front droit (`x = 7,25`
/// sur les 16 lignes, `dx = 0,5`) : 16 fois `Some(7,25)`. (c) Les inégalités de la doc, sur un quart de 4 × 4 au pas 0,5, une ligne à la
/// fois : `[1 ; 0 ; -1 ; -1]` croise de `j = 0` à `j = 1` (un zéro exact est `<= 0`, fraction 1) : `0,75` ; `[1 ; 0,5 ; -0,25 ; -1]` :
/// `0,75 + 0,5 · (0,5 / 0,75)` ; `[1 ; -1 ; 1 ; -1]` : le PREMIER croisement, `0,5` ; `[0 ; -1 ; -1 ; -1]` et `[0 ; 0 ; -1 ; -1]` :
/// `None` (l'inégalité de gauche est stricte) ; `[1 ; 0 ; 0 ; -1]` : `0,75` ; `[1 ; 1 ; 1 ; 1]` et `[-1 ; -1 ; -1 ; -1]` : `None`.
/// (d) Un domaine entier : `Err(DomaineEntier)`, même pour un champ qui croise. (e) L'ordre des opérations : sur 300 champs tirés (`n` de
/// 3 à 9, `dx` de 0,1 à 1,3, `φ` dans `[-1, 1]`), le contour est, bit pour bit, celui de `contour_naif` (la doc écrite dans les tests), et
/// l'interpolation écrite depuis la case de droite (la faute `InterpolationParLaDroite`, la même en réels) donne d'autres bits sur au
/// moins 30 lignes.
///
/// Ne couvre pas : un état né d'un calcul (voir `mesure_egale_au_naif_bit_pour_bit`) ; les lignes de longueur autre que `n`.
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn contour_a_la_main() {
    // (a) la parabole exacte
    let n = 24;
    let xs = xs_parabole_exacte(n);
    let d = dendrite_de_champ(&champ_par_lignes(n, 0.5, &xs, 2.0), n, 0.5, Domaine::Quart);
    let c = contour(&d).unwrap();
    assert_eq!(c.len(), n);
    for (i, x) in c.iter().enumerate() {
        let y = (i as f64 + 0.5) * 0.5;
        let attendu = if i <= 17 { Some(10.0 - (y * y) / 8.0) } else { None };
        assert_eq!(x.map(f64::to_bits), attendu.map(f64::to_bits), "ligne {i}");
    }
    assert_eq!(c[0], Some(9.9921875));
    assert_eq!(c[0].map(f64::to_bits), d.position_pointe().map(f64::to_bits));
    // (b) le front droit
    let d = dendrite_de_champ(&champ_par_lignes(16, 0.5, &[7.25; 16], 2.0), 16, 0.5, Domaine::Quart);
    assert_eq!(contour(&d).unwrap(), vec![Some(7.25); 16]);
    // (c) les inégalités, une ligne à la fois (les trois autres lignes sont sans croisement)
    let cas: [([f64; 4], Option<f64>); 8] = [
        ([1.0, 0.0, -1.0, -1.0], Some(0.75)),
        ([1.0, 0.5, -0.25, -1.0], Some(0.75 + 0.5 * (0.5 / 0.75))),
        ([1.0, -1.0, 1.0, -1.0], Some(0.5)),
        ([0.0, -1.0, -1.0, -1.0], None),
        ([0.0, 0.0, -1.0, -1.0], None),
        ([1.0, 0.0, 0.0, -1.0], Some(0.75)),
        ([1.0, 1.0, 1.0, 1.0], None),
        ([-1.0, -1.0, -1.0, -1.0], None),
    ];
    for (ligne, attendu) in cas {
        for rang in 0..4 {
            let mut phi = vec![-1.0; 16];
            phi[rang * 4..rang * 4 + 4].copy_from_slice(&ligne);
            let c = contour(&dendrite_de_champ(&phi, 4, 0.5, Domaine::Quart)).unwrap();
            for (i, x) in c.iter().enumerate() {
                let voulu = if i == rang { attendu } else { None };
                assert_eq!(
                    x.map(f64::to_bits),
                    voulu.map(f64::to_bits),
                    "ligne {ligne:?} au rang {rang}, ligne lue {i}"
                );
            }
        }
    }
    // (d) le domaine entier
    let d = dendrite_de_champ(&champ_par_lignes(8, 0.5, &[2.0; 8], 2.0), 8, 0.5, Domaine::Entier);
    assert_eq!(contour(&d), Err(ErreurPointe::DomaineEntier));
    // (e) l'ordre des opérations
    let mut g = Lcg(3_141);
    let (mut lignes, mut autre_ordre_differe) = (0, 0);
    for _ in 0..300 {
        let n = 3 + (g.suivant() * 7.0) as usize;
        let dx = g.entre(0.1, 1.3);
        let phi: Vec<f64> = (0..n * n).map(|_| g.entre(-1.0, 1.0)).collect();
        let d = dendrite_de_champ(&phi, n, dx, Domaine::Quart);
        let c = contour(&d).unwrap();
        let attendu = contour_naif(&phi, n, dx, None);
        assert_eq!(
            c.iter().map(|x| x.map(f64::to_bits)).collect::<Vec<_>>(),
            attendu.iter().map(|x| x.map(f64::to_bits)).collect::<Vec<_>>(),
            "n = {n}, dx = {dx}"
        );
        let droite = contour_naif(&phi, n, dx, Some(Faute::InterpolationParLaDroite));
        for (a, b) in attendu.iter().zip(&droite) {
            if let (Some(a), Some(b)) = (a, b) {
                lignes += 1;
                autre_ordre_differe += usize::from(a.to_bits() != b.to_bits());
            }
        }
    }
    assert!(
        lignes >= 500 && autre_ordre_differe >= 30,
        "{lignes} croisements, {autre_ordre_differe} où l'autre ordre diffère"
    );
}
