//! Étape P1 : la règle des nombres d'`AGENTS.md` (des `f64` permis sous une symétrie exacte si chaque somme que la symétrie échange
//! s'écrit par paires, et si un test montre la symétrie au bit près) : la preuve en 1D (le miroir de `Front1d`) et en 2D (les huit
//! symétries du carré, la diffusion de `Diffusion2d`). Valeurs attendues : `references/etape-p1-valeurs.md`, identifiants
//! `V-P1-SYM-xx`.

mod etape_p1_commun;

use etape_p1_commun::*;
use givre_lab::phase::{Diffusion2d, Front1d, Front1dFixe, ModeU};

/// Un état symétrique (`signe = +1` : pair, `φ[n-1-i] = φ[i]`) ou antisymétrique (`signe = -1` : `φ[n-1-i] = -φ[i]`) tiré.
fn symetrique(n: usize, graine: u64, signe: f64) -> (Vec<f64>, Vec<f64>) {
    let mut g = Lcg(graine);
    let (mut phi, mut u) = (vec![0.0; n], vec![0.0; n]);
    for i in 0..n / 2 {
        let (a, b) = (g.entre(-1.0, 1.0), g.entre(-0.3, 0.3));
        phi[i] = a;
        phi[n - 1 - i] = signe * a;
        u[i] = b;
        u[n - 1 - i] = signe * b;
    }
    (phi, u)
}

/// Le nombre de cases de `v` qui ne sont pas l'image exacte (`signe * v[i]`, au bit près) de leur symétrique.
fn defauts(v: &[f64], signe: f64) -> usize {
    let n = v.len();
    (0..n).filter(|&i| v[n - 1 - i].to_bits() != (signe * v[i]).to_bits()).count()
}

/// juste : (V-P1-SYM-01) la symétrie miroir de `Front1d` est exacte au bit près : depuis un état tiré pair (`φ[n-1-i] = φ[i]`, `u` de même)
/// ou impair (`φ[n-1-i] = -φ[i]`, `u` de même : `(φ, u) -> (-φ, -u)` laisse le modèle invariant), `n` pair de 6 à 40, 4 graines, `u`
/// gelé ou joué, les deux états ne cessent d'être symétriques, au bit près, à 1, 2, 3, 5, 10, 50, 200, 1000 et 2000 pas. Chaque somme que le
/// miroir échange (`phi[i-1] + phi[i+1]`) est écrite par paires, la négation est exacte et l'arrondi au plus proche est symétrique : c'est
/// la règle de `AGENTS.md`. (`n` impair : voir `miroir_1d_n_impair_et_zero_central`.)
///
/// Ne couvre pas : `Front1dFixe` (voir `fixe_pair_exact_impair_non`) ; le 2D (voir plus bas).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn miroir_1d_f64_exact() {
    for n in [6_usize, 14, 24, 40] {
        for graine in 1..=4_u64 {
            let mut p = param(1.1, 1.3, 1.2, 0.8, 0.6, 0.0);
            p.dt = pas_de_temps(&p, 0.5);
            for signe in [1.0, -1.0] {
                for mode in [ModeU::Gele, ModeU::Joue] {
                    let (phi, u) = symetrique(n, 100 * graine + n as u64, signe);
                    let mut f = Front1d::new(p, &phi, &u, mode).unwrap();
                    let mut fait = 0;
                    for pas in [1_u64, 2, 3, 5, 10, 50, 200, 1000, 2000] {
                        f.avancer(pas - fait).unwrap();
                        fait = pas;
                        assert_eq!(
                            defauts(f.phi(), signe),
                            0,
                            "n={n} graine {graine} signe {signe} {mode:?} pas {pas} : phi"
                        );
                        assert_eq!(
                            defauts(f.u(), signe),
                            0,
                            "n={n} graine {graine} signe {signe} {mode:?} pas {pas} : u"
                        );
                    }
                    // l'état a bougé (le test ne serait pas à vide) : phi a changé, il a toujours `n` cases, 2000 pas sont faits
                    assert_ne!(bits_de(f.phi()), bits_de(&phi));
                    assert_eq!((f.phi().len(), f.u().len(), f.pas_faits()), (n, n, 2000));
                }
            }
        }
    }
}

