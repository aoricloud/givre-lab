//! Étape 4 : la dynamique EXACTE de `cinetique::Face`, essai après essai. Valeurs attendues :
//! `references/etape-4-valeurs.md`, identifiants `V-CIN-xx`.
//!
//! La doc de `cinetique` fixe la dynamique ET la consommation du hasard appel par appel. Ce fichier contient donc un
//! **simulateur naïf** (`Naif`, une quarantaine de lignes de boucles simples écrites d'après la doc seule : le hamiltonien
//! `H = J Σ|h_i − h_j| − Δμ Σ h_i` est recalculé EN ENTIER avant et après chaque proposition, pour que la règle `k = J(2n − 4)
//! ∓ Δμ` de la doc soit vérifiée, pas supposée) et exige que `Face` donne **exactement** ses hauteurs et son générateur.
//! Un `<` pour un `<=`, un voisin oublié, un signe de `Δμ` inversé, un retrait calculé comme un ajout, une table décalée
//! d'un cran, un tirage de trop ou de moins, un balayage qui n'est pas de `L²` essais, un `borne` remplacé par un modulo :
//! chacune de ces fautes change un état que ce simulateur ne change pas. Les cas de bord de l'acceptation (`r` égal au seuil,
//! `k = 0`, `k = 365`, `k = 366`) se testent sur des graines **construites** pour que le tirage voulu tombe exactement
//! (Python : `essai-4-outils/graines_bords.py`, inversion de l'amorçage de PCG32).

use givre_lab::alea::Alea;
use givre_lab::cinetique::Face;
use givre_lab::math::EXP_NEG;
use std::panic::catch_unwind;

/// Le simulateur naïf, d'après la doc de `cinetique` seule. Les hauteurs sont rangées en `h[y][x]`.
struct Naif {
    l: usize,
    j: i64,
    mu: i64,
    h: Vec<Vec<i64>>,
    alea: Alea,
}

impl Naif {
    fn new(l: usize, j: u32, mu: i32, graine: u64, depart: &[i32]) -> Naif {
        let h = (0..l).map(|y| (0..l).map(|x| i64::from(depart[y * l + x])).collect()).collect();
        Naif { l, j: i64::from(j), mu: i64::from(mu), h, alea: Alea::new(graine) }
    }

    /// `H = J · Σ_liaisons |h_i − h_j| − Δμ · Σ h_i`, en seizièmes de `kT` : chaque liaison une fois (la colonne et sa voisine de
    /// droite, la colonne et sa voisine du bas, modulo `L`).
    fn hamiltonien(&self) -> i64 {
        let (mut liaisons, mut somme) = (0, 0);
        for y in 0..self.l {
            for x in 0..self.l {
                liaisons += (self.h[y][x] - self.h[y][(x + 1) % self.l]).abs();
                liaisons += (self.h[y][x] - self.h[(y + 1) % self.l][x]).abs();
                somme += self.h[y][x];
            }
        }
        self.j * liaisons - self.mu * somme
    }

    /// Un essai, tirage après tirage comme dans la doc ; rend vrai s'il est accepté.
    fn essai(&mut self) -> bool {
        let i = self.alea.borne((self.l * self.l) as u32) as usize;
        let (x, y) = (i % self.l, i / self.l);
        let ajout = self.alea.suivant() >> 31 == 1;
        let avant = self.hamiltonien();
        let pas = if ajout { 1 } else { -1 };
        self.h[y][x] += pas;
        let k = self.hamiltonien() - avant;
        let accepte = if k <= 0 {
            true
        } else if k >= 366 {
            false
        } else {
            u64::from(self.alea.suivant()) < EXP_NEG[k as usize]
        };
        if !accepte {
            self.h[y][x] -= pas;
        }
        accepte
    }

    fn hauteurs(&self) -> Vec<i32> {
        self.h.iter().flatten().map(|&v| v as i32).collect()
    }

    /// L'écriture canonique écrite à la main d'après la doc : `GIVRE004`, `L`, `J`, `Δμ`, les hauteurs, le générateur.
    fn octets(&self) -> Vec<u8> {
        let mut o = b"GIVRE004".to_vec();
        o.extend_from_slice(&(self.l as u32).to_le_bytes());
        o.extend_from_slice(&(self.j as u32).to_le_bytes());
        o.extend_from_slice(&(self.mu as i32).to_le_bytes());
        for v in self.hauteurs() {
            o.extend_from_slice(&v.to_le_bytes());
        }
        o.extend_from_slice(&self.alea.octets());
        o
    }
}

