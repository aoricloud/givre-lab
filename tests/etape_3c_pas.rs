//! Étape 3c : un pas et plusieurs pas du modèle avec le bord circulaire (et avec le bord hexagonal sur de plus grands hexagones que ceux de
//! l'étape 3). Valeurs attendues : `references/etape-3-valeurs.md`, identifiants `V-BORD-xx` ; les états viennent de l'outil indépendant
//! `modele_independant_3c.py` (Python, entiers) et d'un simulateur NAÏF écrit dans le test (`etape_3c_commun::NaifBord`, d'après la doc).
//!
//! Les fautes que ces tests voient : un bord ignoré par `avec_bord` ; une diffusion appliquée aussi aux cases du bord ; un bord qui n'est pas remis
//! à `β` (ou pas à chaque pas) ; un bord remis à `β` AVANT que les voisines l'aient lu ; une case du bord laissée hors de la mise à jour ; des
//! voisins pris hors de l'hexagone ; la couronne seule remise à `β` avec le bord circulaire.

mod etape_3_commun;
mod etape_3c_commun;

use etape_3_commun::{brut_de, cases, masses_formule, parametres};
use etape_3c_commun::valeurs::{POSES_CIRCULAIRE, POSES_HEXAGONAL, Pose};
use etape_3c_commun::{NaifBord, du_bord};
use givre_lab::flocon::{Bord, Flocon};
use givre_lab::math::Fixe;

/// Un état posé à la main : (rayon, α, β, γ, formule des masses `(a, b, c, d)`, cases gelées posées).
type PoseALaMain = (i32, &'static str, &'static str, &'static str, (i64, i64, i64, i64), &'static [(i32, i32)]);

/// Les masses posées d'une `Pose` de l'outil Python : la formule `Fixe::rapport((a · i + b) % c, d)`, puis les cases `gel` à `Fixe::UN`, la
/// deuxième (`gel[1]`) à `UN + 1 ulp`.
fn masses_de_la_pose(pose: &Pose) -> Vec<Fixe> {
    let (a, b, c, d) = pose.formule;
    let surcharges: Vec<((i32, i32), Fixe)> = pose
        .gel
        .iter()
        .enumerate()
        .map(|(k, &cellule)| (cellule, if k == 1 { Fixe(Fixe::UN.0 + 1) } else { Fixe::UN }))
        .collect();
    masses_formule(pose.rayon as i32, a, b, c, d, &surcharges)
}

/// juste : depuis des masses posées, un pas puis vingt pas EXACTS (V-BORD-04), pour `R = 9` et `R = 14`, les deux bords, comparés aux états de l'outil
/// indépendant (Python, `modele_independant_3c.py`, entiers) : TOUTES les masses (`brut`) après 1 pas, puis après 20 pas, et l'empreinte, le
/// nombre de gelées et `distance2_cristal` après 20 pas. Les masses sont une formule `Fixe::rapport((a · i + b) % c, d)` avec cinq cases gelées
/// posées (dont une sur la couronne, une exactement au bord circulaire `(-8, 0)` ou `(-13, 0)` et une à `1 + 1 ulp`). Pour `R = 9` le bord circulaire
/// a 6 cases de plus que la couronne (les coins de la rangée 8), pour `R = 14` 30 de plus : les états des deux bords diffèrent à chaque pas (le test le
/// vérifie, ils ne sont pas égaux). Un bord ignoré, une diffusion qui touche les cases du bord, un bord non remis à `β`, une couronne seule remise à
/// `β` avec le bord circulaire, l'ordre de deux opérations échouent ici.
///
/// Ne couvre pas : les états de départ qui viennent du germe (voir `etape_3c_etat` et `naif_depuis_le_germe`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn un_pas_et_vingt_pas_python() {
    let jeux: [(&[Pose], Bord); 2] = [(POSES_CIRCULAIRE, Bord::Circulaire), (POSES_HEXAGONAL, Bord::Hexagonal)];
    let mut apres_un_pas: Vec<(String, Vec<i64>)> = Vec::new();
    for (poses, bord) in jeux {
        assert_eq!(poses.len(), 2);
        for pose in poses {
            let p = parametres(pose.alpha, pose.beta, pose.gamma);
            let masses = masses_de_la_pose(pose);
            let mut f = Flocon::avec_bord_et_masses(pose.rayon, p, bord, &masses);
            assert_eq!(f.bord(), bord);
            assert_eq!(brut_de(&f.masses()), brut_de(&masses), "{} {bord:?} : l'état posé est conservé", pose.nom);
            f.pas();
            assert_eq!(f.temps(), 1);
            assert_eq!(brut_de(&f.masses()), pose.apres_un_pas, "{} {bord:?} : après 1 pas", pose.nom);
            apres_un_pas.push((format!("{} {bord:?}", pose.nom), brut_de(&f.masses())));
            f.avancer(19);
            assert_eq!(f.temps(), 20);
            assert_eq!(brut_de(&f.masses()), pose.apres_vingt_pas, "{} {bord:?} : après 20 pas", pose.nom);
            assert_eq!(f.empreinte(), pose.empreinte_vingt_pas, "{} {bord:?} : empreinte après 20 pas", pose.nom);
            assert_eq!(f.gelees(), pose.gelees_vingt_pas, "{} {bord:?} : gelées", pose.nom);
            assert_eq!(
                f.distance2_cristal(),
                pose.distance2_cristal_vingt_pas,
                "{} {bord:?} : distance2_cristal",
                pose.nom
            );
        }
    }
    // les deux bords donnent des états différents dès le premier pas
    for k in 0..2 {
        assert_ne!(apres_un_pas[k].1, apres_un_pas[2 + k].1, "{} : le bord change l'état", apres_un_pas[k].0);
    }
}