fn bits_de(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// juste : (V-P1-SYM-02) en virgule fixe, le miroir PAIR est exact (champ brut égal à celui du symétrique, 2000 pas, `u` gelé ou joué, 4 graines) :
/// il n'échange que les deux opérandes d'un `+`. Le miroir IMPAIR (changement de signe), lui, ne l'est pas : `Fixe` arrondit les demi-cas vers
/// `+∞` (`0,5 · dphi` avec un champ brut impair est un demi-cas), pas vers zéro, donc `Fixe(-x) * y` n'est pas `-(Fixe(x) * y)`. Cette
/// asymétrie est celle du produit de `Fixe` (`src/math.rs`) : on mesure qu'elle apparaît (au moins une des 4 graines) quand `u` est joué.
///
/// Ne couvre pas : le miroir impair avec `u` gelé (il est exact : aucun demi-cas ne s'y produit dans ces essais, sans que ce soit une garantie).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn fixe_pair_exact_impair_non() {
    let mut p = param(1.1, 1.3, 1.2, 0.8, 0.6, 0.0);
    p.dt = pas_de_temps(&p, 0.5);
    let mut impair_casse = 0;
    for graine in 1..=4_u64 {
        for mode in [ModeU::Gele, ModeU::Joue] {
            let (phi, u) = symetrique(24, 500 + graine, 1.0);
            let mut f = Front1dFixe::new(p, &phi, &u, mode).unwrap();
            f.avancer(2000).unwrap();
            let n = f.phi().len();
            for i in 0..n {
                assert_eq!(f.phi()[n - 1 - i], f.phi()[i], "pair, graine {graine} {mode:?} : phi[{i}]");
                assert_eq!(f.u()[n - 1 - i], f.u()[i], "pair, graine {graine} {mode:?} : u[{i}]");
            }
            let (phi, u) = symetrique(24, 500 + graine, -1.0);
            let mut f = Front1dFixe::new(p, &phi, &u, mode).unwrap();
            f.avancer(1).unwrap();
            let n = f.phi().len();
            let rompu = (0..n).any(|i| f.phi()[n - 1 - i].0 != -f.phi()[i].0 || f.u()[n - 1 - i].0 != -f.u()[i].0);
            if mode == ModeU::Joue && rompu {
                impair_casse += 1;
            }
        }
    }
    assert!(impair_casse >= 1, "le miroir impair de Fixe devait se casser (demi-cas) pour au moins une graine");
}

/// juste : (V-P1-SYM-03) la règle est NÉCESSAIRE en 1D : le naïf `f64` écrit SANS paires (`(phi[i-1] - 2 phi[i]) + phi[i+1]`, la faute
/// `SansPaires`) perd la symétrie miroir (pair et impair) dès les 3 premiers pas, pour chacune des 4 graines (observé : au pas 1 ou 2),
/// alors que le même naïf par paires la garde sur les mêmes 3 pas.
///
/// Ne couvre pas : la perte à long terme ; le 2D (voir plus bas).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn sans_paires_perd_le_miroir_1d() {
    let mut p = param(1.1, 1.3, 1.2, 0.8, 0.6, 0.0);
    p.dt = pas_de_temps(&p, 0.5);
    for signe in [1.0, -1.0] {
        for graine in 1..=4_u64 {
            let (phi, u) = symetrique(24, graine, signe);
            let rompu = |faute: Option<Faute>| {
                (1..=3_u64).any(|pas| {
                    let e = naif64(&p, &phi, &u, ModeU::Joue, pas, faute);
                    defauts(&e.phi, signe) + defauts(&e.u, signe) > 0
                })
            };
            assert!(
                rompu(Some(Faute::SansPaires)),
                "signe {signe} graine {graine} : le naïf sans paires devait perdre la symétrie"
            );
            assert!(!rompu(None), "signe {signe} graine {graine} : le naïf par paires devait la garder");
        }
    }
}

