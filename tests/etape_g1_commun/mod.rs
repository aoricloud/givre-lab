//! Aides communes aux tests de l'étape G1 : un simulateur NAÏF du modèle de Gravner et Griffeath sans bruit, écrit d'après la
//! doc de `src/neige.rs` seule (autre représentation : un dictionnaire ordonné `(q, r) -> [a, b, c, d]`, des entiers `i128`,
//! les arrondis écrits à la main, les voisins parcourus dans l'ordre inverse, un sous-pas = une nouvelle carte), l'énumération
//! des cases, les douze symétries de l'hexagone, des états de départ faits à la main ou tirés par un congruentiel linéaire.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

pub mod valeurs;

use givre_lab::flocon::lire_decimal;
use givre_lab::math::Fixe;
use givre_lab::neige::{Case, Neige, Parametres};
use std::collections::BTreeMap;

/// Les sept réglages écrits en décimaux, lus par `flocon::lire_decimal` (et non par `Parametres::lire`, qui est un stub de
/// cette étape) : dans l'ordre `rho, beta, alpha, theta, kappa, mu, gamma`.
pub fn parametres(rho: &str, beta: &str, alpha: &str, theta: &str, kappa: &str, mu: &str, gamma: &str) -> Parametres {
    let lire = |texte: &str| lire_decimal(texte).expect("décimal valide");
    Parametres {
        rho: lire(rho),
        beta: lire(beta),
        alpha: lire(alpha),
        theta: lire(theta),
        kappa: lire(kappa),
        mu: lire(mu),
        gamma: lire(gamma),
    }
}

/// Les réglages sous forme de sept `brut` (`i128`), dans le même ordre.
pub fn brut_des_reglages(p: &Parametres) -> [i128; 7] {
    [p.rho, p.beta, p.alpha, p.theta, p.kappa, p.mu, p.gamma].map(|x| i128::from(x.0))
}

/// Les cases de l'hexagone de rayon `rayon` dans l'ordre canonique de la doc de `reseau` (r croissant, puis q croissant),
/// réécrites ici : pour `r` de `-R` à `R`, `q` de `max(-R, -R - r)` à `min(R, R - r)`.
pub fn cases(rayon: i32) -> Vec<(i32, i32)> {
    let mut sortie = Vec::new();
    for r in -rayon..=rayon {
        for q in (-rayon).max(-rayon - r)..=rayon.min(rayon - r) {
            sortie.push((q, r));
        }
    }
    sortie
}

/// La distance hexagonale à l'origine : `max(|q|, |r|, |q + r|)`.
pub fn distance(q: i32, r: i32) -> i32 {
    q.abs().max(r.abs()).max((q + r).abs())
}

/// Les six directions de la doc de `reseau`, dans l'ordre `DIRECTIONS_HEX`, réécrites ici.
pub const DIRECTIONS: [(i32, i32); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];

/// Une symétrie de l'hexagone : une fonction de `(q, r)` vers `(q, r)`.
pub type Symetrie = fn(i32, i32) -> (i32, i32);

/// Les douze symétries de l'hexagone (le groupe diédral engendré par la rotation de 60° `(q, r) -> (-r, q + r)` et la
/// réflexion `(q, r) -> (r, q)`), écrites ici en toutes lettres : six rotations, puis ces six rotations après la réflexion.
pub fn symetries() -> Vec<(&'static str, Symetrie)> {
    vec![
        ("identité", |q, r| (q, r)),
        ("rotation 60°", |q, r| (-r, q + r)),
        ("rotation 120°", |q, r| (-q - r, q)),
        ("rotation 180°", |q, r| (-q, -r)),
        ("rotation 240°", |q, r| (r, -q - r)),
        ("rotation 300°", |q, r| (q + r, -q)),
        ("réflexion (q, r) -> (r, q)", |q, r| (r, q)),
        ("réflexion puis rotation 60°", |q, r| (-q, q + r)),
        ("réflexion puis rotation 120°", |q, r| (-q - r, r)),
        ("réflexion puis rotation 180°", |q, r| (-r, -q)),
        ("réflexion puis rotation 240°", |q, r| (q, -q - r)),
        ("réflexion puis rotation 300°", |q, r| (q + r, -r)),
    ]
}

/// Un état en `brut` : `[a, b, c, d]` (`a` vaut 0 ou 1).
pub type Brut = [i128; 4];

/// L'unité `Fixe::UN` en `i128`.
pub const UN: i128 = 1 << 32;