/// juste : la valeur d'un pas calculée À LA MAIN sur un cas minuscule (V-BORD-13) : `R = 8`, `α = 1`, `β = 0`, `γ = 0`, toutes les masses nulles sauf la
/// case `(7, 0)` à 1/2 (`brut` `2^31`). Avec le bord CIRCULAIRE, `(7, 0)` est du bord (`4 · 49 = 196 >= 3 · 64 = 192`) : elle ne diffuse pas, elle est
/// remise à `β = 0`, mais sa masse de départ sert à ses voisines. Ses six voisines : `(8, 0)`, `(8, -1)` et `(7, 1)` sont du bord (elles sont sur la
/// couronne) ; `(6, 0)`, `(6, 1)` et `(7, -1)` (`4 · 43 = 172 < 192`) ne le sont pas. Chacune des trois reçoit :
/// `moyenne = Fixe(2^31).div_entier(6) = 357913941` (`(2 · 2^31 + 6) / 12` arrondi par le bas), `ecart = moyenne - 0`, `alpha * ecart = 357913941`
/// (`alpha = 1`), `diffusion = div_entier(2) = (2 · 357913941 + 2) / 4 = 178956971`. Tout le reste vaut 0. Avec le bord HEXAGONAL, `(7, 0)` n'est pas du
/// bord : elle diffuse et garde `2^31 + diffusion` avec `moyenne = 0`, `ecart = -2^31`, `diffusion = (2 · (-2^31) + 2) / 4` arrondi par le bas `=
/// -2^30`, soit `2^30 = 1073741824`, et ses trois voisines intérieures reçoivent les mêmes `178956971` (leurs autres voisins valent 0). Calculé aussi
/// par Python. Un bord qui diffuse (le résultat du centre `(7, 0)`), un bord non remis à `β`, un bord ignoré (les deux résultats seraient
/// égaux) échouent ici.
///
/// Ne couvre pas : plus d'un pas (voir `un_pas_et_vingt_pas_python`, `naif_depuis_des_masses_posees`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn un_pas_a_la_main() {
    let p = parametres("1", "0", "0");
    let mut masses = vec![Fixe::ZERO; cases(8).len()];
    let indice = cases(8).iter().position(|&c| c == (7, 0)).expect("(7, 0) est dans l'hexagone");
    masses[indice] = Fixe(1 << 31);
    for (bord, centre) in [(Bord::Circulaire, 0_i64), (Bord::Hexagonal, 1_i64 << 30)] {
        let mut f = Flocon::avec_bord_et_masses(8, p, bord, &masses);
        f.pas();
        let mut attendu = vec![0_i64; masses.len()];
        for cellule in [(6, 0), (6, 1), (7, -1)] {
            attendu[cases(8).iter().position(|&c| c == cellule).unwrap()] = 178_956_971;
        }
        attendu[indice] = centre;
        assert_eq!(brut_de(&f.masses()), attendu, "{bord:?}");
    }
}