fn canonique(face: &Face) -> Vec<u8> {
    let mut octets = Vec::new();
    face.ecrire_canonique(&mut octets);
    octets
}

/// Des hauteurs de départ au hasard dans `-2..=2` (`Alea::new(graine).borne(5) − 2`, colonne après colonne) : une face qui
/// n'est pas plane, où tous les voisinages se produisent.
fn hauteurs_au_hasard(l: usize, graine: u64) -> Vec<i32> {
    let mut alea = Alea::new(graine);
    (0..l * l).map(|_| alea.borne(5) as i32 - 2).collect()
}

/// Un cas du simulateur naïf : côté, `J`, `Δμ` (seizièmes), graine, balayages, graine des hauteurs de départ (`None` : face plane).
struct Cas {
    l: usize,
    j: u32,
    mu: i32,
    graine: u64,
    balayages: usize,
    depart: Option<u64>,
}

const CAS: [Cas; 12] = [
    Cas { l: 8, j: 16, mu: 0, graine: 1, balayages: 40, depart: None },
    Cas { l: 8, j: 8, mu: 4, graine: 2, balayages: 40, depart: None },
    Cas { l: 16, j: 12, mu: -6, graine: 3, balayages: 20, depart: None },
    Cas { l: 8, j: 0, mu: 8, graine: 4, balayages: 40, depart: None },
    Cas { l: 8, j: 0, mu: -8, graine: 5, balayages: 30, depart: None },
    Cas { l: 5, j: 12, mu: 4, graine: 6, balayages: 60, depart: None },
    Cas { l: 3, j: 10, mu: 0, graine: 7, balayages: 200, depart: None },
    Cas { l: 8, j: 20, mu: 24, graine: 8, balayages: 80, depart: None },
    Cas { l: 8, j: 100, mu: 20, graine: 9, balayages: 30, depart: Some(100) },
    Cas { l: 6, j: 40, mu: -12, graine: 10, balayages: 40, depart: Some(101) },
    Cas { l: 4, j: 16, mu: 0, graine: 11, balayages: 60, depart: Some(102) },
    Cas { l: 7, j: 24, mu: 16, graine: 12, balayages: 40, depart: Some(103) },
];

fn face_du_cas(c: &Cas) -> Face {
    match c.depart {
        None => Face::new(c.l, c.j, c.mu, c.graine),
        Some(g) => Face::depuis_hauteurs(c.l, c.j, c.mu, c.graine, &hauteurs_au_hasard(c.l, g)),
    }
}

fn naif_du_cas(c: &Cas) -> Naif {
    let depart = c.depart.map_or(vec![0; c.l * c.l], |g| hauteurs_au_hasard(c.l, g));
    Naif::new(c.l, c.j, c.mu, c.graine, &depart)
}