/// Le nombre de fois que chaque branche de la règle de rattachement a été prise par le naïf (pour vérifier qu'un jeu de
/// réglages exerce bien chaque branche, et non seulement qu'il passe).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Compteurs {
    /// `n = 1 ou 2` et `b2 >= beta` : rattachée.
    pub n12_oui: u64,
    /// `n = 1 ou 2` et `b2 < beta` : pas rattachée.
    pub n12_non: u64,
    /// `n = 3` et `b2 >= 1` : rattachée.
    pub n3_seuil_un: u64,
    /// `n = 3`, `b2 < 1`, `T < theta` et `b2 >= alpha` : rattachée (la lame de couteau).
    pub n3_couteau: u64,
    /// `n = 3`, `b2 < 1`, `T < theta` mais `b2 < alpha` : pas rattachée.
    pub n3_theta_ok_alpha_non: u64,
    /// `n = 3`, `b2 < 1`, `T >= theta` (et `b2 >= alpha`) : pas rattachée.
    pub n3_alpha_ok_theta_non: u64,
    /// `n = 3`, `b2 < 1`, ni l'un ni l'autre : pas rattachée.
    pub n3_ni_l_un_ni_l_autre: u64,
    /// `n >= 4` : rattachée sans condition.
    pub n4_plus: u64,
    /// Parmi les `n >= 4`, ceux dont `b2` est nul (donc rattachés par la seule règle `n >= 4`).
    pub n4_plus_sans_bord: u64,
    /// Les cases de bord dont la comparaison a été EXACTEMENT une égalité (`b2 = beta`, `b2 = 1`, `b2 = alpha`, `T = theta`).
    pub egalites: u64,
}

/// Le simulateur naïf : un état en `brut` dans un `BTreeMap` indexé par `(q, r)`, et le pas de la doc écrit sous-pas par
/// sous-pas avec des entiers `i128` et des arrondis à la main (sans `Fixe`).
#[derive(Clone)]
pub struct Naif {
    pub rayon: i32,
    /// `rho, beta, alpha, theta, kappa, mu, gamma` en `brut`.
    pub p: [i128; 7],
    pub temps: u64,
    pub etat: BTreeMap<(i32, i32), Brut>,
    pub compteurs: Compteurs,
}

/// `floor((2 s + 7) / 14)` : `s / 7` arrondi au plus proche (jamais de demi-cas), pour tout entier `s`.
pub fn diviser_par_sept(s: i128) -> i128 {
    (2 * s + 7).div_euclid(14)
}

/// `floor((p q + 2^31) / 2^32)` : le produit `Fixe`, arrondi au plus proche, demi-cas vers `+∞`.
pub fn produit(p: i128, q: i128) -> i128 {
    (p * q + (1 << 31)).div_euclid(1 << 32)
}

impl Naif {
    /// L'état de départ de la doc : `(1, 0, UN, 0)` en `(0, 0)` et `(0, 0, 0, rho)` ailleurs.
    pub fn initial(rayon: i32, p: &Parametres) -> Naif {
        let reglages = brut_des_reglages(p);
        let mut etat = BTreeMap::new();
        for c in cases(rayon) {
            etat.insert(c, [0, 0, 0, reglages[0]]);
        }
        etat.insert((0, 0), [1, 0, UN, 0]);
        Naif { rayon, p: reglages, temps: 0, etat, compteurs: Compteurs::default() }
    }

    /// Un état donné (une carte `(q, r) -> [a, b, c, d]` qui doit couvrir toutes les cases).
    pub fn depuis(rayon: i32, p: &Parametres, etat: &BTreeMap<(i32, i32), Brut>) -> Naif {
        assert_eq!(etat.len(), cases(rayon).len());
        Naif { rayon, p: brut_des_reglages(p), temps: 0, etat: etat.clone(), compteurs: Compteurs::default() }
    }

    fn dans(&self, (q, r): (i32, i32)) -> bool {
        distance(q, r) <= self.rayon
    }

    /// Les voisins présents, dans l'ordre INVERSE de `DIRECTIONS` (la doc dit que l'ordre ne change aucun bit).
    pub fn voisins(&self, (q, r): (i32, i32)) -> Vec<(i32, i32)> {
        DIRECTIONS.iter().rev().map(|&(dq, dr)| (q + dq, r + dr)).filter(|&y| self.dans(y)).collect()
    }

    /// `n(x)` : le nombre de voisins rattachés de `x`, pour l'état courant.
    pub fn n_rattaches(&self, x: (i32, i32)) -> usize {
        self.voisins(x).iter().filter(|y| self.etat[*y][0] == 1).count()
    }

