//! Étape 3c : l'état initial, l'écriture canonique et l'empreinte avec le bord circulaire, et `Flocon::new` qui reste hexagonal à l'octet. Valeurs
//! attendues : `references/etape-3-valeurs.md`, identifiants `V-BORD-xx` ; les écritures et les empreintes viennent de l'outil indépendant
//! `modele_independant_3c.py` (Python, entiers, `hashlib`) ou de celui de l'étape 3 (`modele_independant.py`) pour ce qui existait déjà.
//!
//! Les fautes que ces tests voient : l'octet de forme oublié (1 pour les deux bords) ou mis à une autre valeur ; un bord ignoré par `new` ou par
//! `avec_bord` ; un état initial qui dépend du bord ; une écriture du bord hexagonal qui a changé.

mod etape_3_commun;
mod etape_3c_commun;

use etape_3_commun::valeurs::{ECRITURE_INITIALE_HEX, ECRITURE_INITIALE_SHA256, P3_EMPREINTE_20, TRAJECTOIRES};
use etape_3_commun::{brut_de, cases, parametres};
use etape_3c_commun::valeurs::{
    ECRITURE_CIRCULAIRE_APRES_UN_PAS_HEX, ECRITURE_CIRCULAIRE_APRES_UN_PAS_SHA256, ECRITURE_CIRCULAIRE_INITIALE_HEX,
    ECRITURE_CIRCULAIRE_INITIALE_SHA256, TRAJECTOIRES_BORD,
};
use givre_lab::empreinte::sha256_hex;
use givre_lab::flocon::{Bord, Flocon, Parametres};
use givre_lab::math::Fixe;

fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// L'écriture canonique de la doc, écrite à la main : l'étiquette, l'octet de forme `forme`, le rayon, les trois `brut`, le temps, les masses.
fn ecriture_a_la_main(forme: u8, rayon: u32, p: Parametres, temps: u64, masses: &[i64]) -> Vec<u8> {
    let mut octets = b"GIVRE003".to_vec();
    octets.push(forme);
    octets.extend_from_slice(&rayon.to_le_bytes());
    octets.extend_from_slice(&p.alpha.0.to_le_bytes());
    octets.extend_from_slice(&p.beta.0.to_le_bytes());
    octets.extend_from_slice(&p.gamma.0.to_le_bytes());
    octets.extend_from_slice(&temps.to_le_bytes());
    for m in masses {
        octets.extend_from_slice(&m.to_le_bytes());
    }
    octets
}

/// juste : l'état initial d'un flocon à bord circulaire (V-BORD-17) : pour `R` de 1 à 12 et trois jeux de paramètres, `avec_bord(R, p,
/// Bord::Circulaire)` est au temps 0, `bord()` vaut `Circulaire`, `rayon()` vaut `R`, `parametres()` est `p`, la case `(0, 0)` vaut `Fixe::UN` et TOUTES
/// les autres cases valent `β` (les cases du bord aussi : on les lit une à une), une seule case est gelée (`gelees() = 1`, `rayon_cristal() = 0`,
/// `distance2_cristal() = 0`) et c'est la même chose avec `Bord::Hexagonal`. Un état initial qui dépend du bord, un bord laissé à 0, ou un
/// germe oublié échouent ici.
///
/// Ne couvre pas : l'écriture octet par octet (voir `ecriture_canonique_circulaire`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn etat_initial_a_bord_circulaire() {
    for (a, b, g) in [("1", "0.4", "0.001"), ("0.7", "0.3", "0.05"), ("2", "0", "0.5")] {
        let p = parametres(a, b, g);
        for rayon in 1..=12_u32 {
            for bord in [Bord::Circulaire, Bord::Hexagonal] {
                let f = Flocon::avec_bord(rayon, p, bord);
                assert_eq!(f.bord(), bord);
                assert_eq!((f.rayon(), f.temps(), f.parametres()), (rayon, 0, p));
                assert_eq!(f.masse(0, 0), Fixe::UN, "R = {rayon} {bord:?}");
                for (q, r) in cases(rayon as i32) {
                    if (q, r) != (0, 0) {
                        assert_eq!(f.masse(q, r), p.beta, "R = {rayon} {bord:?} ({q}, {r})");
                    }
                }
                assert_eq!(f.masses().len(), 3 * (rayon as usize) * (rayon as usize + 1) + 1);
                assert_eq!((f.gelees(), f.rayon_cristal(), f.distance2_cristal()), (1, 0, 0), "R = {rayon} {bord:?}");
            }
        }
    }
}