/// juste : `Face` donne **exactement** (simulateur naïf écrit dans le test, d'après la doc) les hauteurs, la décision de
/// chaque essai et l'état du générateur du simulateur naïf (V-CIN-01), essai après essai, sur douze jeux : côtés 3, 4, 5, 6, 7,
/// 8, 16 ; `J` de 0 à 100 seizièmes ; `Δμ` de -12 à 24 ; faces planes et faces de départ au hasard (où se produisent tous les
/// voisinages et les trois cas `k <= 0`, `1 <= k <= 365`, `k >= 366`) ; graines 1 à 12 ; de 20 à 200 balayages. À chaque essai
/// `Face::essai` rend ce que rend le naïf, et rend `true` si et seulement si une hauteur a changé ; après chaque balayage les
/// octets canoniques (hauteurs **et** générateur) sont égaux à ceux du naïf. Un second `Face` du même cas, mené par
/// `balayages(n)`, arrive aux mêmes octets (un balayage est `L²` essais, ni plus ni moins). Chaque jeu accepte au moins 5
/// essais et en refuse au moins 5 (le test n'est pas vide).
///
/// Ne couvre pas : les très grandes faces (voir `grande_face_rejets_de_borne`), les cas de bord de la table (voir
/// `bords_de_la_table_graines_construites`), les mesures.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn face_egale_au_simulateur_naif() {
    for (numero, c) in CAS.iter().enumerate() {
        let (mut face, mut naif) = (face_du_cas(c), naif_du_cas(c));
        assert_eq!(canonique(&face), naif.octets(), "cas {numero} : état de départ");
        let (mut acceptes, mut refuses) = (0, 0);
        for balayage in 1..=c.balayages {
            for essai in 0..c.l * c.l {
                let somme_avant = face.somme_hauteurs();
                let decision = face.essai();
                assert_eq!(decision, naif.essai(), "cas {numero}, balayage {balayage}, essai {essai} : décision");
                assert_eq!(
                    decision,
                    face.somme_hauteurs() != somme_avant,
                    "cas {numero} : essai() ment sur la hauteur"
                );
                if decision { acceptes += 1 } else { refuses += 1 }
            }
            assert_eq!(face.hauteurs(), naif.hauteurs(), "cas {numero}, balayage {balayage} : hauteurs");
            assert_eq!(canonique(&face), naif.octets(), "cas {numero}, balayage {balayage} : octets canoniques");
        }
        assert!(acceptes >= 5 && refuses >= 5, "cas {numero} vide : {acceptes} acceptés, {refuses} refusés");
        let mut autre = face_du_cas(c);
        autre.balayages(c.balayages);
        assert_eq!(canonique(&autre), naif.octets(), "cas {numero} : balayages(n)");
        let mut un_par_un = face_du_cas(c);
        for _ in 0..c.balayages {
            un_par_un.balayage();
        }
        assert_eq!(canonique(&un_par_un), naif.octets(), "cas {numero} : balayage() répété");
    }
}

/// Un cas de bord : une face plane de 4 × 4 (seize colonnes, `borne(16)` ne rejette jamais et consomme un tirage), une graine
/// construite dont les trois premiers tirages sont (n'importe quoi, le bit de pile voulu, le tirage voulu), `J`, `Δμ`, le sens de
/// l'essai, l'indice `k` attendu, la décision attendue et le nombre de tirages bruts consommés.
struct Bord {
    nom: &'static str,
    graine: u64,
    j: u32,
    mu: i32,
    ajout: bool,
    k: i64,
    r3: Option<u64>,
    accepte: bool,
    tirages: usize,
}