/// juste : (V-P1-SYM-04) la diffusion 2D de `Diffusion2d` garde les HUIT symétries du carré (l'identité, trois rotations, quatre miroirs) au bit
/// près à CHAQUE pas : `n` = 3, 8, 16, 17 et 33, `k` = 0,2 et 0,249, 300 pas, depuis six graines à huit symétries (`graine_2d_sel`, sels 0 à 5 :
/// des flottants aux mantisses pleines, tous différents selon la distance au centre) ; les symétries sont écrites dans le test
/// (`SYMETRIES_2D`, des transformations d'indices), pas déduites de la bibliothèque ; et l'état n'est pas resté la graine (les bits ont
/// changé ; pour `n >= 16` le maximum a baissé de plus de 10 %), la somme est conservée à `1e-9` près (le bord sans flux).
///
/// Ne couvre pas : le pas isotrope complet `(φ, u)` en 2D (voir `MODELE.md` : mesuré pour information, pas gelé) ; une graine sans symétrie.
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn diffusion_2d_garde_les_huit_symetries() {
    for n in [3_usize, 8, 16, 17, 33] {
        for sel in 0..6_u64 {
            for k in [0.2, 0.249] {
                let graine = graine_2d_sel(n, sel);
                assert!(
                    defauts_2d(n, &graine).iter().all(|&d| d == 0),
                    "la graine de n={n} sel={sel} n'est pas symétrique"
                );
                let somme0: f64 = graine.iter().sum();
                let max0 = graine.iter().cloned().fold(0.0, f64::max);
                let mut d = Diffusion2d::new(n, k, &graine).unwrap();
                for pas in 1..=300 {
                    d.pas();
                    let defauts = defauts_2d(n, d.cellules());
                    for (s, &x) in defauts.iter().enumerate() {
                        assert_eq!(x, 0, "n={n} sel={sel} k={k} pas {pas} : {} ({x} cases)", SYMETRIES_2D[s].0);
                    }
                }
                let somme: f64 = d.cellules().iter().sum();
                assert!((somme - somme0).abs() <= 1e-9, "n={n} sel={sel} k={k} : somme {somme} contre {somme0}");
                assert_ne!(bits_de(d.cellules()), bits_de(&graine), "n={n} sel={sel} k={k} : l'état n'a pas bougé");
                let max = d.cellules().iter().cloned().fold(0.0, f64::max);
                assert!(
                    max > 0.0 && (n < 16 || max < 0.9 * max0),
                    "n={n} sel={sel} k={k} : maximum {max} contre {max0} (la diffusion n'a pas eu lieu)"
                );
            }
        }
    }
}

/// juste : (V-P1-SYM-05) la règle est NÉCESSAIRE en 2D : la même diffusion écrite sans paires perd les symétries. Le naïf de ce test, avec la
/// somme `((e + o) + n) + s`, perd au pas 1 six des sept symétries non triviales (toutes sauf le miroir est-ouest, qui n'échange que les
/// deux opérandes du premier `+`) ; avec `(e + n) + (o + s)` (des paires qui ne sont PAS des opposées) il en perd quatre (les deux miroirs et les deux rotations d'un quart de tour ; il garde la
/// rotation de 180 degrés, la transposition et l'anti-transposition) ; avec `((e + n) + o) + s` il les perd toutes ; et chacune
/// est perdue par au moins une cellule dans les 3 premiers pas (`n = 17`, `k = 0,2`). Le naïf par paires, lui, n'en perd aucune.
/// (Le test de la règle serait faible si l'écriture sans paires gardait la symétrie : elle ne la garde pas.)
///
/// Ne couvre pas : la raison pour laquelle ces six ou cinq symétries sont perdues dans cet ordre exact (on le mesure, on ne le démontre pas).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn diffusion_2d_sans_paires_perd_les_symetries() {
    let n = 17;
    let graine = graine_2d(n);
    // par écriture : les indices (dans SYMETRIES_2D) des symétries qui doivent être perdues dans les 3 premiers pas
    let attendu: [(u8, &[usize]); 4] =
        [(0, &[]), (1, &[1, 2, 3, 4, 5, 6]), (2, &[0, 1, 3, 4]), (3, &[0, 1, 2, 3, 4, 5, 6])];
    for (ecriture, perdues) in attendu {
        let mut c = graine.clone();
        let mut vues = [false; 7];
        for _ in 0..3 {
            c = pas_2d(n, 0.2, &c, ecriture);
            for (s, &x) in defauts_2d(n, &c).iter().enumerate() {
                vues[s] |= x > 0;
            }
        }
        let perdues_vues: Vec<usize> = (0..7).filter(|&s| vues[s]).collect();
        assert_eq!(perdues_vues, perdues, "écriture {ecriture} : symétries perdues dans les 3 premiers pas");
    }
}