/// juste : l'écriture canonique à bord circulaire (V-BORD-06), écrite À LA MAIN : celle de l'état initial de `R = 1`, `α = 0.7`, `β = 0.3`, `γ = 0.05` est
/// celle de l'étape 3 (101 octets) avec l'octet de forme (le 9e, d'indice 8) égal à **2** au lieu de 1 : `"GIVRE003"`, `02`, `01 00 00 00`, `α`, `β`,
/// `γ` en `i64` petit-boutiste, le temps `0`, les 7 masses. Elle est égale à celle de Python (`ECRITURE_CIRCULAIRE_INITIALE_HEX`), à celle de
/// l'étape 3 octet 8 excepté (`ECRITURE_INITIALE_HEX`, qui vaut `01`), et à `ecriture_canonique()` ; son SHA-256 (Python `hashlib`) est `empreinte()`.
/// Et la même après un pas (le temps 1 aux octets 37 à 44). Puis, pour `R` de 1 à 9 et les deux bords, après 3 pas : l'octet 8 vaut 1 (hexagonal)
/// ou 2 (circulaire), et tout le reste de l'écriture est celle de la doc écrite à la main à partir de `masses()`. Un octet de forme oublié (1
/// pour les deux), ou une autre valeur (0, 3, `b'c'`), échoue ici.
///
/// Ne couvre pas : les grands états (voir `trajectoires_python_a_bord_circulaire`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn ecriture_canonique_circulaire() {
    let p = parametres("0.7", "0.3", "0.05");
    let beta = p.beta.0;
    let masses = [beta, beta, beta, Fixe::UN.0, beta, beta, beta];
    let a_la_main = ecriture_a_la_main(2, 1, p, 0, &masses);
    assert_eq!(a_la_main.len(), 45 + 8 * 7);
    assert_eq!(hex(&a_la_main), ECRITURE_CIRCULAIRE_INITIALE_HEX);
    let f = Flocon::avec_bord(1, p, Bord::Circulaire);
    assert_eq!(f.ecriture_canonique(), a_la_main);
    assert_eq!(
        &f.ecriture_canonique()[..13],
        &[0x47, 0x49, 0x56, 0x52, 0x45, 0x30, 0x30, 0x33, 0x02, 0x01, 0x00, 0x00, 0x00]
    );
    assert_eq!(f.empreinte(), ECRITURE_CIRCULAIRE_INITIALE_SHA256);
    assert_eq!(f.empreinte(), sha256_hex(&a_la_main));
    // la même que celle de l'étape 3, à l'octet de forme près
    let ancienne = ECRITURE_INITIALE_HEX;
    let nouvelle = ECRITURE_CIRCULAIRE_INITIALE_HEX;
    assert_eq!(&ancienne[..16], &nouvelle[..16]);
    assert_eq!(&ancienne[16..18], "01");
    assert_eq!(&nouvelle[16..18], "02");
    assert_eq!(&ancienne[18..], &nouvelle[18..]);
    assert_ne!(ECRITURE_INITIALE_SHA256, ECRITURE_CIRCULAIRE_INITIALE_SHA256);
    // après un pas
    let mut f = Flocon::avec_bord(1, p, Bord::Circulaire);
    f.pas();
    let ecriture = f.ecriture_canonique();
    assert_eq!(hex(&ecriture), ECRITURE_CIRCULAIRE_APRES_UN_PAS_HEX);
    assert_eq!(&ecriture[37..45], &1_u64.to_le_bytes());
    assert_eq!(f.empreinte(), ECRITURE_CIRCULAIRE_APRES_UN_PAS_SHA256);
    // R de 1 à 9, les deux bords, après 3 pas
    let p = parametres("1.5", "0.55", "0.0001");
    for rayon in 1..=9_u32 {
        for (bord, forme) in [(Bord::Hexagonal, 1_u8), (Bord::Circulaire, 2_u8)] {
            let mut f = Flocon::avec_bord(rayon, p, bord);
            f.avancer(3);
            let attendu = ecriture_a_la_main(forme, rayon, p, 3, &brut_de(&f.masses()));
            assert_eq!(f.ecriture_canonique(), attendu, "R = {rayon} {bord:?}");
            assert_eq!(f.ecriture_canonique()[8], forme);
            assert_eq!(f.empreinte(), sha256_hex(&attendu));
        }
    }
}