/// Les graines viennent de `essai-4-outils/graines_bords.py` (voir la fiche) : `Alea::new(graine)` tire, dans l'ordre,
/// `a`, `b` avec `b >> 31 = pile`, puis `c = r3`. `EXP_NEG[12] = 2028798896`, `EXP_NEG[20] = 1230528733`, `EXP_NEG[1] =
/// 4034748382` (V-EXP-01 de l'étape 0).
const BORDS: [Bord; 14] = [
    Bord {
        nom: "ajout k=12, r = seuil (refusé)",
        graine: 6_062_922_519_442_009_649,
        j: 4,
        mu: 4,
        ajout: true,
        k: 12,
        r3: Some(2_028_798_896),
        accepte: false,
        tirages: 3,
    },
    Bord {
        nom: "ajout k=12, r = seuil - 1 (accepté)",
        graine: 8_755_884_821_639_507_505,
        j: 4,
        mu: 4,
        ajout: true,
        k: 12,
        r3: Some(2_028_798_895),
        accepte: true,
        tirages: 3,
    },
    Bord {
        nom: "retrait k=20, r = seuil (refusé)",
        graine: 11_125_629_878_168_646_030,
        j: 4,
        mu: 4,
        ajout: false,
        k: 20,
        r3: Some(1_230_528_733),
        accepte: false,
        tirages: 3,
    },
    Bord {
        nom: "retrait k=20, r = seuil - 1 (accepté)",
        graine: 3_765_393_909_872_538_859,
        j: 4,
        mu: 4,
        ajout: false,
        k: 20,
        r3: Some(1_230_528_732),
        accepte: true,
        tirages: 3,
    },
    Bord {
        nom: "ajout k=365, r = 0 (accepté : la dernière entrée vaut 1)",
        graine: 12_322_596_831_189_428_948,
        j: 92,
        mu: 3,
        ajout: true,
        k: 365,
        r3: Some(0),
        accepte: true,
        tirages: 3,
    },
    Bord {
        nom: "ajout k=365, r = 1 (refusé)",
        graine: 10_797_608_377_802_748_302,
        j: 92,
        mu: 3,
        ajout: true,
        k: 365,
        r3: Some(1),
        accepte: false,
        tirages: 3,
    },
    Bord {
        nom: "ajout k=1, r = seuil - 1 (accepté)",
        graine: 16_047_264_031_906_918_798,
        j: 4,
        mu: 15,
        ajout: true,
        k: 1,
        r3: Some(4_034_748_381),
        accepte: true,
        tirages: 3,
    },
    Bord {
        nom: "ajout k=1, r = seuil (refusé)",
        graine: 8_687_028_063_610_811_627,
        j: 4,
        mu: 15,
        ajout: true,
        k: 1,
        r3: Some(4_034_748_382),
        accepte: false,
        tirages: 3,
    },
    Bord {
        nom: "ajout k=366 (au-delà de la table : refusé sans tirage)",
        graine: 4_748_008_964_488_622_478,
        j: 92,
        mu: 2,
        ajout: true,
        k: 366,
        r3: None,
        accepte: false,
        tirages: 2,
    },
    Bord {
        nom: "ajout k=0 (accepté sans tirage)",
        graine: 4_748_008_964_488_622_478,
        j: 4,
        mu: 16,
        ajout: true,
        k: 0,
        r3: None,
        accepte: true,
        tirages: 2,
    },
    Bord {
        nom: "ajout k=-5 (accepté sans tirage)",
        graine: 4_748_008_964_488_622_478,
        j: 4,
        mu: 21,
        ajout: true,
        k: -5,
        r3: None,
        accepte: true,
        tirages: 2,
    },
    Bord {
        nom: "retrait k=370 (refusé sans tirage)",
        graine: 4_522_577_221_040_630_484,
        j: 92,
        mu: 2,
        ajout: false,
        k: 370,
        r3: None,
        accepte: false,
        tirages: 2,
    },
    Bord {
        nom: "retrait k=0 (accepté sans tirage)",
        graine: 4_522_577_221_040_630_484,
        j: 4,
        mu: -16,
        ajout: false,
        k: 0,
        r3: None,
        accepte: true,
        tirages: 2,
    },
    Bord {
        nom: "retrait k=-5 (accepté sans tirage)",
        graine: 4_522_577_221_040_630_484,
        j: 4,
        mu: -21,
        ajout: false,
        k: -5,
        r3: None,
        accepte: true,
        tirages: 2,
    },
];

/// juste : graines construites par l'inversion de l'amorçage de PCG32 (Python), acceptation écrite dans la doc : les
/// **bords de l'acceptation** (V-CIN-02). Sur une face plane de 4 × 4, un essai donne `k = 4J − Δμ` (ajout) ou `4J + Δμ`
/// (retrait). Quatorze essais, dont le tirage de décision tombe **exactement** sur le seuil : à `r = EXP_NEG[k]` l'essai est
/// **refusé** (`<` strict), à `EXP_NEG[k] − 1` il est accepté (`k = 12`, `20`, `1`) ; pour `k = 365` (`EXP_NEG[365] = 1`),
/// `r = 0` est accepté et `r = 1` refusé ; pour `k = 366`, `k = 370` (au-delà de la table) l'essai est refusé **sans tirage**,
/// pour `k = 0` et `k = -5` accepté **sans tirage**. Pour chacun : la graine donne bien les tirages annoncés (contrôle
/// sur `Alea`), `Face::indice` vaut `k`, la décision est celle attendue, une seule colonne a changé de `±1` (ou aucune), et le
/// générateur est exactement celui d'un `Alea` qui a tiré 2 ou 3 fois (un tirage de trop ou de moins se voit). Elle tue le `<`
/// pour `<=`, le test `k < 0` pour `k <= 0`, la table décalée, la décision sans tirage consommée.
///
/// Ne couvre pas : les faces non planes (le simulateur naïf s'en charge), les valeurs de `k` entre 2 et 11 hors des cas listés.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn bords_de_la_table_graines_construites() {
    for b in &BORDS {
        // contrôle de la construction : les trois premiers tirages de la graine
        let mut alea = Alea::new(b.graine);
        let colonne = alea.borne(16) as usize;
        let pile = alea.suivant() >> 31;
        assert_eq!(pile == 1, b.ajout, "{} : le bit de pile de la graine construite", b.nom);
        if let Some(r3) = b.r3 {
            assert_eq!(u64::from(alea.suivant()), r3, "{} : le troisième tirage de la graine construite", b.nom);
        }
        let mut face = Face::new(4, b.j, b.mu, b.graine);
        let (x, y) = (colonne % 4, colonne / 4);
        assert_eq!(face.indice(x, y, b.ajout), b.k, "{} : indice", b.nom);
        assert_eq!(face.essai(), b.accepte, "{} : décision", b.nom);
        let attendu: Vec<i32> =
            (0..16).map(|i| if b.accepte && i == colonne { if b.ajout { 1 } else { -1 } } else { 0 }).collect();
        assert_eq!(face.hauteurs(), attendu, "{} : hauteurs", b.nom);
        let mut apres = Alea::new(b.graine);
        for _ in 0..b.tirages {
            apres.suivant();
        }
        let octets = canonique(&face);
        assert_eq!(
            octets[octets.len() - 16..],
            apres.octets(),
            "{} : tirages consommés (attendu {})",
            b.nom,
            b.tirages
        );
    }
}