/// juste : (V-P1-SYM-06) `Diffusion2d` = le naïf par paires du test (écriture 0), bit pour bit, sur 100 pas (`n` = 16 et 17, `k` = 0,2),
/// depuis la graine symétrique ET depuis un état tiré sans symétrie ; et un pas À LA MAIN : `n = 2`, `k = 1/4`, `c = [[1, 0], [0, 0]]` donne
/// `[[1/2, 1/4], [1/4, 0]]` (bord sans flux : la voisine hors grille est la case elle-même).
///
/// Ne couvre pas : les symétries (voir les tests précédents).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn diffusion_2d_egale_le_naif_et_un_pas_a_la_main() {
    for n in [16_usize, 17] {
        let mut g = Lcg(31 + n as u64);
        let departs = [graine_2d(n), (0..n * n).map(|_| g.suivant()).collect::<Vec<f64>>()];
        for depart in departs {
            let mut d = Diffusion2d::new(n, 0.2, &depart).unwrap();
            let mut c = depart.clone();
            for pas in 1..=100 {
                d.pas();
                c = pas_2d(n, 0.2, &c, 0);
                assert_eq!(bits_de(d.cellules()), bits_de(&c), "n={n} pas {pas}");
            }
        }
    }
    let mut d = Diffusion2d::new(2, 0.25, &[1.0, 0.0, 0.0, 0.0]).unwrap();
    d.pas();
    assert_eq!(d.cellules(), [0.5, 0.25, 0.25, 0.0]);
}

/// juste : (V-P1-SYM-07) le miroir 1D quand `n` est IMPAIR (7, 15, 31 cases, `u` joué, 500 pas) : le miroir PAIR reste exact au bit
/// près ; le miroir IMPAIR reste exact EN VALEUR (`φ[n-1-i] == -φ[i]`, `u` de même), et au bit près partout sauf sur la case du
/// milieu, qui vaut `+0.0` (bits nuls) alors que son image attendue `-(+0.0)` est `-0.0` : c'est la limite de la règle des nombres
/// pour une symétrie qui change le signe (un point fixe nul n'a pas « les mêmes bits que son image »). La symétrie d'une
/// dendrite (un miroir sans changement de signe) n'a pas cette limite.
///
/// Ne couvre pas : `Front1dFixe` ; le 2D.
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn miroir_1d_n_impair_et_zero_central() {
    let mut p = param(1.1, 1.3, 1.2, 0.8, 0.6, 0.0);
    p.dt = pas_de_temps(&p, 0.5);
    for n in [7_usize, 15, 31] {
        for signe in [1.0, -1.0] {
            let (mut phi, mut u) = symetrique(n, 900 + n as u64, signe);
            if signe > 0.0 {
                // le miroir pair n'impose rien à la case du milieu : une valeur quelconque
                phi[n / 2] = 0.3;
                u[n / 2] = -0.1;
            }
            let mut f = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
            f.avancer(500).unwrap();
            assert_ne!(bits_de(f.phi()), bits_de(&phi));
            assert_eq!((f.phi().len(), f.u().len(), f.pas_faits()), (n, n, 500));
            if signe > 0.0 {
                assert_eq!(defauts(f.phi(), 1.0) + defauts(f.u(), 1.0), 0, "n={n} : miroir pair");
            } else {
                for i in 0..n {
                    assert!(
                        f.phi()[n - 1 - i] == -f.phi()[i] && f.u()[n - 1 - i] == -f.u()[i],
                        "n={n} : miroir impair, case {i}"
                    );
                }
                assert_eq!(
                    (f.phi()[n / 2].to_bits(), f.u()[n / 2].to_bits()),
                    (0, 0),
                    "n={n} : le milieu devait rester +0.0"
                );
                assert_eq!(
                    defauts(f.phi(), -1.0),
                    1,
                    "n={n} : seule la case du milieu diffère de son image au bit près"
                );
                assert_eq!(defauts(f.u(), -1.0), 1, "n={n} : de même pour u");
            }
        }
    }
}
