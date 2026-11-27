//! Étape P3 : le sens de variation de la pointe selon l'anisotropie, sur NOTRE modèle, en temps court, à deux pas de grille. **Ce
//! fichier ne reproduit aucune valeur publiée** : il décrit ce que le calcul fait (le sens est celui que Karma et Rappel lisent en 3D, PRL 77,
//! 4050, table I ; la théorie 2D n'a pas été lue ; ici le régime n'est pas établi). Valeurs attendues : `references/etape-p3-valeurs.md`, identifiant `V-P3-SENS-01`.

mod etape_p3_commun;

use etape_p3_commun::*;
use givre_lab::pointe::{Pointe, constante_de_selection, mesurer_pointe, vitesse_de_pointe};

/// Les anisotropies comparées.
const EPSILONS: [f64; 4] = [0.0, 0.02, 0.04, 0.06];
/// Les deux pas de grille.
const PAS_DE_GRILLE: [f64; 2] = [0.8, 0.4];

/// Une valeur figée : `(position, lignes, rayon, vitesse, sigma)`.
type Figee = (f64, usize, f64, f64, f64);

/// Les valeurs figées (le prototype et le second code du juge, mêmes bits) : pour chaque pas de grille (0,8 puis 0,4) et chaque
/// `epsilon` de `EPSILONS`, `(position à t = 150, lignes retenues, rayon, vitesse de t = 125 à 150, sigma)`.
const FIGEES: [[Figee; 4]; 2] = [
    [
        (34.276_752_176_906_04, 15, 37.338_831_351_551_38, 0.082_469_857_611_051_8, 0.008_697_277_219_631_747),
        (34.997_559_779_932_3, 13, 27.684_236_484_209_52, 0.086_724_246_479_871_86, 0.015_045_075_244_029_78),
        (36.039_074_908_812_715, 10, 16.723_553_671_356_004, 0.093_998_379_185_223_08, 0.038_038_422_530_045_08),
        (37.823_856_839_488_89, 7, 7.465_400_149_069_487, 0.106_680_107_106_049_61, 0.168_193_951_854_186_4),
    ],
    [
        (34.671_077_509_451_14, 29, 34.664_231_514_430_114, 0.083_405_447_196_017_52, 0.009_977_975_461_462_186),
        (35.453_515_617_612_67, 25, 25.092_857_707_882_192, 0.088_514_508_024_901_2, 0.017_942_596_570_943_648),
        (36.625_049_441_624_355, 19, 14.571_321_934_218_42, 0.097_251_918_604_996_77, 0.048_428_823_104_015_21),
        (38.606_176_753_991_03, 13, 6.418_246_953_361_745, 0.111_489_490_497_071_98, 0.217_737_495_199_820_14),
    ],
];

/// Un cas : la graine de Kim (`graine_de_kim` : `Delta = 0,65`, `d0 = 0,5`, `D = 1`, `W0 = 1`, rayon 15, quart de 60 de côté, `dt` sous
/// la borne de P2 avec `25 / dt` entier), mesurée à `t = 125` et à `t = 150` avec la profondeur 2 ; rend `(mesure à 150, vitesse de 125
/// à 150, sigma à 150 avec d0 = 0,5 et D = 1)`.
fn cas(epsilon: f64, dx: f64) -> (Pointe, f64, f64) {
    let (mut d, pas_par_periode) = graine_de_kim(epsilon, dx, 60.0, 25.0);
    d.avancer(5 * pas_par_periode).unwrap();
    let a = mesurer_pointe(&d, 2.0).unwrap();
    d.avancer(pas_par_periode).unwrap();
    let b = mesurer_pointe(&d, 2.0).unwrap();
    assert!((a.temps - 125.0).abs() < 1e-9 && (b.temps - 150.0).abs() < 1e-9, "temps {} et {}", a.temps, b.temps);
    let v = vitesse_de_pointe(&a, &b).unwrap();
    (b, v, constante_de_selection(0.5, 1.0, b.rayon, v).unwrap())
}