/// Les hauteurs de la face de 4 × 4 des cas à la main, rangées `y * 4 + x` : trois rangées de zéros avec un îlot.
const A_LA_MAIN: [i32; 16] = [0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 3, 0, 0, 0, 0, 0];

/// juste : calculé à la main d'après la doc et recoupé par le simulateur Python, l'**indice `k`** d'un essai sur chaque colonne
/// (V-CIN-03). Face de 4 × 4 `0000 / 0110 / 0130 / 0000` (rangées `y = 0` à `3`), `J = 16`, `Δμ = ±5` (seizièmes). Voisins
/// dans l'ordre `(x+1, y), (x, y+1), (x-1, y), (x, y-1)`, modulo 4. Exemples : la colonne `(1, 1)` (hauteur 1, voisins `1, 1, 0,
/// 0`) : ajouter, `n = 4` voisins `<= 1`, `k = 16·4 − Δμ = 59` (pour `Δμ = 5`) ; ôter, `n = 2` voisins `>= 1`, `k = 16·0 + Δμ = 5`.
/// La colonne `(2, 2)` (hauteur 3, voisins `0, 0, 1, 1`) : ajouter `n = 4`, `k = 59` ; ôter `n = 0`, `k = −64 + 5 = −59`. La
/// colonne `(0, 0)` (plane, voisins `0, 0, 0, 0` en comptant l'enroulement) : ajouter `59`, ôter `69`. Les neuf colonnes listées
/// donnent les 18 valeurs du tableau pour `Δμ = 5` ; pour `Δμ = −5` l'ajout gagne `+10` et le retrait perd `10`.
///
/// Ne couvre pas : les faces de côté 3 (voisins distincts, mais sans cas écrit à la main), la consommation du hasard.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn indice_a_la_main() {
    // (x, y, k d'un ajout, k d'un retrait) pour Δμ = +5
    let attendus: [(usize, usize, i64, i64); 9] = [
        (1, 1, 59, 5),
        (2, 2, 59, -59),
        (0, 0, 59, 69),
        (3, 1, 27, 69),
        (2, 1, 27, 5),
        (1, 2, 27, 5),
        (1, 3, 27, 69),
        (1, 0, 27, 69),
        (3, 3, 59, 69),
    ];
    for mu in [5_i32, -5] {
        let face = Face::depuis_hauteurs(4, 16, mu, 1, &A_LA_MAIN);
        for (x, y, ajout, retrait) in attendus {
            let (ajout, retrait) = if mu == 5 { (ajout, retrait) } else { (ajout + 10, retrait - 10) };
            assert_eq!(face.indice(x, y, true), ajout, "ajout en ({x}, {y}), Δμ = {mu}");
            assert_eq!(face.indice(x, y, false), retrait, "retrait en ({x}, {y}), Δμ = {mu}");
        }
        // l'indice ne tire pas au hasard et ne change rien
        assert_eq!(face.hauteurs(), A_LA_MAIN);
    }
}