/// juste : le bord est remis à `β` à CHAQUE pas, et il sert de valeur avant d'être écrasé (V-BORD-14). Les masses de départ sont posées par la formule
/// (donc toutes différentes de `β`, bord compris), `R = 9`, `α = 0.9`, `β = 0.35`, `γ = 0.02`, bord circulaire (60 cases) puis hexagonal (54) : après
/// le 1er pas et après chacun des 10 suivants, TOUTES les cases du bord valent exactement `β`, et (le test le vérifie) aucune case hors du bord ne
/// vaut `β` partout par hasard : au moins les trois quarts des cases hors du bord valent autre chose que `β`. Le bord est calculé par le test d'une
/// autre façon que la doc (`(2q + r)² + 3 r² >= 3 R²`). Un bord remis à `β` seulement au premier pas, un bord laissé tel quel, ou la couronne
/// seule remise à `β` avec le bord circulaire (les 6 coins de la rangée 8 gardent leur valeur) échouent ici.
///
/// Ne couvre pas : la valeur des cases hors du bord (voir `un_pas_et_vingt_pas_python`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn le_bord_est_remis_a_beta_a_chaque_pas() {
    let p = parametres("0.9", "0.35", "0.02");
    let masses = masses_formule(9, 37, 11, 97, 120, &[((7, 0), Fixe::UN), ((-3, 5), Fixe::UN)]);
    for bord in [Bord::Circulaire, Bord::Hexagonal] {
        let mut f = Flocon::avec_bord_et_masses(9, p, bord, &masses);
        for t in 1..=11 {
            f.pas();
            let mut hors_du_bord = 0;
            let mut differentes = 0;
            for (q, r) in cases(9) {
                if du_bord(bord, 9, q, r) {
                    assert_eq!(f.masse(q, r), p.beta, "{bord:?} pas {t} : ({q}, {r}) est du bord, donc vaut β");
                } else {
                    hors_du_bord += 1;
                    differentes += usize::from(f.masse(q, r) != p.beta);
                }
            }
            assert!(
                4 * differentes >= 3 * hors_du_bord,
                "{bord:?} pas {t} : l'intérieur n'est pas partout β ({differentes} sur {hors_du_bord})"
            );
        }
    }
}

/// Fait avancer `Flocon` et le naïf pas à pas depuis le même état et vérifie l'égalité des masses à chaque pas (et du temps).
fn comparer(mut f: Flocon, mut naif: NaifBord, pas: u64, etiquette: &str) {
    assert_eq!(brut_de(&f.masses()), naif.brut(), "{etiquette} : état de départ");
    for t in 1..=pas {
        f.pas();
        naif.pas();
        assert_eq!(f.temps(), t);
        assert_eq!(naif.temps, t);
        assert_eq!(brut_de(&f.masses()), naif.brut(), "{etiquette} : pas {t}");
    }
}

/// juste : depuis le germe, avec chaque bord, le simulateur naïf du test (`NaifBord`, d'après la doc, avec un dictionnaire de cases et un autre ordre des
/// voisins, et un bord calculé par `(2q + r)² + 3 r² >= 3 R²` en `i128`) donne EXACTEMENT les `brut` de toutes les cases, à chaque pas (V-BORD-15) : huit
/// jeux `(α, β, γ)`, des rayons de 8 à 16 (là où le bord circulaire diffère de la couronne), 40 à 80 pas, les deux bords. Les dynamiques couvertes :
/// un germe qui ne grandit pas (`β = 0, γ = 0`), des plaques, des étoiles, des branches, la vapeur qui atteint le bord. Une faute plausible (un
/// bord oublié, un voisin hors de l'hexagone lu comme zéro ou comme `β`, l'ordre de deux opérations) fait diverger le naïf.
///
/// Ne couvre pas : les états de départ posés à la main (voir `naif_depuis_des_masses_posees`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn naif_depuis_le_germe() {
    let jeux = [
        (9, "0.7", "0.3", "0.05", 60),
        (10, "1", "0.4", "0.001", 80),
        (11, "2", "0.2", "0.3", 50),
        (12, "0.5", "0", "0.2", 60),
        (14, "1.5", "0.55", "0.0001", 70),
        (13, "1", "0.45", "0.1", 60),
        (16, "0.3", "0.8", "0.002", 40),
        (8, "1", "0", "0", 20),
    ];
    for (rayon, a, b, g, pas) in jeux {
        for bord in [Bord::Circulaire, Bord::Hexagonal] {
            let p = parametres(a, b, g);
            comparer(
                Flocon::avec_bord(rayon, p, bord),
                NaifBord::initial(rayon as i32, p, bord),
                pas,
                &format!("R={rayon} {bord:?} alpha {a} beta {b} gamma {g}"),
            );
        }
    }
}

