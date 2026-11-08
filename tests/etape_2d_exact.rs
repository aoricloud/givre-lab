//! Étape 2d : les variantes de la DLA (`givre_lab::dla_variantes`), tests EXACTS. Valeurs attendues : `references/etape-2d-valeurs.md`,
//! identifiants `V-VAR-xx` (la dynamique), `V-NEUT-xx` (le lien avec l'étape 1), `V-CAP2-xx` (la capacité), `V-IND-xx` (le code Python
//! indépendant), `V-CAN2-xx` (l'écriture canonique), `V-INV-xx` (les invariants), `V-GRN-xx` (graines et paramètres).
//!
//! Le test le plus fort est `collage_egal_au_simulateur_naif` et son jumeau `bruit_egal_au_simulateur_naif` : un simulateur NAÏF écrit ici
//! (des boucles simples : un anneau énuméré à chaque lancement, un `BTreeSet` pour l'amas, un `BTreeMap` pour les compteurs) qui répète mot
//! pour mot la section « La dynamique, à la lettre » de la doc de `src/dla_variantes.rs`, y compris le tirage de collage `suivant()`, le pas
//! refusé contre l'amas, l'abandon après un compteur trop bas et la consommation du hasard appel par appel ; la bibliothèque doit donner
//! EXACTEMENT ses sites, l'état de son générateur et ses compteurs. Il a un défaut qu'on connaît : il est de la même main que la doc. C'est
//! pourquoi `valeurs_du_code_independant` compare aussi à des valeurs calculées par un QUATRIÈME code, en Python, écrit d'après la doc seule
//! (`ref_variantes.py`, modèle PCG32 de l'étape 0, structures autres : un `dict` de compteurs, un `set`), et pourquoi les valeurs neutres
//! (`p = 1`, `m = 1`) sont comparées à `Amas` de l'étape 1, validé par ailleurs.

mod etape_2d_commun;

/// Le délai d'un test de simulation, en secondes (voir `etape_2d_commun::avec_delai`).
const DELAI: u64 = 120;

use givre_lab::alea::Alea;
use givre_lab::dla::Amas;
use givre_lab::dla_variantes::{AmasVariante, BRUIT_MAX, ETIQUETTE, TOUJOURS, Variante};
use std::collections::{BTreeMap, BTreeSet};

/// `2^32`, écrit en dur (jamais lu dans la bibliothèque).
const DEUX_32: u64 = 1 << 32;
/// `p = 0,5` en Q0.32.
const MOITIE: u64 = 1 << 31;
/// `p = 0,2` en Q0.32 : `0,2 * 2^32 = 858993459,2`.
const CINQUIEME: u64 = 858_993_459;
/// `p = 0,05` en Q0.32 : `0,05 * 2^32 = 214748364,8`.
const VINGTIEME: u64 = 214_748_365;
/// `p = 1 - 2^-32` : un tirage sur `2^32` seulement ne colle pas, mais le tirage a lieu à chaque contact.
const PRESQUE: u64 = DEUX_32 - 1;

// ------------------------------------------------------------------------------------------------ le simulateur naïf

/// Les douze premiers sites d'un amas.
type Douze = [(i32, i32); 12];

const DIRECTIONS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)]; // droite, bas, gauche, haut

/// L'anneau, énuméré comme dit la doc de `dla::anneau` : lignes `y` de `-R` à `R`, dans chaque ligne `x` de `-R` à `R`, on garde
/// `R² <= x² + y² < (R+1)²`.
fn anneau_naif(r: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    for y in -r..=r {
        for x in -r..=r {
            let d = x * x + y * y;
            if d >= r * r && d < (r + 1) * (r + 1) {
                points.push((x, y));
            }
        }
    }
    points
}

/// Le plus petit entier `r >= 0` tel que tous les sites vérifient `x² + y² <= r²`.
fn rayon_naif(sites: &[(i32, i32)]) -> i32 {
    let mut r = 0;
    while !sites.iter().all(|&(x, y)| x * x + y * y <= r * r) {
        r += 1;
    }
    r
}

/// Le simulateur naïf d'une variante : l'état complet (générateur, sites, compteurs des sites vides) et la liste des tirages de contact
/// de la variante collage (les `suivant()` de contact, dans l'ordre).
struct Naif {
    alea: Alea,
    /// Le demi-côté du réseau (la capacité).
    h: i32,
    variante: Variante,
    sites: Vec<(i32, i32)>,
    dedans: BTreeSet<(i32, i32)>,
    compteurs: BTreeMap<(i32, i32), u32>,
    tirages_de_contact: Vec<u32>,
}

impl Naif {
    fn new(graine: u64, demi_cote: u32, variante: Variante) -> Naif {
        Naif {
            alea: Alea::new(graine),
            h: demi_cote as i32,
            variante,
            sites: vec![(0, 0)],
            dedans: BTreeSet::from([(0, 0)]),
            compteurs: BTreeMap::new(),
            tirages_de_contact: Vec::new(),
        }
    }

    fn coller(&mut self, x: i32, y: i32) -> Option<(i32, i32)> {
        if x.abs() > self.h || y.abs() > self.h {
            return None; // hors du réseau : la capacité est dépassée
        }
        self.sites.push((x, y));
        self.dedans.insert((x, y));
        Some((x, y))
    }