/// juste : le hamiltonien est recalculé en entier dans le test ; sur 200 faces de départ au hasard (côtés 3, 4, 5 ; hauteurs
/// de `-2` à `2` ; `J` de 0 à 40 et `Δμ` de -30 à 30 seizièmes, tirés par un générateur de graine fixe), `Face::indice(x, y, ajout)`
/// vaut, pour chaque colonne et chaque sens, **la différence exacte** de `H = J Σ|h_i − h_j| − Δμ Σ h_i` quand la hauteur de la
/// colonne change de `+1` ou `−1` (V-CIN-04). C'est la formule `k = J(2n − 4) ∓ Δμ` de la doc, vérifiée contre sa définition.
///
/// Ne couvre pas : le hasard (autres tests), les faces plus grandes que 5 × 5.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn indice_egal_difference_de_hamiltonien() {
    let mut alea = Alea::new(2026);
    for essai in 0..200 {
        let l = 3 + alea.borne(3) as usize;
        let j = alea.borne(41);
        let mu = alea.borne(61) as i32 - 30;
        let depart = hauteurs_au_hasard(l, 5000 + essai);
        let face = Face::depuis_hauteurs(l, j, mu, 1, &depart);
        let naif = Naif::new(l, j, mu, 1, &depart);
        let avant = naif.hamiltonien();
        for y in 0..l {
            for x in 0..l {
                for ajout in [true, false] {
                    let mut modifie = Naif::new(l, j, mu, 1, &depart);
                    modifie.h[y][x] += if ajout { 1 } else { -1 };
                    let attendu = modifie.hamiltonien() - avant;
                    assert_eq!(
                        face.indice(x, y, ajout),
                        attendu,
                        "jeu {essai}, L={l}, J={j}, Δμ={mu}, ({x},{y}), ajout={ajout}"
                    );
                }
            }
        }
    }
}

/// juste : les tirages sont rejoués à la main ; **`Δμ` très grand : tout ajout est accepté, aucun retrait ne l'est, et aucun
/// tirage n'est consommé pour décider** (V-CIN-05). Face plane de 8 × 8 (`borne(64)` consomme un tirage), `J = 16`,
/// `Δμ = 1000` seizièmes : un ajout a `k <= 64 − 1000 < 0`, un retrait `k >= −64 + 1000 >= 366`. Sur 3000 essais, `essai()` rend
/// `true` si et seulement si le bit de pile vaut 1 ; chaque essai consomme exactement 2 tirages (la colonne puis la pile) ;
/// à la fin chaque colonne a pour hauteur le nombre de fois où elle a été tirée avec pile = 1, et le générateur est celui d'un
/// `Alea` qui a tiré 6000 fois. Même chose pour `Δμ = −1000` (tout retrait accepté, aucun ajout : les hauteurs sont `<= 0`).
///
/// Ne couvre pas : les valeurs de `Δμ` où `k` passe de part et d'autre de 0 ou de 366 (les bords et le simulateur naïf).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn mu_enorme_ajout_toujours_accepte() {
    for (mu, signe) in [(1000_i32, 1_i32), (-1000, -1)] {
        let mut face = Face::new(8, 16, mu, 5);
        let mut alea = Alea::new(5);
        let mut attendu = vec![0_i32; 64];
        for essai in 0..3000 {
            let colonne = alea.borne(64) as usize;
            let pile = alea.suivant() >> 31 == 1;
            let accepte = pile == (signe == 1);
            if accepte {
                attendu[colonne] += signe;
            }
            assert_eq!(face.essai(), accepte, "Δμ = {mu}, essai {essai}");
        }
        assert_eq!(face.hauteurs(), attendu, "Δμ = {mu}");
        let octets = canonique(&face);
        assert_eq!(
            octets[octets.len() - 16..],
            alea.octets(),
            "Δμ = {mu} : le générateur a tiré autre chose que 2 par essai"
        );
        assert!(attendu.iter().any(|&h| h != 0));
    }
}