/// régression : (V-P3-SENS-01) **le sens de variation selon `ε`, mesuré sur notre modèle, pas une reproduction** : la même graine
/// ronde (table II de Kim et coll. : `Delta = 0,65`, `d0 = 0,5`, `D = 1`, `W0 = 1`, rayon 15, `u` joué ; quart de 60 de côté), au même
/// temps (`t = 150`, la vitesse sur `[125 ; 150]`, la profondeur 2), pour `ε` = 0 ; 0,02 ; 0,04 ; 0,06, à `dx` = 0,8 ET 0,4 (huit
/// calculs, faits en parallèle). (a) Les valeurs sont celles de `FIGEES` (position, vitesse, rayon, `σ*` à `1e-9` près en relatif ; le
/// nombre de lignes retenues exactement). (b) À chaque pas de grille, l'ORDRE est strict : quand `ε` monte, la pointe est plus loin,
/// elle va plus vite, son rayon baisse et `σ*` monte. (c) L'écart entre deux `ε` voisins dépasse NETTEMENT l'écart entre les grilles : pour
/// chaque paire voisine et chaque pas de grille, l'écart de la vitesse, du rayon et de `σ*` vaut au moins 2 fois le plus grand des deux
/// écarts entre les grilles (`|Q(0,8) - Q(0,4)|` aux deux `ε` de la paire ; observé : au moins 2,24 pour la vitesse, 3,58 pour le
/// rayon, 2,19 pour `σ*`), et celui de la position au moins 1,5 fois (observé : 1,58). Le domaine compte peu (hors des tests : un quart
/// de 96 déplace chaque grandeur de moins de 1,5 % de l'écart entre les grilles). **Le régime n'est pas établi** : à `t = 150` la vitesse baisse encore et le rayon
/// change encore (voir `MODELE.md`, les repères en régime établi).
///
/// Ne couvre pas : le régime établi ; une valeur publiée de la vitesse, du rayon ou de `σ*` (aucune en 2D n'est figée) ; un `ε` entre
/// ceux-ci ; un autre `Delta`, `d0` ou rayon de graine ; une faute de la dendrite elle-même (les tests de P2 la gardent).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn la_pointe_s_effile_et_accelere_quand_epsilon_monte() {
    let mesures: Vec<Vec<(Pointe, f64, f64)>> = std::thread::scope(|s| {
        let fils: Vec<Vec<_>> =
            PAS_DE_GRILLE.iter().map(|&dx| EPSILONS.iter().map(|&e| s.spawn(move || cas(e, dx))).collect()).collect();
        fils.into_iter().map(|ligne| ligne.into_iter().map(|f| f.join().unwrap()).collect()).collect()
    });
    let proche = |a: f64, b: f64| ((a - b) / b).abs() < 1e-9;
    // (a) les valeurs figées
    for (g, dx) in PAS_DE_GRILLE.iter().enumerate() {
        for (k, e) in EPSILONS.iter().enumerate() {
            let (p, v, s) = mesures[g][k];
            let (pos, lignes, rayon, vitesse, sigma) = FIGEES[g][k];
            assert_eq!(p.lignes, lignes, "dx = {dx}, eps = {e} : lignes");
            assert!(
                proche(p.position, pos) && proche(p.rayon, rayon) && proche(v, vitesse) && proche(s, sigma),
                "dx = {dx}, eps = {e} : position {} rayon {} vitesse {v} sigma {s}",
                p.position,
                p.rayon
            );
        }
    }
    // (b) l'ordre, (c) l'écart contre l'erreur de grille
    let grandeur = |q: usize, g: usize, k: usize| -> f64 {
        let (p, v, s) = mesures[g][k];
        [p.position, v, p.rayon, s][q]
    };
    let noms = ["position", "vitesse", "rayon", "sigma"];
    let monte = [true, true, false, true];
    let rapport_minimal = [1.5, 2.0, 2.0, 2.0];
    for q in 0..4 {
        for (g, dx) in PAS_DE_GRILLE.iter().enumerate() {
            for k in 0..3 {
                let (a, b) = (grandeur(q, g, k), grandeur(q, g, k + 1));
                assert!(
                    if monte[q] { b > a } else { b < a },
                    "{} : l'ordre, dx = {dx}, eps {} -> {}",
                    noms[q],
                    EPSILONS[k],
                    EPSILONS[k + 1]
                );
                let grille = (grandeur(q, 0, k) - grandeur(q, 1, k))
                    .abs()
                    .max((grandeur(q, 0, k + 1) - grandeur(q, 1, k + 1)).abs());
                assert!(
                    (b - a).abs() >= rapport_minimal[q] * grille,
                    "{} : dx = {dx}, eps {} -> {} : écart {:e}, erreur de grille {:e}",
                    noms[q],
                    EPSILONS[k],
                    EPSILONS[k + 1],
                    (b - a).abs(),
                    grille
                );
            }
        }
    }
}