    /// Une particule, mot pour mot la doc (« Une particule se joue ainsi »).
    fn particule(&mut self) -> Option<(i32, i32)> {
        let lancement = rayon_naif(&self.sites) + 5; // R = r + MARGE (5)
        let abandon = 2 * lancement; // A = FACTEUR_ABANDON * R (2)
        loop {
            // 1. lancer
            let anneau = anneau_naif(lancement);
            let k = self.alea.borne(anneau.len() as u32) as usize;
            let (mut x, mut y) = anneau[k];
            loop {
                // 2.1 contact
                if DIRECTIONS.iter().any(|&(dx, dy)| self.dedans.contains(&(x + dx, y + dy))) {
                    match self.variante {
                        Variante::Collage(p) => {
                            let colle = if p == DEUX_32 {
                                true // « toujours » : aucun tirage
                            } else {
                                let u = self.alea.suivant();
                                self.tirages_de_contact.push(u);
                                u64::from(u) < p
                            };
                            if colle {
                                return self.coller(x, y);
                            }
                            // sinon : on passe à 2.2 avec la même position
                        }
                        Variante::Bruit(m) => {
                            if x.abs() > self.h || y.abs() > self.h {
                                return None; // contact hors du réseau : pas de compteur, capacité dépassée
                            }
                            let compteur = self.compteurs.entry((x, y)).or_insert(0);
                            *compteur += 1;
                            if *compteur == m {
                                self.compteurs.remove(&(x, y)); // le site rejoint l'amas : son compteur n'a plus de sens
                                return self.coller(x, y);
                            }
                            break; // abandonné : retour à l'étape 1 (même R, nouveau tirage borne(len))
                        }
                    }
                }
                // 2.2 abandonner
                if x * x + y * y > abandon * abandon {
                    break;
                }
                // 2.3 marcher : le pas vers un site de l'amas est refusé
                let (dx, dy) = DIRECTIONS[self.alea.borne(4) as usize];
                if !self.dedans.contains(&(x + dx, y + dy)) {
                    x += dx;
                    y += dy;
                }
            }
        }
    }

    /// Fait croître jusqu'à `n` sites (la capacité ne doit pas être dépassée).
    fn croitre(&mut self, n: usize) {
        while self.sites.len() < n {
            self.particule().expect("capacité dépassée dans le simulateur naïf");
        }
    }
}

/// Les 16 derniers octets de l'écriture canonique : l'état du générateur.
fn etat_du_generateur(amas: &AmasVariante) -> [u8; 16] {
    let octets = amas.ecriture_canonique();
    octets[octets.len() - 16..].try_into().unwrap()
}

/// Les sites, l'état du générateur et TOUS les compteurs du réseau de `amas` sont ceux du simulateur naïf.
fn comparer(amas: &AmasVariante, naif: &Naif, contexte: &str) {
    assert_eq!(amas.sites(), naif.sites.as_slice(), "{contexte} : sites");
    assert_eq!(etat_du_generateur(amas), naif.alea.octets(), "{contexte} : état du générateur");
    let h = amas.demi_cote() as i32;
    for y in -h..=h {
        for x in -h..=h {
            let attendu = naif.compteurs.get(&(x, y)).copied().unwrap_or(0);
            assert_eq!(amas.compteur(x, y), attendu, "{contexte} : compteur de ({x}, {y})");
        }
    }
}

fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

// -------------------------------------------------------------------------------- la dynamique, contre le simulateur naïf

/// juste : le simulateur naïf écrit dans ce fichier (V-VAR-01), pour la variante COLLAGE : les graines 1 à 8, `p` = 0,5, 0,2, 0,05,
/// `1 - 2^-32` et `1` (`TOUJOURS`, sans tirage), des amas de 30, 100 puis 200 sites (le même `AmasVariante`, grandi par
/// `croitre_jusqu_a`) : à chaque étape les `sites()` sont EXACTEMENT ceux du simulateur naïf, l'état du générateur (les 16 derniers octets de
/// l'écriture canonique) aussi, et tous les compteurs valent 0. Le test voit : un collage au lieu d'un tirage, un tirage de moins ou de plus
/// par contact (l'état du générateur diverge), un `<=` pour un `<` (voir `seuil_de_collage_egal_a_un_tirage`), un pas refusé contre l'amas
/// remplacé par un pas permis ou par un autre tirage, un nouveau test de contact oublié après un pas refusé, un tirage pris ailleurs que
/// dans `suivant()`, un lancement refait après un collage manqué, un marcheur qui colle sur sa position précédente.
///
/// Ne couvre pas : les amas de plus de 200 sites (le simulateur naïf y est lent), ni le tirage au seuil exact (autre test).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn collage_egal_au_simulateur_naif() {
    etape_2d_commun::avec_delai(DELAI, || {
        for p in [MOITIE, CINQUIEME, VINGTIEME, PRESQUE, DEUX_32] {
            for graine in 1..=8_u64 {
                let variante = Variante::Collage(p);
                let mut naif = Naif::new(graine, 60, variante);
                let mut amas = AmasVariante::new(graine, 60, variante);
                for n in [30, 100, 200] {
                    amas.croitre_jusqu_a(n);
                    naif.croitre(n);
                    comparer(&amas, &naif, &format!("collage p = {p}, graine {graine}, {n} sites"));
                }
            }
        }
    });
}

/// juste : le simulateur naïf (V-VAR-02), pour la variante BRUIT : `m` = 1, 2, 3 (graines 1 à 8, amas de 30, 100 puis 200 sites), `m` = 5 (graines 1 à 5) et
/// `m` = 8 (graines 1 à 4), amas de 30 puis 100 sites (le simulateur naïf est lent quand `m` est grand) :
/// à chaque étape les `sites()`, l'état du générateur et le compteur de CHAQUE case du réseau (`compteur(x, y)`, 0 pour un site de l'amas)
/// sont EXACTEMENT ceux du simulateur naïf. Le test voit : un compteur mis à zéro au mauvais moment (par marcheur, ou quand un autre site
/// est touché), un compteur incrémenté ailleurs qu'à la position du marcheur (le site voisin de l'amas, la position précédente), un seuil
/// `>` pour `>=` (ou `m + 1`), un marcheur qui continue sa marche après un compteur trop bas au lieu d'être abandonné, un lancement avec
/// un `R` différent ou sans tirage, un compteur partagé entre deux sites (mauvais indice), un site rempli sans que son compteur ait atteint `m`.
///
/// Ne couvre pas : les amas de plus de 200 sites (de plus de 100 pour `m` >= 5) ; la capacité (voir `capacite_et_variantes`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn bruit_egal_au_simulateur_naif() {
    etape_2d_commun::avec_delai(DELAI, || {
        // (m, graines, tailles) : le simulateur naïf est lent quand m est grand (chaque site coûte m lancements), les cas s'allègent avec m
        let cas: [(u32, std::ops::RangeInclusive<u64>, &[usize]); 5] = [
            (1, 1..=8, &[30, 100, 200]),
            (2, 1..=8, &[30, 100, 200]),
            (3, 1..=8, &[30, 100, 200]),
            (5, 1..=5, &[30, 100]),
            (8, 1..=4, &[30, 100]),
        ];
        for (m, graines, tailles) in cas {
            for graine in graines {
                let variante = Variante::Bruit(m);
                let mut naif = Naif::new(graine, 60, variante);
                let mut amas = AmasVariante::new(graine, 60, variante);
                for &n in tailles {
                    amas.croitre_jusqu_a(n);
                    naif.croitre(n);
                    comparer(&amas, &naif, &format!("bruit m = {m}, graine {graine}, {n} sites"));
                }
            }
        }
    });
}