/// juste : calcul : **sans essai accepté, la face ne change pas** (V-CIN-06). Face plane de 8 × 8 avec `J = 100` seizièmes et
/// `Δμ = 0`, `30` ou `-30` : un ajout et un retrait ont `k = 400`, `370` ou `430`, tous `>= 366` : aucun essai n'est accepté,
/// aucun tirage de décision n'est consommé. Après 5 balayages (320 essais) toutes les hauteurs valent 0, chaque `essai()` a rendu
/// `false`, l'empreinte est celle d'une face plane dont le générateur a tiré `2 × 320` fois, et la somme des hauteurs, la
/// hauteur moyenne et la rugosité valent exactement 0.
///
/// Ne couvre pas : une face plane qui change (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn sans_essai_accepte_la_face_ne_change_pas() {
    for mu in [0_i32, 30, -30] {
        let mut face = Face::new(8, 100, mu, 17);
        let mut alea = Alea::new(17);
        for _ in 0..320 {
            assert!(!face.essai(), "Δμ = {mu} : un essai accepté sur une face plane à J = 100");
            alea.borne(64);
            alea.suivant();
        }
        assert!(face.hauteurs().iter().all(|&h| h == 0));
        assert_eq!((face.somme_hauteurs(), face.somme_carres()), (0, 0));
        assert_eq!((face.hauteur_moyenne(), face.rugosite_carree()), (0.0, 0.0));
        let octets = canonique(&face);
        assert_eq!(octets[octets.len() - 16..], alea.octets(), "Δμ = {mu} : tirages consommés");
    }
}

/// juste : un simulateur naïf creux écrit dans le test : **`borne` avec rejets**. Une face de 3000 × 3000 (9 000 000 colonnes, qui
/// n'est pas une puissance de 2 : `borne` rejette un tirage sur `2096 / 4 294 967` environ, soit 4,6e-4, car `seuil = 2^32 mod n =
/// 1 967 296`) est menée pendant 40 000 essais (`J = 16`, `Δμ = 24` seizièmes) ; le naïf, qui réécrit la boucle de `borne` d'après sa
/// doc (`seuil = n.wrapping_neg() % n`, rejet de `r < seuil`) et n'enregistre que les colonnes touchées, compte les rejets (au moins
/// 5 : le test n'est pas vide) ; les hauteurs des colonnes touchées, celles de toutes les autres (0) et le générateur sont
/// exactement les siens (V-CIN-07). Un `suivant() % n` à la place de `borne`, un modulo sans rejet, ou un `borne` sur un autre `n`
/// (par exemple `L·(L−1)`) font diverger l'état. Le changement d'énergie y est calculé sur la colonne et ses quatre liaisons
/// seulement (c'est la même différence de `H`, `indice_egal_difference_de_hamiltonien` le prouve en petit).
///
/// Ne couvre pas : les faces moins grandes (les rejets y sont trop rares pour se voir).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn grande_face_rejets_de_borne() {
    use std::collections::BTreeMap;
    const L: usize = 3000;
    const N: u32 = (L * L) as u32;
    let (j, mu) = (16_i64, 24_i64);
    let mut face = Face::new(L, 16, 24, 31);
    let mut alea = Alea::new(31);
    let mut h: BTreeMap<usize, i64> = BTreeMap::new();
    let lire = |h: &BTreeMap<usize, i64>, x: usize, y: usize| h.get(&(y * L + x)).copied().unwrap_or(0);
    let seuil = N.wrapping_neg() % N;
    let mut rejets = 0;
    for essai in 0..40_000 {
        let i = loop {
            let r = alea.suivant();
            if r >= seuil {
                break (r % N) as usize;
            }
            rejets += 1;
        };
        let (x, y) = (i % L, i / L);
        let ajout = alea.suivant() >> 31 == 1;
        let (c, pas) = (lire(&h, x, y), if ajout { 1 } else { -1 });
        let voisins = [
            lire(&h, (x + 1) % L, y),
            lire(&h, x, (y + 1) % L),
            lire(&h, (x + L - 1) % L, y),
            lire(&h, x, (y + L - 1) % L),
        ];
        let energie = |hauteur: i64| j * voisins.iter().map(|v| (hauteur - v).abs()).sum::<i64>() - mu * hauteur;
        let k = energie(c + pas) - energie(c);
        let accepte = if k <= 0 {
            true
        } else if k >= 366 {
            false
        } else {
            u64::from(alea.suivant()) < EXP_NEG[k as usize]
        };
        if accepte {
            h.insert(i, c + pas);
        }
        assert_eq!(face.essai(), accepte, "essai {essai}");
    }
    assert!(rejets >= 5, "seulement {rejets} rejets de borne : le test est vide");
    assert!(h.values().any(|&v| v != 0));
    for (i, &v) in face.hauteurs().iter().enumerate() {
        assert_eq!(i64::from(v), h.get(&i).copied().unwrap_or(0), "colonne {i}");
    }
    let octets = canonique(&face);
    assert_eq!(octets[octets.len() - 16..], alea.octets(), "générateur");
}