/// juste : `Flocon::new` est EXACTEMENT `avec_bord(.., Bord::Hexagonal)` (V-BORD-18), et reste ce qu'il était. Pour six couples `(R, p)` (dont `R` = 8,
/// 12 et 16 où les deux bords diffèrent), `new` et `avec_bord(.., Hexagonal)` ont la même écriture canonique à l'état initial et après 30 pas,
/// `bord()` vaut `Hexagonal` pour les deux, et l'octet de forme vaut 1. Puis les références de l'étape 3 (calculées par Python, avant l'ajout du bord
/// circulaire) : `ECRITURE_INITIALE_SHA256`, les six `TRAJECTOIRES` (empreinte, gelées, rayon, pointes) et `P3_EMPREINTE_20` sont encore celles de `new`
/// ET de `avec_bord(.., Hexagonal)`. Un `new` qui prend le bord circulaire, ou un octet de forme qui change pour le bord hexagonal, échouent ici.
///
/// Ne couvre pas : `depuis_masses` (voir `etape_3c_pas`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn new_est_hexagonal_a_l_octet() {
    let couples = [
        (3_u32, "1", "0.4", "0.001"),
        (8, "0.7", "0.3", "0.05"),
        (9, "2", "0.2", "0.3"),
        (12, "0.5", "0", "0.2"),
        (14, "1.5", "0.55", "0.0001"),
        (16, "1", "0.45", "0.1"),
    ];
    for (rayon, a, b, g) in couples {
        let p = parametres(a, b, g);
        let mut vieux = Flocon::new(rayon, p);
        let mut neuf = Flocon::avec_bord(rayon, p, Bord::Hexagonal);
        assert_eq!(vieux.bord(), Bord::Hexagonal);
        assert_eq!(vieux.ecriture_canonique(), neuf.ecriture_canonique(), "R = {rayon} : état initial");
        assert_eq!(vieux.ecriture_canonique()[8], 1);
        vieux.avancer(30);
        neuf.avancer(30);
        assert_eq!(vieux.ecriture_canonique(), neuf.ecriture_canonique(), "R = {rayon} : après 30 pas");
        assert_eq!(vieux.empreinte(), neuf.empreinte());
    }
    let p = parametres("0.7", "0.3", "0.05");
    assert_eq!(Flocon::new(1, p).empreinte(), ECRITURE_INITIALE_SHA256);
    assert_eq!(Flocon::avec_bord(1, p, Bord::Hexagonal).empreinte(), ECRITURE_INITIALE_SHA256);
    for &(rayon, a, b, g, pas, empreinte, gelees, rayon_cristal, pointes) in TRAJECTOIRES {
        let p = parametres(a, b, g);
        let mut vieux = Flocon::new(rayon, p);
        let mut neuf = Flocon::avec_bord(rayon, p, Bord::Hexagonal);
        vieux.avancer(pas);
        neuf.avancer(pas);
        for f in [&vieux, &neuf] {
            assert_eq!(f.empreinte(), empreinte, "R={rayon} alpha {a} beta {b} gamma {g} après {pas} pas");
            assert_eq!((f.gelees(), f.rayon_cristal(), f.pointes()), (gelees, rayon_cristal, pointes));
        }
    }
    let p = parametres("1", "0.45", "0.1");
    let mut vieux = Flocon::new(3, p);
    let mut neuf = Flocon::avec_bord(3, p, Bord::Hexagonal);
    vieux.avancer(20);
    neuf.avancer(20);
    assert_eq!(vieux.empreinte(), P3_EMPREINTE_20);
    assert_eq!(neuf.empreinte(), P3_EMPREINTE_20);
}