/// juste : le collage se décide par `tirage < p`, STRICTEMENT (V-VAR-03). Pour les graines 1 à 8, le simulateur naïf à `p = 0,5` donne le
/// PREMIER tirage de contact `u0` (il ne dépend pas de `p`, avant le premier contact) ; on prend alors `p = u0` (le tirage est égal à `p` :
/// il ne colle PAS) et `p = u0 + 1` (il colle) : pour chacun, 80 sites de `AmasVariante` sont EXACTEMENT ceux du simulateur naïf, état du
/// générateur compris, et la première particule consomme plus de tirages avec `p = u0` qu'avec `p = u0 + 1` (le test du test : les deux histoires
/// diffèrent bien ; elles peuvent se rejoindre plus tard, le hasard les remet en phase). Un `<=` pour un `<` (ou un `u + 1 < p`) est vu à coup sûr, ce que des graines au hasard
/// ne verraient qu'une fois sur `2^32`.
///
/// Ne couvre pas : le tirage au seuil aux contacts suivants (le simulateur naïf compare tous les tirages de la même façon).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn seuil_de_collage_egal_a_un_tirage() {
    etape_2d_commun::avec_delai(DELAI, || {
        for graine in 1..=8_u64 {
            let mut sonde = Naif::new(graine, 60, Variante::Collage(MOITIE));
            sonde.croitre(5);
            let u0 = u64::from(sonde.tirages_de_contact[0]);
            assert!(u0 >= 1, "graine {graine} : le premier tirage de contact vaut 0, p = 0 est interdit");
            let mut etats = Vec::new();
            for p in [u0, u0 + 1] {
                let variante = Variante::Collage(p);
                let mut naif = Naif::new(graine, 60, variante);
                let mut amas = AmasVariante::new(graine, 60, variante);
                amas.croitre_jusqu_a(80);
                naif.croitre(80);
                comparer(&amas, &naif, &format!("collage p = {p} (u0 = {u0}), graine {graine}"));
                // l'état du générateur après la PREMIÈRE particule : avec p = u0 elle consomme plus de tirages (le premier contact échoue)
                let mut premiere = Naif::new(graine, 60, variante);
                premiere.croitre(2);
                etats.push(premiere.alea.octets());
            }
            assert_ne!(
                etats[0], etats[1],
                "graine {graine} : avec p = u0 et p = u0 + 1 la première particule ne consomme pas pareil"
            );
        }
    });
}

/// juste : le tirage de collage aux deux EXTRÉMITÉS de ses valeurs (V-VAR-04). Deux graines FABRIQUÉES par inversion du générateur (comme à l'étape 0 :
/// `graines_extremes.py` de la session, l'état est remonté jusqu'à l'amorçage) dont le PREMIER tirage de contact de l'amas vaut `2^32 − 1`
/// (`16191125048797634697` : le septième tirage de l'amas) et `0` (`16075594526346206209` : le sixième) : le simulateur naïf le confirme ici. Pour la première, avec
/// `p` = 0,5, `1 − 2^-32` (le tirage est alors le plus grand possible et ne colle PAS : `u < p` est faux) et 0,2, puis pour la seconde, avec `p` = 0,5, 0,2 et 0,05
/// (le tirage est nul : il colle toujours), 60 sites de `AmasVariante` sont EXACTEMENT ceux du simulateur naïf, état du générateur compris. Un tirage fait par
/// `borne(2^32 − 1)` ou par une conversion en flottant ou en `i32` (qui déforme les valeurs extrêmes) est vu à coup sûr.
///
/// Ne couvre pas : les tirages intermédiaires (le simulateur naïf et les graines ordinaires les comparent tous).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn tirage_de_collage_aux_extremes() {
    etape_2d_commun::avec_delai(DELAI, || {
        let cas: [(u64, u32, [u64; 3]); 2] = [
            (16_191_125_048_797_634_697, u32::MAX, [MOITIE, PRESQUE, CINQUIEME]),
            (16_075_594_526_346_206_209, 0, [MOITIE, CINQUIEME, VINGTIEME]),
        ];
        for (graine, extreme, probabilites) in cas {
            let mut sonde = Naif::new(graine, 60, Variante::Collage(MOITIE));
            sonde.croitre(2);
            assert_eq!(sonde.tirages_de_contact[0], extreme, "graine {graine} : le premier tirage de contact");
            for p in probabilites {
                let variante = Variante::Collage(p);
                let mut naif = Naif::new(graine, 60, variante);
                let mut amas = AmasVariante::new(graine, 60, variante);
                amas.croitre_jusqu_a(60);
                naif.croitre(60);
                comparer(&amas, &naif, &format!("collage p = {p}, graine fabriquée {graine}"));
            }
        }
    });
}

// ------------------------------------------------------------------------------------------------- le lien avec l'étape 1