/// juste : depuis des états posés à la main (la formule `Fixe::rapport((a · i + b) % c, d)` et des gelées dont une sur le bord), comme
/// `naif_depuis_le_germe` mais sur les poses de l'outil Python et trois autres (V-BORD-15) : `avec_bord_et_masses` donne les mêmes `brut` que le
/// naïf, à chaque pas, 30 pas, pour les deux bords. Les cases du bord de l'état de départ ne valent pas `β` : leur valeur sert aux voisines au premier
/// pas, puis elles sont remises à `β`.
///
/// Ne couvre pas : l'équivariance (voir `etape_3c_symetrie`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn naif_depuis_des_masses_posees() {
    let cas: [PoseALaMain; 5] = [
        (9, "0.9", "0.35", "0.02", (37, 11, 97, 120), &[(7, 0), (-3, 5), (-8, 0), (0, 0), (0, -9)]),
        (14, "1", "0.4", "0.05", (53, 7, 89, 110), &[(12, 0), (-6, 10), (-13, 0), (7, 7), (5, -9)]),
        (10, "2", "0.5", "0.25", (31, 5, 83, 100), &[(9, 0), (-9, 9), (0, -5)]),
        (12, "0.6", "0.1", "0.01", (41, 3, 79, 90), &[(0, 11), (11, -11), (6, 6), (-10, 2)]),
        (8, "1.5", "0", "0.3", (29, 17, 71, 80), &[(7, 0), (3, 4), (-7, 7)]),
    ];
    for (rayon, a, b, g, (fa, fb, fc, fd), gel) in cas {
        let p = parametres(a, b, g);
        let surcharges: Vec<((i32, i32), Fixe)> = gel.iter().map(|&c| (c, Fixe::UN)).collect();
        let masses = masses_formule(rayon, fa, fb, fc, fd, &surcharges);
        let brut = brut_de(&masses);
        for bord in [Bord::Circulaire, Bord::Hexagonal] {
            comparer(
                Flocon::avec_bord_et_masses(rayon as u32, p, bord, &masses),
                NaifBord::depuis_brut(rayon, p, bord, &brut),
                30,
                &format!("R={rayon} {bord:?} alpha {a} beta {b} gamma {g}"),
            );
        }
    }
}

/// juste : `depuis_masses` reste hexagonal (V-BORD-16) : `Flocon::depuis_masses(r, p, m)` donne le même état, pas à pas, que
/// `avec_bord_et_masses(r, p, Bord::Hexagonal, m)`, et `bord()` vaut `Bord::Hexagonal` ; avec `Bord::Circulaire` et `R >= 8` l'état diffère
/// dès le premier pas. Les masses sont la formule de la pose `R = 9`.
///
/// Ne couvre pas : `Flocon::new` (voir `etape_3c_etat`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn depuis_masses_est_hexagonal() {
    let p = parametres("0.9", "0.35", "0.02");
    let masses = masses_formule(9, 37, 11, 97, 120, &[((7, 0), Fixe::UN), ((-3, 5), Fixe::UN)]);
    let mut a = Flocon::depuis_masses(9, p, &masses);
    let mut b = Flocon::avec_bord_et_masses(9, p, Bord::Hexagonal, &masses);
    let mut c = Flocon::avec_bord_et_masses(9, p, Bord::Circulaire, &masses);
    assert_eq!(a.bord(), Bord::Hexagonal);
    assert_eq!(a.ecriture_canonique(), b.ecriture_canonique());
    assert_ne!(a.ecriture_canonique(), c.ecriture_canonique());
    for t in 1..=5 {
        a.pas();
        b.pas();
        c.pas();
        assert_eq!(brut_de(&a.masses()), brut_de(&b.masses()), "pas {t}");
        assert_ne!(brut_de(&a.masses()), brut_de(&c.masses()), "pas {t}");
    }
}