    /// Le sous-pas 1 : la nouvelle vapeur de chaque case (une nouvelle carte).
    pub fn diffusion(&self) -> BTreeMap<(i32, i32), i128> {
        let mut d1 = BTreeMap::new();
        for (&x, v) in &self.etat {
            if v[0] == 1 {
                d1.insert(x, v[3]);
                continue;
            }
            let mut somme = v[3];
            for dir in DIRECTIONS.iter().rev() {
                let y = (x.0 + dir.0, x.1 + dir.1);
                somme += if self.dans(y) && self.etat[&y][0] == 0 { self.etat[&y][3] } else { v[3] };
            }
            d1.insert(x, diviser_par_sept(somme));
        }
        d1
    }

    /// Les sous-pas 1 et 2 : `(b2, c2, d2)` de chaque case.
    pub fn apres_le_gel(&self) -> BTreeMap<(i32, i32), [i128; 3]> {
        let d1 = self.diffusion();
        let mut sortie = BTreeMap::new();
        for (&x, v) in &self.etat {
            if v[0] == 0 && self.n_rattaches(x) >= 1 {
                let g = produit(self.p[4], d1[&x]);
                sortie.insert(x, [v[1] + (d1[&x] - g), v[2] + g, 0]);
            } else {
                sortie.insert(x, [v[1], v[2], d1[&x]]);
            }
        }
        sortie
    }

    /// `T(x)` : la vapeur du voisinage de sept cases de `x` après le sous-pas 2.
    pub fn voisinage(&self, apres: &BTreeMap<(i32, i32), [i128; 3]>, x: (i32, i32)) -> i128 {
        apres[&x][2] + self.voisins(x).iter().map(|y| apres[y][2]).sum::<i128>()
    }

    /// Un pas de la doc, sous-pas après sous-pas.
    pub fn pas(&mut self) {
        let [_, beta, alpha, theta, _, mu, gamma] = self.p;
        let apres = self.apres_le_gel();

        // 3. Les décisions d'abord (sur les `a` du début du pas), les changements ensuite.
        let mut nouveaux = Vec::new();
        let mut k = self.compteurs;
        for (&x, v) in &self.etat {
            let n = self.n_rattaches(x);
            if v[0] == 1 || n == 0 {
                continue;
            }
            let b2 = apres[&x][0];
            let oui = if n >= 4 {
                k.n4_plus += 1;
                if b2 == 0 {
                    k.n4_plus_sans_bord += 1;
                }
                true
            } else if n == 3 {
                let t = self.voisinage(&apres, x);
                if b2 == UN || b2 == alpha || t == theta {
                    k.egalites += 1;
                }
                if b2 >= UN {
                    k.n3_seuil_un += 1;
                    true
                } else {
                    match (t < theta, b2 >= alpha) {
                        (true, true) => {
                            k.n3_couteau += 1;
                            true
                        }
                        (true, false) => {
                            k.n3_theta_ok_alpha_non += 1;
                            false
                        }
                        (false, true) => {
                            k.n3_alpha_ok_theta_non += 1;
                            false
                        }
                        (false, false) => {
                            k.n3_ni_l_un_ni_l_autre += 1;
                            false
                        }
                    }
                }
            } else {
                if b2 == beta {
                    k.egalites += 1;
                }
                if b2 >= beta {
                    k.n12_oui += 1;
                    true
                } else {
                    k.n12_non += 1;
                    false
                }
            };
            if oui {
                nouveaux.push(x);
            }
        }
        self.compteurs = k;
        let mut a3: BTreeMap<(i32, i32), i128> = self.etat.iter().map(|(&x, v)| (x, v[0])).collect();
        let mut bc: BTreeMap<(i32, i32), [i128; 2]> = apres.iter().map(|(&x, v)| (x, [v[0], v[1]])).collect();
        for &x in &nouveaux {
            a3.insert(x, 1);
            let [b2, c2] = bc[&x];
            bc.insert(x, [0, c2 + b2]);
        }

        // 4. La fonte, aux cases non rattachées qui ont un voisin rattaché après le sous-pas 3.
        let mut suivant = BTreeMap::new();
        for &x in self.etat.keys() {
            let [b3, c3] = bc[&x];
            let mut d = apres[&x][2];
            let (mut b4, mut c4) = (b3, c3);
            let touche = self.voisins(x).iter().any(|y| a3[y] == 1);
            if a3[&x] == 0 && touche {
                let fb = produit(mu, b3);
                let fc = produit(gamma, c3);
                b4 = b3 - fb;
                c4 = c3 - fc;
                d += fb + fc;
            }
            suivant.insert(x, [a3[&x], b4, c4, d]);
        }
        self.etat = suivant;
        self.temps += 1;
    }