/// juste : `Collage(TOUJOURS)` et `Bruit(1)` redonnent `Amas` de l'étape 1, appel par appel (V-NEUT-01) : pour les graines 1 à 12 et 500
/// particules, chaque `ajouter()` rend le même site que `Amas::ajouter()`, `rayon()` est le même, et l'état du générateur (les 16 derniers
/// octets des deux écritures canoniques) aussi : aucun tirage de plus ni de moins. Puis, sur un réseau de `demi_cote` 0, 1 et 3 et vingt
/// graines, 70 appels de `essayer_ajouter` rendent les mêmes `Some(site)` ou `None` que ceux d'`Amas`, avec le même état du
/// générateur après chacun : le dépassement de la capacité a lieu au même appel. Les deux écritures canoniques sont, elles, différentes
/// (étiquette et paramètre).
///
/// Ne couvre pas : les amas de plus de 500 sites (la comparaison est exacte, donc elle porterait sur plus de la même chose).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn valeurs_neutres_redonnent_l_amas_de_l_etape_1() {
    etape_2d_commun::avec_delai(DELAI, || {
        for variante in [Variante::Collage(DEUX_32), Variante::Bruit(1)] {
            for graine in 1..=12_u64 {
                let mut reference = Amas::new(graine, 150);
                let mut amas = AmasVariante::new(graine, 150, variante);
                for k in 1..500 {
                    assert_eq!(amas.ajouter(), reference.ajouter(), "{variante:?}, graine {graine}, particule {k}");
                }
                assert_eq!(amas.sites(), reference.sites());
                assert_eq!(amas.rayon(), reference.rayon());
                let (a, b) = (amas.ecriture_canonique(), reference.ecriture_canonique());
                assert_eq!(a[a.len() - 16..], b[b.len() - 16..], "{variante:?}, graine {graine} : état du générateur");
                assert_ne!(a, b, "l'écriture canonique d'une variante n'est pas celle de la DLA");
            }
            for demi_cote in [0_u32, 1, 3] {
                for graine in 1..=20_u64 {
                    let mut reference = Amas::new(graine, demi_cote);
                    let mut amas = AmasVariante::new(graine, demi_cote, variante);
                    for appel in 0..70 {
                        let (attendu, obtenu) = (reference.essayer_ajouter(), amas.essayer_ajouter());
                        assert_eq!(
                            obtenu, attendu,
                            "{variante:?}, demi_cote {demi_cote}, graine {graine}, appel {appel}"
                        );
                        let (a, b) = (amas.ecriture_canonique(), reference.ecriture_canonique());
                        assert_eq!(a[a.len() - 16..], b[b.len() - 16..], "état du générateur, appel {appel}");
                    }
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------------------------------------------- capacité

/// juste : la capacité (V-CAP2-01), contre le simulateur naïf muni du même réseau. Pour les variantes `Collage(0,5)`, `Collage(0,05)`,
/// `Bruit(2)`, `Bruit(3)` et `Bruit(5)`, un réseau de `demi_cote` 0, 1 et 3 et vingt graines, 70 appels de `essayer_ajouter` : chaque
/// appel rend le même `Some(site)` ou `None` que le simulateur, l'état du générateur est le même après chacun, les sites aussi, et les
/// compteurs de toutes les cases du réseau (donc un `None` ne change pas les sites, garde les compteurs que les marcheurs abandonnés de cet appel
/// avaient incrémentés, et les appels suivants reprennent de cet état). Avec `Bruit`, un contact HORS du réseau rend `None` tout de suite, quel que soit `m` : un `Bruit(m)` qui
/// donnerait un compteur aux cases hors du réseau, ou qui attendrait `m` contacts dehors, diverge du simulateur. Et, par le principe des tiroirs
/// (au plus `(2 demi_cote + 1)² − 1` sites entrent), chaque graine rend au moins `70 − ((2 demi_cote + 1)² − 1)` fois `None` (mesuré : 22 à 70).
///
/// Ne couvre pas : la valeur exacte du seuil où la particule déborde (elle dépend du hasard), ni les très grands réseaux.
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn capacite_et_variantes() {
    etape_2d_commun::avec_delai(DELAI, || {
        for variante in [
            Variante::Collage(MOITIE),
            Variante::Collage(VINGTIEME),
            Variante::Bruit(2),
            Variante::Bruit(3),
            Variante::Bruit(5),
        ] {
            for demi_cote in [0_u32, 1, 3] {
                let cases = (2 * demi_cote as usize + 1).pow(2);
                for graine in 1..=20_u64 {
                    let mut nones = 0;
                    let mut naif = Naif::new(graine, demi_cote, variante);
                    let mut amas = AmasVariante::new(graine, demi_cote, variante);
                    for appel in 0..70 {
                        let (attendu, obtenu) = (naif.particule(), amas.essayer_ajouter());
                        let contexte = format!("{variante:?}, demi_cote {demi_cote}, graine {graine}, appel {appel}");
                        assert_eq!(obtenu, attendu, "{contexte}");
                        comparer(&amas, &naif, &contexte);
                        nones += usize::from(obtenu.is_none());
                    }
                    assert!(
                        nones >= 70 - (cases - 1),
                        "{variante:?}, demi_cote {demi_cote}, graine {graine} : {nones} dépassements, au moins {} attendus",
                        70 - (cases - 1)
                    );
                }
            }
        }
    });
}

/// juste : `ajouter` et `croitre_jusqu_a` PANIQUENT avec un message contenant « capacité » quand la particule colle (ou touche, avec
/// `Bruit`) hors du réseau (doc de `AmasVariante`) (V-CAP2-02) : `ajouter()` sur un réseau de `demi_cote` 0 pour `Collage(0,5)`, `Collage(1)`
/// et `Bruit(1)`, `Bruit(2)`, `Bruit(7)` ; `croitre_jusqu_a(60)` sur un réseau de 49 cases (principe des tiroirs) pour deux variantes ;
/// aucun site n'est ajouté par l'appel qui panique ; et `AmasVariante::new` refuse un `demi_cote` au-delà de `DEMI_COTE_MAX` (2047).
///
/// Ne couvre pas : le message exact, ni le message de la seconde panique.
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn capacite_ajouter_panique() {
    etape_2d_commun::avec_delai(DELAI, || {
        let message = |panique: Box<dyn std::any::Any + Send>| {
            panique.downcast_ref::<String>().cloned().or_else(|| panique.downcast_ref::<&str>().map(|s| s.to_string()))
        };
        for variante in [
            Variante::Collage(MOITIE),
            Variante::Collage(DEUX_32),
            Variante::Bruit(1),
            Variante::Bruit(2),
            Variante::Bruit(7),
        ] {
            let panique = std::panic::catch_unwind(move || AmasVariante::new(5, 0, variante).ajouter())
                .expect_err("ajouter devait paniquer");
            assert!(
                message(panique).is_some_and(|m| m.contains("capacité")),
                "{variante:?} : message sans « capacité »"
            );
            let mut amas = AmasVariante::new(5, 0, variante);
            assert_eq!(amas.essayer_ajouter(), None);
            assert_eq!(amas.sites(), [(0, 0)]);
        }
        for variante in [Variante::Collage(CINQUIEME), Variante::Bruit(2)] {
            let panique = std::panic::catch_unwind(move || AmasVariante::new(1, 3, variante).croitre_jusqu_a(60))
                .expect_err("croitre_jusqu_a devait paniquer");
            assert!(message(panique).is_some_and(|m| m.contains("capacité")), "{variante:?}");
        }
        assert!(std::panic::catch_unwind(|| AmasVariante::new(1, 2048, Variante::Bruit(2))).is_err());
        assert_eq!(AmasVariante::new(1, 2047, Variante::Collage(MOITIE)).demi_cote(), 2047);
    });
}

// -------------------------------------------------------------------------------- le code indépendant (Python, hashlib)

/// juste : les premiers sites de onze couples (graine, variante), puis le SHA-256 de l'écriture canonique, calculés par le code Python
/// indépendant `ref_variantes.py` (V-IND-01). Les premiers sites (douze) : `Collage` à `p` = 0,5 (graines 1, 1000), 0,2 (graine 2), 0,05 (graine 3),
/// `1 - 2^-32` (graine 7) ; `Bruit` à `m` = 2 (graine 1), 3 (graine 2), 8 (graine 3), 5 (graine 42) ; et les deux graines FABRIQUÉES
/// `6848047025811403090` et `10997253461852598610` (le premier tirage de `Alea::new` vaut 15 et 16 : celui du lancement est rejeté, puis accepté, par
/// `borne(40)`, comme à l'étape 1). Puis le SHA-256 de l'écriture canonique de 200 sites (`demi_cote` 300) pour ces neuf couples, et de 60
/// sites pour les deux graines fabriquées (l'écriture canonique du `Bruit` comprend les compteurs des sites vides et celle du `Collage` n'en
/// a pas : voir la doc). Le code Python et le prototype Rust donnent les mêmes valeurs ; le test juge la bibliothèque contre le Python.
///
/// Ne couvre pas : la capacité (V-CAP2-01) ; les amas de plus de 200 sites.
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn valeurs_du_code_independant() {
    etape_2d_commun::avec_delai(DELAI, || {
        #[rustfmt::skip]
        let premiers: [(u64, Variante, Douze); 11] = [
            (1, Variante::Collage(MOITIE), [(0, 0), (0, -1), (1, 0), (-1, -1), (1, -1), (0, 1), (0, 2), (0, 3), (-1, 0), (2, -1), (0, 4), (3, -1)]),
            (1000, Variante::Collage(MOITIE), [(0, 0), (1, 0), (-1, 0), (1, 1), (2, 0), (-1, 1), (3, 0), (-2, 1), (3, -1), (-2, 0), (-2, 2), (-1, -1)]),
            (2, Variante::Collage(CINQUIEME), [(0, 0), (1, 0), (0, 1), (1, 1), (2, 0), (2, -1), (2, -2), (3, -2), (4, -2), (4, -3), (3, -3), (1, -1)]),
            (3, Variante::Collage(VINGTIEME), [(0, 0), (0, 1), (0, 2), (-1, 2), (1, 2), (-2, 2), (0, 3), (2, 2), (1, 0), (0, -1), (-3, 2), (0, -2)]),
            (7, Variante::Collage(PRESQUE), [(0, 0), (0, 1), (0, 2), (-1, 2), (-1, 3), (-1, 0), (-1, 4), (-1, 5), (-2, 0), (0, 4), (-2, 3), (-1, 6)]),
            (1, Variante::Bruit(2), [(0, 0), (-1, 0), (0, -1), (-2, 0), (-3, 0), (0, 1), (-3, -1), (1, 0), (-2, 1), (1, -1), (0, 2), (0, -2)]),
            (2, Variante::Bruit(3), [(0, 0), (0, -1), (-1, 0), (1, -1), (0, -2), (-2, 0), (2, -1), (-3, 0), (0, 1), (-4, 0), (3, -1), (0, -3)]),
            (3, Variante::Bruit(8), [(0, 0), (0, -1), (0, 1), (-1, -1), (1, 0), (-2, -1), (0, 2), (0, -2), (0, 3), (-3, -1), (1, -1), (-4, -1)]),
            (42, Variante::Bruit(5), [(0, 0), (-1, 0), (-2, 0), (-2, 1), (1, 0), (-3, 0), (-2, 2), (0, 1), (-4, 0), (1, -1), (2, 0), (-3, -1)]),
            (6_848_047_025_811_403_090, Variante::Collage(MOITIE), [(0, 0), (-1, 0), (-1, -1), (-2, -1), (-2, -2), (-1, 1), (1, 0), (-2, 1), (1, 1), (-3, -1), (-2, -3), (2, 1)]),
            (10_997_253_461_852_598_610, Variante::Bruit(2), [(0, 0), (-1, 0), (0, -1), (-2, 0), (-2, 1), (0, -2), (0, 1), (-2, 2), (-3, 1), (1, 1), (-4, 1), (0, -3)]),
        ];
        for (graine, variante, sites) in premiers {
            let mut amas = AmasVariante::new(graine, 300, variante);
            amas.croitre_jusqu_a(12);
            assert_eq!(amas.sites(), sites, "graine {graine}, {variante:?}");
        }
        let sha = [
            (1_u64, Variante::Collage(MOITIE), 200, "caacc55937ba334662da5ccdaa9b14ce60ee0b95a7d3d547eb6230e861406798"),
            (2, Variante::Collage(CINQUIEME), 200, "df091f4698ef2e1c89d805b19393d9727696fd22cce42a995756965c07edec46"),
            (3, Variante::Collage(VINGTIEME), 200, "07e98f38df57b82be1788b3837cbd63492c742c186178ddfa9af138a08be6d8b"),
            (7, Variante::Collage(PRESQUE), 200, "9a9caf159dc6a26d27fb0dbeb413ed4a8859370d751c0df303383517be960ac7"),
            (1000, Variante::Collage(MOITIE), 200, "f775351d62b36ae7ed2a17e30fa666c4a4694bcdd5cc3aa05d04b4d1de7e2e56"),
            (1, Variante::Bruit(2), 200, "060801ee64e3c3b149b9c37f1d8ad4fceb664372e1b6ccc7fa52c6a38d54e2df"),
            (2, Variante::Bruit(3), 200, "7ee1a0b11d4e9e2455b6ec80dd0720b6a076777a26763e0908eb1722b9f68818"),
            (3, Variante::Bruit(8), 200, "d31b2059df4ade59f37f73cb7c4a36c197f130ba0be1e1170f9302da3bf93943"),
            (42, Variante::Bruit(5), 200, "c6bdaed17935a42b9aec57225aa6980e4b38c246398a9365cb1b1e254bf88ede"),
            (
                6_848_047_025_811_403_090,
                Variante::Collage(MOITIE),
                60,
                "6bf5decc163fa0760cfebd16edd02db1fa64b5327180bb0434f88827f37f86d8",
            ),
            (
                10_997_253_461_852_598_610,
                Variante::Bruit(2),
                60,
                "52fd6af1b116596376bdd5e72f87f92ca68c442c44e15fe0b166f30ade30f43a",
            ),
        ];
        for (graine, variante, n, attendu) in sha {
            let mut amas = AmasVariante::new(graine, 300, variante);
            amas.croitre_jusqu_a(n);
            assert_eq!(
                givre_lab::empreinte::sha256_hex(&amas.ecriture_canonique()),
                attendu,
                "graine {graine}, {variante:?}, {n} sites"
            );
            assert_eq!(
                amas.empreinte(),
                attendu,
                "graine {graine} : l'empreinte est le SHA-256 de l'écriture canonique"
            );
        }
    });
}

// ------------------------------------------------------------------------------------------- écriture canonique, empreinte

/// juste : l'écriture canonique, octet par octet (V-CAN2-01). (1) Le germe seul, `demi_cote` 5, graine 7, `Collage(0,5)` : `GIVRE2D0`
/// (`47 49 56 52 45 32 44 30`), `00` (collage), `p = 2^31` en `u64` petit-boutiste (`00 00 00 80 00 00 00 00`), `05 00 00 00`, `01 00 00 00`, le germe
/// (huit octets nuls), l'état de `Alea::new(7)` (`d1a0bdb386a42205 6d00000000000000`, celui de l'étape 0) : 49 octets. (2) `Collage(TOUJOURS)` :
/// `p = 2^32` s'écrit `00 00 00 00 01 00 00 00` (un `u32` ne le tiendrait pas). (3) `Bruit(3)` : `01`, `03` sur huit octets, puis, après le
/// germe, `00 00 00 00` (aucun site vide touché), 53 octets. (4) `Bruit(2)`, graine 1, `demi_cote` 9, après UNE particule : le site `(-1, 0)` est
/// entré (le germe est à `(0, 0)`) et deux sites vides ont été touchés une fois, `(0, -1)` puis `(1, 0)` (balayage ligne par ligne : la ligne
/// `y = -1` avant la ligne `y = 0`), donc `c = 2` et, pour chacun, `x`, `y` (`i32`) puis le compteur (`u32`), tous petit-boutistes : 85 octets ;
/// les compteurs lus par `compteur(x, y)` sont 1, 1, et 0 sur le site entré `(-1, 0)` et sur `(0, 1)` ; les SHA-256 sont ceux de Python
/// `hashlib`. (5) Après DEUX particules : 105 octets, trois sites vides touchés. Une écriture grand-boutiste, un `p` sur quatre octets, une
/// étiquette de l'étape 1, un balayage par colonnes, un compteur de site entré gardé ou l'état du générateur oublié échouent ici.
///
/// Ne couvre pas : les amas de plus de trois sites (le test de SHA-256 de `valeurs_du_code_independant`).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn ecriture_canonique_a_la_main() {
    etape_2d_commun::avec_delai(DELAI, || {
        assert_eq!(ETIQUETTE, *b"GIVRE2D0");
        let etat_7 = "d1a0bdb386a422056d00000000000000"; // Alea::new(7).octets()
        let etiquette = "4749565245324430"; // « GIVRE2D0 »
        let germe_nul = "0000000000000000"; // le germe (0, 0) : x puis y

        let germe = AmasVariante::new(7, 5, Variante::Collage(MOITIE));
        let attendu = [etiquette, "00", "0000008000000000", "05000000", "01000000", germe_nul, etat_7].concat();
        assert_eq!(hex(&germe.ecriture_canonique()), attendu);
        assert_eq!(germe.ecriture_canonique().len(), 49);
        assert_eq!(germe.empreinte(), "73b637427c7051f92f0c5b4f8765fa8549fbf023626b5b51b09b0d246cdc972f");

        let toujours = AmasVariante::new(7, 5, Variante::Collage(TOUJOURS));
        let attendu = [etiquette, "00", "0000000001000000", "05000000", "01000000", germe_nul, etat_7].concat();
        assert_eq!(hex(&toujours.ecriture_canonique()), attendu);
        assert_eq!(toujours.empreinte(), "a9969a140eef8ba5afe1f889bdbebcd420ec2343eda6d1e7f5a3e15d8e569cc7");

        let bruit = AmasVariante::new(7, 5, Variante::Bruit(3));
        let attendu =
            [etiquette, "01", "0300000000000000", "05000000", "01000000", germe_nul, "00000000", etat_7].concat();
        assert_eq!(hex(&bruit.ecriture_canonique()), attendu);
        assert_eq!(bruit.ecriture_canonique().len(), 53);
        assert_eq!(bruit.empreinte(), "662898784251eca9d91d29e7b47f7bf19425d909ca647091ea386859fa2dc670");
        let un = AmasVariante::new(7, 5, Variante::Bruit(1));
        assert_eq!(un.empreinte(), "6c5b54e501084cce4efe97e2d30cc9c0dfdc5e31c2599f3d647c1b0118af944d");

        let mut amas = AmasVariante::new(1, 9, Variante::Bruit(2));
        amas.croitre_jusqu_a(2);
        assert_eq!(amas.sites(), [(0, 0), (-1, 0)]);
        assert_eq!(
            [amas.compteur(0, -1), amas.compteur(1, 0), amas.compteur(-1, 0), amas.compteur(0, 1)],
            [1, 1, 0, 0]
        );
        let attendu = [
            etiquette,
            "01",
            "0200000000000000",
            "09000000",
            "02000000",
            germe_nul,
            "ffffffff00000000",         // le site (-1, 0)
            "02000000",                 // c = 2 sites vides touchés
            "00000000ffffffff01000000", // (0, -1), compteur 1
            "010000000000000001000000", // (1, 0), compteur 1
            "2de9e38a2030a79b6d00000000000000",
        ]
        .concat();
        assert_eq!(hex(&amas.ecriture_canonique()), attendu);
        assert_eq!(amas.ecriture_canonique().len(), 85);
        assert_eq!(amas.empreinte(), "09cff9eba7137d99c293ad50ed889b320a0f9e7493eceb682861bc3a1dc6ad9c");

        amas.croitre_jusqu_a(3);
        assert_eq!(amas.sites(), [(0, 0), (-1, 0), (0, -1)]);
        assert_eq!(amas.ecriture_canonique().len(), 105);
        assert_eq!(amas.empreinte(), "35d27b875074f2b988817c8df6e9c67523be3ac57b1cedb42c305a11ae1d10e7");
    });
}

// ------------------------------------------------------------------------------------------------- les invariants

/// juste : les propriétés qui valent pour toute graine (V-INV-01), pour `Collage(0,5)`, `Collage(0,05)`, `Bruit(2)` et `Bruit(4)`, graines 1 à 6, 120
/// sites : le premier site est le germe ; tous distincts ; chaque site (sauf le germe) a un voisin PAR UN CÔTÉ parmi les sites arrivés avant lui
/// (un collage par la diagonale ou sur la position précédente viole cela) ; tous dans le réseau ; `contient` est vrai exactement sur les sites ;
/// `rayon()` est le plus petit entier `r` tel que tous les sites vérifient `x² + y² <= r²` ; `ajouter()` rend le dernier site. Avec `Bruit(m)`,
/// à chaque particule : le compteur du site qui entre valait moins de `m` avant l'appel et vaut 0 ensuite (un site de l'amas n'a pas de
/// compteur), aucun autre compteur n'a diminué, tout compteur non nul est sur un site VIDE au contact de l'amas par un côté, et reste strictement
/// inférieur à `m` ; avec `Collage`, tous les compteurs valent 0.
///
/// Ne couvre pas : que le collage se fasse au PREMIER contact, ni la loi du hasard (le simulateur naïf).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn proprietes_des_amas() {
    etape_2d_commun::avec_delai(DELAI, || {
        for variante in
            [Variante::Collage(MOITIE), Variante::Collage(VINGTIEME), Variante::Bruit(2), Variante::Bruit(4)]
        {
            for graine in 1..=6_u64 {
                let mut amas = AmasVariante::new(graine, 60, variante);
                assert_eq!(amas.sites(), [(0, 0)]);
                assert_eq!(amas.rayon(), 0);
                assert_eq!((amas.demi_cote(), amas.variante()), (60, variante));
                let mut avant: BTreeSet<(i32, i32)> = BTreeSet::from([(0, 0)]);
                let cases: Vec<(i32, i32)> = (-60..=60).flat_map(|y| (-60..=60).map(move |x| (x, y))).collect();
                for k in 1..120 {
                    let compteurs_avant: Vec<u32> = cases.iter().map(|&(x, y)| amas.compteur(x, y)).collect();
                    let site = amas.ajouter();
                    assert_eq!(amas.sites().len(), k + 1);
                    assert_eq!(amas.sites().last(), Some(&site), "ajouter() rend le dernier site");
                    let contexte = format!("{variante:?}, graine {graine}, site {k} = {site:?}");
                    assert!(
                        DIRECTIONS.iter().any(|&(dx, dy)| avant.contains(&(site.0 + dx, site.1 + dy))),
                        "{contexte} : aucun voisin par un côté parmi les sites précédents"
                    );
                    assert!(avant.insert(site), "{contexte} : déjà dans l'amas");
                    assert!(site.0.abs() <= 60 && site.1.abs() <= 60, "dans le réseau");
                    for (&(x, y), &avant_ce_site) in cases.iter().zip(&compteurs_avant) {
                        let apres = amas.compteur(x, y);
                        match variante {
                            Variante::Collage(_) => {
                                assert_eq!(apres, 0, "{contexte} : compteur de ({x}, {y}) avec Collage")
                            }
                            Variante::Bruit(m) if (x, y) == site => {
                                assert!(
                                    avant_ce_site < m,
                                    "{contexte} : le compteur du site qui entre valait {avant_ce_site} avant l'appel, m = {m}"
                                );
                                assert_eq!(apres, 0, "{contexte} : un site de l'amas n'a pas de compteur");
                            }
                            Variante::Bruit(m) => {
                                assert!(apres >= avant_ce_site, "{contexte} : le compteur de ({x}, {y}) a diminué");
                                assert!(apres < m, "{contexte} : compteur ({x}, {y}) = {apres}, m = {m}");
                                if apres > 0 {
                                    assert!(!avant.contains(&(x, y)), "{contexte} : un site de l'amas a un compteur");
                                    assert!(
                                        DIRECTIONS.iter().any(|&(dx, dy)| avant.contains(&(x + dx, y + dy))),
                                        "{contexte} : ({x}, {y}) touché sans être au contact de l'amas"
                                    );
                                }
                            }
                        }
                    }
                }
                let r = amas.rayon() as i32;
                let plus_grand = amas.sites().iter().map(|&(x, y)| x * x + y * y).max().unwrap();
                assert!(
                    r * r >= plus_grand && (r == 0 || (r - 1) * (r - 1) < plus_grand),
                    "rayon {r}, plus grand carré {plus_grand}"
                );
                assert_eq!(r, rayon_naif(amas.sites()));
                for &(x, y) in &cases {
                    assert_eq!(amas.contient(x, y), avant.contains(&(x, y)), "contient({x}, {y})");
                }
                assert!(
                    !amas.contient(i32::MAX, i32::MIN) && !amas.contient(i32::MIN, 0) && !amas.contient(0, i32::MAX)
                );
                assert_eq!(amas.compteur(i32::MAX, 0), 0, "compteur hors du réseau : 0, sans panique");
            }
        }
    });
}

// ---------------------------------------------------------------------------------------------- graines et paramètres

/// juste : même graine et même variante, même amas ; deux graines, deux amas et deux empreintes ; deux variantes différentes (ou deux paramètres
/// différents) à la même graine, deux empreintes ; `croitre_jusqu_a(n)` ne fait rien quand l'amas a déjà `n` sites ou plus (l'écriture
/// canonique ne change pas), ni `croitre_jusqu_a(0)` et `(1)` sur un germe ; deux amas de `demi_cote` différents (assez grands) ont les mêmes
/// sites mais des écritures canoniques différentes, et grandir en deux temps (`(40)` puis `(100)`) donne la même écriture que `(100)` seul
/// (V-GRN-01).
///
/// Ne couvre pas : que deux graines donnent des amas « assez différents » (propriété statistique).
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn graines_variantes_et_croissance() {
    etape_2d_commun::avec_delai(DELAI, || {
        for variante in [Variante::Collage(CINQUIEME), Variante::Bruit(3)] {
            let mut amas = AmasVariante::new(9, 80, variante);
            let germe = amas.ecriture_canonique();
            amas.croitre_jusqu_a(0);
            amas.croitre_jusqu_a(1);
            assert_eq!(amas.ecriture_canonique(), germe);
            amas.croitre_jusqu_a(40);
            let a_40 = amas.ecriture_canonique();
            amas.croitre_jusqu_a(40);
            amas.croitre_jusqu_a(25);
            assert_eq!(amas.ecriture_canonique(), a_40);
            amas.croitre_jusqu_a(100);
            let mut en_un_coup = AmasVariante::new(9, 80, variante);
            en_un_coup.croitre_jusqu_a(100);
            assert_eq!(
                amas.ecriture_canonique(),
                en_un_coup.ecriture_canonique(),
                "grandir en deux temps ou d'un coup"
            );

            let (mut a, mut b) = (AmasVariante::new(5, 80, variante), AmasVariante::new(5, 80, variante));
            a.croitre_jusqu_a(100);
            b.croitre_jusqu_a(100);
            assert_eq!(a.sites(), b.sites());
            assert_eq!(a.empreinte(), b.empreinte());
            let mut vus = BTreeSet::new();
            for graine in 1..=8_u64 {
                let mut c = AmasVariante::new(graine, 80, variante);
                c.croitre_jusqu_a(100);
                assert!(vus.insert(c.empreinte()), "la graine {graine} donne une empreinte déjà vue");
                if graine != 5 {
                    assert_ne!(c.sites(), a.sites(), "les graines {graine} et 5 donnent le même amas");
                }
            }
            let mut grand = AmasVariante::new(5, 200, variante);
            grand.croitre_jusqu_a(100);
            assert_eq!(grand.sites(), a.sites(), "le demi_cote ne change pas la dynamique");
            assert_ne!(grand.ecriture_canonique(), a.ecriture_canonique(), "mais il est dans l'écriture canonique");
        }
        let empreinte = |v: Variante| {
            let mut amas = AmasVariante::new(5, 80, v);
            amas.croitre_jusqu_a(60);
            amas.empreinte()
        };
        let toutes = [
            Variante::Collage(MOITIE),
            Variante::Collage(MOITIE + 1),
            Variante::Collage(CINQUIEME),
            Variante::Collage(TOUJOURS),
            Variante::Bruit(1),
            Variante::Bruit(2),
            Variante::Bruit(3),
        ];
        let distinctes: BTreeSet<String> = toutes.iter().map(|&v| empreinte(v)).collect();
        assert_eq!(distinctes.len(), toutes.len(), "chaque variante et chaque paramètre a son empreinte");
    });
}

/// juste : les constantes et les paramètres (V-GRN-02). `TOUJOURS` vaut `2^32` (4294967296), `BRUIT_MAX` un million ; `AmasVariante::new`
/// accepte `Collage(1)`, `Collage(TOUJOURS)`, `Bruit(1)` et `Bruit(BRUIT_MAX)` et rend la variante par `variante()` ; il PANIQUE pour
/// `Collage(0)`, `Collage(TOUJOURS + 1)`, `Bruit(0)` et `Bruit(BRUIT_MAX + 1)`. Un `Collage(1)` (`p = 2^-32`) ne colle presque jamais : on ne
/// le fait pas croître.
///
/// Ne couvre pas : le message des paniques.
#[test]
#[cfg_attr(not(feature = "etape-2"), ignore)]
fn constantes_et_parametres() {
    etape_2d_commun::avec_delai(DELAI, || {
        assert_eq!(TOUJOURS, 4_294_967_296);
        assert_eq!(BRUIT_MAX, 1_000_000);
        for variante in
            [Variante::Collage(1), Variante::Collage(TOUJOURS), Variante::Bruit(1), Variante::Bruit(BRUIT_MAX)]
        {
            assert_eq!(AmasVariante::new(3, 4, variante).variante(), variante);
        }
        for mauvaise in
            [Variante::Collage(0), Variante::Collage(TOUJOURS + 1), Variante::Bruit(0), Variante::Bruit(BRUIT_MAX + 1)]
        {
            assert!(
                std::panic::catch_unwind(move || AmasVariante::new(3, 4, mauvaise)).is_err(),
                "{mauvaise:?} doit paniquer"
            );
        }
    });
}