/// juste : des trajectoires depuis l'état initial, calculées par Python (V-BORD-05), pour six jeux `(R, α, β, γ, pas)` avec `R` de 10 à 16 (là où le
/// bord circulaire diffère de la couronne) : avec `Bord::Circulaire`, l'empreinte, les cases gelées, le rayon du cristal, les pointes et
/// `distance2_cristal` sont ceux de Python ; avec `Bord::Hexagonal`, l'empreinte est celle de Python aussi (elle diffère de la précédente : le bord
/// compte). Les deux bords ont des empreintes différentes pour chaque jeu. Une faute dans la règle du bord (n'importe laquelle), dans l'ordre
/// des opérations ou dans l'octet de forme, change l'une des douze empreintes.
///
/// Ne couvre pas : les très grands rayons (voir `etape_3c_table`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn trajectoires_python_a_bord_circulaire() {
    for &(rayon, a, b, g, pas, circ, gelees, rayon_cristal, pointes, d2, hexagonal) in TRAJECTOIRES_BORD {
        let p = parametres(a, b, g);
        let mut rond = Flocon::avec_bord(rayon, p, Bord::Circulaire);
        let mut hexa = Flocon::avec_bord(rayon, p, Bord::Hexagonal);
        rond.avancer(pas);
        hexa.avancer(pas);
        let etiquette = format!("R={rayon} alpha {a} beta {b} gamma {g} après {pas} pas");
        assert_eq!(rond.empreinte(), circ, "{etiquette} : bord circulaire");
        assert_eq!(
            (rond.gelees(), rond.rayon_cristal(), rond.pointes(), rond.distance2_cristal()),
            (gelees, rayon_cristal, pointes, d2),
            "{etiquette}"
        );
        assert_eq!(hexa.empreinte(), hexagonal, "{etiquette} : bord hexagonal");
        assert_ne!(circ, hexagonal, "{etiquette} : deux bords, deux empreintes");
    }
}

/// juste : deux flocons de même rayon, mêmes paramètres, même temps et bords différents ont deux empreintes (V-BORD-19), même quand leurs masses sont
/// égales : pour `R <= 7` le bord circulaire est la couronne, donc après 30 pas les masses sont identiques (`masses()` égales) et seules les
/// écritures canoniques diffèrent, d'un seul octet (le 9e : 1 contre 2) ; pour `R` de 8 à 12 les masses diffèrent aussi. À l'état initial les masses
/// sont égales pour tout `R` et l'écriture ne diffère encore que par l'octet de forme. Une empreinte qui ne voit pas le bord (octet de forme oublié) échoue
/// ici.
///
/// Ne couvre pas : les valeurs des empreintes (voir `trajectoires_python_a_bord_circulaire`).
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn deux_bords_deux_empreintes() {
    let p = parametres("1", "0.4", "0.01");
    for rayon in 1..=12_u32 {
        let mut rond = Flocon::avec_bord(rayon, p, Bord::Circulaire);
        let mut hexa = Flocon::avec_bord(rayon, p, Bord::Hexagonal);
        for (t, pas) in [(0_u64, 0_u64), (30, 30)] {
            rond.avancer(pas);
            hexa.avancer(pas);
            assert_eq!(rond.temps(), t);
            let (e1, e2) = (rond.ecriture_canonique(), hexa.ecriture_canonique());
            assert_ne!(rond.empreinte(), hexa.empreinte(), "R = {rayon} temps {t}");
            let differences: Vec<usize> = (0..e1.len()).filter(|&i| e1[i] != e2[i]).collect();
            if t == 0 || rayon <= 7 {
                assert_eq!(differences, vec![8], "R = {rayon} temps {t} : seul l'octet de forme diffère");
                assert_eq!(rond.masses(), hexa.masses());
            } else {
                assert!(
                    differences.contains(&8) && differences.len() > 1,
                    "R = {rayon} temps {t} : les masses diffèrent aussi"
                );
            }
        }
    }
}