    pub fn avancer(&mut self, n: u64) {
        for _ in 0..n {
            self.pas();
        }
    }

    /// La masse totale `sum (b + c + d)`.
    pub fn masse_totale(&self) -> i128 {
        self.etat.values().map(|v| v[1] + v[2] + v[3]).sum()
    }

    /// Le nombre de cases rattachées.
    pub fn rattachees(&self) -> usize {
        self.etat.values().filter(|v| v[0] == 1).count()
    }

    /// La somme de `c` sur les cases rattachées.
    pub fn masse_du_cristal(&self) -> i128 {
        self.etat.values().filter(|v| v[0] == 1).map(|v| v[2]).sum()
    }

    /// La plus grande distance hexagonale d'une case rattachée.
    pub fn rayon_cristal(&self) -> i32 {
        self.etat.iter().filter(|(_, v)| v[0] == 1).map(|(&(q, r), _)| distance(q, r)).max().unwrap_or(0)
    }
}

/// L'état de la bibliothèque, case par case, dans l'ordre canonique ÉCRIT ICI (`cases(rayon)`), en `brut`. Vérifie en passant
/// que `Neige::cases()` est dans cet ordre.
pub fn etat_de(neige: &Neige) -> Vec<Brut> {
    let rayon = neige.rayon() as i32;
    let liste = neige.cases();
    let ordre = cases(rayon);
    assert_eq!(liste.len(), ordre.len(), "nombre de cases");
    ordre
        .iter()
        .enumerate()
        .map(|(i, &(q, r))| {
            let c = neige.case(q, r).expect("case de l'hexagone");
            assert_eq!(liste[i], c, "cases() n'est pas dans l'ordre canonique à l'indice {i}");
            [i128::from(c.a), i128::from(c.b.0), i128::from(c.c.0), i128::from(c.d.0)]
        })
        .collect()
}

/// L'état du naïf dans le même ordre.
pub fn etat_du_naif(naif: &Naif) -> Vec<Brut> {
    cases(naif.rayon).iter().map(|c| naif.etat[c]).collect()
}

/// Une `Case` de la bibliothèque depuis un état en `brut`.
pub fn case_de(v: Brut) -> Case {
    Case { a: v[0] == 1, b: Fixe(v[1] as i64), c: Fixe(v[2] as i64), d: Fixe(v[3] as i64) }
}

/// La neige de la bibliothèque et le naïf, depuis le même état en `brut` (une carte qui doit couvrir toutes les cases).
pub fn depuis_un_etat(rayon: u32, p: &Parametres, etat: &BTreeMap<(i32, i32), Brut>) -> (Neige, Naif) {
    let liste: Vec<Case> = cases(rayon as i32).iter().map(|c| case_de(etat[c])).collect();
    let neige = Neige::essayer_depuis_cases(rayon, *p, &liste).expect("état valide");
    (neige, Naif::depuis(rayon as i32, p, etat))
}

/// Un état vide : aucune case rattachée, aucune masse.
pub fn etat_vide(rayon: i32) -> BTreeMap<(i32, i32), Brut> {
    cases(rayon).into_iter().map(|c| (c, [0, 0, 0, 0])).collect()
}

/// Un congruentiel linéaire de 64 bits (constantes de Knuth, MMIX), pour des états de départ sans symétrie : des graines
/// fixes, écrites dans les tests.
pub struct Lcg(pub u64);

impl Lcg {
    pub fn suivant(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 11
    }

    /// Un entier de `0` à `borne` exclu.
    pub fn jusqu_a(&mut self, borne: u64) -> u64 {
        self.suivant() % borne
    }
}