/// juste : la doc dit « panique » : `Face::new` et `Face::depuis_hauteurs` paniquent si la taille n'est pas de 3 à 4096 (2 et 4097 :
/// pas de face ; 0 aussi), `depuis_hauteurs` si le nombre de hauteurs n'est pas `taille²`, et `hauteur` et `indice` si la colonne
/// est hors de la face (`x = taille`, `y = taille`) ; 3 et 4096 sont permis (V-CIN-08).
///
/// Une hauteur de l'`i32` qui déborderait **panique** (la doc : « jamais de rebouclage silencieux ») : une face de 3 × 3 toute à
/// `i32::MAX` avec `J = 0`, `Δμ = 1000` (tout ajout accepté) panique dans les 200 premiers essais (un ajout a une chance sur deux à
/// chaque essai) ; une face toute à `i32::MIN` avec `Δμ = −1000` (tout retrait accepté) de même.
///
/// Ne couvre pas : le message de la panique.
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn bornes_et_paniques() {
    for (depart, mu) in [(i32::MAX, 1000), (i32::MIN, -1000)] {
        let mut face = Face::depuis_hauteurs(3, 0, mu, 1, &[depart; 9]);
        let resultat = catch_unwind(move || {
            for _ in 0..200 {
                face.essai();
            }
        });
        assert!(resultat.is_err(), "un débordement de l'i32 doit paniquer (départ {depart})");
    }
    for taille in [0_usize, 1, 2, 4097, 100_000] {
        assert!(catch_unwind(|| Face::new(taille, 16, 0, 1)).is_err(), "Face::new({taille}) devait paniquer");
    }
    assert!(catch_unwind(|| Face::depuis_hauteurs(2, 16, 0, 1, &[0; 4])).is_err());
    assert!(catch_unwind(|| Face::depuis_hauteurs(3, 16, 0, 1, &[0; 8])).is_err());
    assert!(catch_unwind(|| Face::depuis_hauteurs(3, 16, 0, 1, &[0; 10])).is_err());
    let face = Face::new(3, 16, 0, 1);
    assert_eq!(face.taille(), 3);
    assert!(catch_unwind(|| face.hauteur(3, 0)).is_err());
    assert!(catch_unwind(|| face.hauteur(0, 3)).is_err());
    assert!(catch_unwind(|| face.indice(3, 0, true)).is_err());
    assert!(catch_unwind(|| face.indice(0, 3, false)).is_err());
    assert_eq!(face.hauteur(2, 2), 0);
    let grande = Face::new(4096, 16, 0, 1);
    assert_eq!(grande.taille(), 4096);
    assert_eq!(grande.hauteur(4095, 4095), 0);
}

/// juste : lecture seule : `Face::hauteur(x, y)` lit la colonne `(x, y)` rangée au numéro `y * taille + x`, et `hauteurs()` rend
/// toutes les hauteurs dans cet ordre (V-CIN-09). Sur la face de 3 × 3 `0 1 -2 / 3 0 0 / -1 5 2` : `hauteur(1, 0) = 1`, `hauteur(2,
/// 0) = -2`, `hauteur(0, 1) = 3`, `hauteur(1, 2) = 5`, `hauteur(2, 2) = 2`.
///
/// Ne couvre pas : l'écriture canonique et les sommes (`etape_4_mesures`).
#[test]
#[cfg_attr(not(feature = "etape-4"), ignore)]
fn hauteur_lit_la_colonne_y_fois_taille_plus_x() {
    let depart = [0, 1, -2, 3, 0, 0, -1, 5, 2];
    let face = Face::depuis_hauteurs(3, 16, 0, 1, &depart);
    assert_eq!(face.hauteurs(), depart);
    assert_eq!((face.hauteur(1, 0), face.hauteur(2, 0), face.hauteur(0, 1)), (1, -2, 3));
    assert_eq!((face.hauteur(0, 2), face.hauteur(1, 2), face.hauteur(2, 2)), (-1, 5, 2));
    for y in 0..3 {
        for x in 0..3 {
            assert_eq!(face.hauteur(x, y), depart[y * 3 + x]);
        }
    }
}