/// Un état de départ sans symétrie, valide (I1, I2, I3) : un amas rattaché, connexe, grandi au hasard autour de l'origine ;
/// sur ses cases de bord, des `b`, `c`, `d` quelconques ; ailleurs de la vapeur. Si `multiples_de_sept`, toutes les vapeurs
/// sont des multiples de 7 `brut` (la division par 7 du sous-pas 1 est alors exacte au premier pas).
pub fn etat_au_hasard(rayon: i32, graine: u64, multiples_de_sept: bool) -> BTreeMap<(i32, i32), Brut> {
    let mut g = Lcg(graine);
    let dans = |(q, r): (i32, i32)| distance(q, r) <= rayon;
    let mut rattachees: BTreeMap<(i32, i32), i128> = BTreeMap::new();
    rattachees.insert((0, 0), 1);
    for _ in 0..(2 + g.jusqu_a(4)) {
        let candidates: Vec<(i32, i32)> = rattachees
            .keys()
            .flat_map(|&(q, r)| DIRECTIONS.iter().map(move |&(dq, dr)| (q + dq, r + dr)))
            .filter(|&y| dans(y) && !rattachees.contains_key(&y))
            .collect();
        for y in candidates {
            if g.jusqu_a(100) < 45 {
                rattachees.insert(y, 1);
            }
        }
    }
    let vapeur = |g: &mut Lcg, borne: u64| {
        let v = g.jusqu_a(borne) as i128;
        if multiples_de_sept { 7 * (v / 7) } else { v }
    };
    let mut etat = BTreeMap::new();
    for x in cases(rayon) {
        let touche = DIRECTIONS.iter().any(|&(dq, dr)| rattachees.contains_key(&(x.0 + dq, x.1 + dr)));
        let v: Brut = if rattachees.contains_key(&x) {
            [1, 0, (UN / 2) + vapeur(&mut g, (2 * UN) as u64), 0]
        } else if touche {
            let b = vapeur(&mut g, (5 * UN / 2) as u64);
            let c = vapeur(&mut g, (UN / 4) as u64);
            let d = vapeur(&mut g, (UN / 2) as u64);
            [0, b, c, d]
        } else {
            [0, 0, 0, vapeur(&mut g, UN as u64)]
        };
        etat.insert(x, v);
    }
    etat
}

/// L'image d'un état par une symétrie : la case `g(x)` du nouvel état porte le contenu de la case `x`.
pub fn image_de_l_etat(etat: &BTreeMap<(i32, i32), Brut>, g: Symetrie) -> BTreeMap<(i32, i32), Brut> {
    etat.iter().map(|(&(q, r), &v)| (g(q, r), v)).collect()
}

/// Un jeu de réglages en décimaux : `rho, beta, alpha, theta, kappa, mu, gamma`.
pub type Jeu = [&'static str; 7];

/// **Article** (annexe 8, ligne « 9 » : `β α θ κ μ γ` ; `ρ` lu sur la figure 9, qui porte ρ = .40, .41, .42, .44, .46, .50) : la
/// plaque à secteurs de la figure 9 (`ρ = .40`) et le dendrite (`ρ = .50`). Voir `references/etape-g1-valeurs.md` (V-G1-01).
pub const FIG9_RHO_040: Jeu = ["0.4", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"];
/// **Article** : la figure 9 à `ρ = .50` (la même ligne de l'annexe ; la ligne « 15tr » en est la copie avec `ρ = .5`).
pub const FIG9_RHO_050: Jeu = ["0.5", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"];
/// **Article** : la figure 13 au milieu (« stellar crystal with plate ends »), le seul cristal de la figure 13 dont le texte et
/// l'annexe donnent les mêmes réglages (V-G1-02). Il demande plus de 100 000 pas pour la figure : ici il ne sert qu'à des essais.
pub const FIG13_MILIEU: Jeu = ["0.36", "1.09", "0.01", "0.0745", "0.0001", "0.14", "0.00001"];
/// **Pas l'article** : un jeu d'essai proche de l'annexe (ligne « 13r ») mais avec le `γ` de l'annexe, que le texte contredit
/// (`.00006`) ; il sert d'entrée, sans rien prétendre de la figure. Il exerce l'instabilité en lame de couteau.
pub const JEU_B: Jeu = ["0.38", "1.06", "0.35", "0.112", "0.001", "0.14", "0.0006"];
/// **Pas l'article** : un jeu d'essai dense (`ρ` fort, `β` près de 1, seuils hauts) : la croissance est rapide, tous les rattachements
/// se produisent vite.
pub const JEU_C: Jeu = ["0.9", "1.05", "0.5", "0.5", "0.1", "0.05", "0.001"];
/// **Pas l'article** : un jeu d'essai (`α` et `θ` dans l'ordre de l'annexe pour la ligne « 13l », `γ = .00001` du texte et de « 15bm »).
pub const JEU_D: Jeu = ["0.65", "1.75", "0.2", "0.026", "0.15", "0.015", "0.00001"];
/// **Pas l'article** : un jeu d'essai à fort `θ` (la lame de couteau se déclenche souvent), `β = 1`.
pub const JEU_E: Jeu = ["1", "1", "0.3", "2", "0.02", "0.01", "0.0001"];

/// Les réglages d'un jeu.
pub fn p(jeu: Jeu) -> Parametres {
    parametres(jeu[0], jeu[1], jeu[2], jeu[3], jeu[4], jeu[5], jeu[6])
}
