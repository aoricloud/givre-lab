//! Ce que les fichiers de test de l'étape D1 partagent : le simulateur NAÏF du claquage diélectrique, et deux petits outils.
//!
//! Le simulateur naïf (`Naif`) répète mot pour mot la section « La dynamique, à la lettre » de la documentation de
//! `src/claquage.rs` : des boucles simples sur toute la boîte, un test d'appartenance à chaque site, rien d'optimisé, aucun appel
//! à `givre_lab::claquage` (il n'emploie que `givre_lab::alea::Alea`, déjà éprouvé à l'étape 0). Il est LENT et c'est voulu : la
//! bibliothèque doit donner EXACTEMENT ses sites, ses candidates, ses poids et ses potentiels, au bit près.

#![allow(dead_code)]

use givre_lab::alea::Alea;

/// Le seuil d'arrêt de la relaxation de la documentation : `2^-20`, écrit ici en dur (jamais lu dans la bibliothèque).
pub const SEUIL_DOC: f64 = 1.0 / 1_048_576.0;

/// Les quatre voisins, dans l'ordre de la documentation : droite, bas, gauche, haut.
pub const VOISINS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

/// Le facteur de sur-relaxation de la documentation : `2.0 - 6.0 / (d + 6.0)`, écrit ici en dur.
pub fn omega_doc(d: i32) -> f64 {
    2.0 - 6.0 / (f64::from(d) + 6.0)
}

/// L'état du simulateur naïf. Le potentiel est un tableau `(2d + 1)²` de `f64` pour les sites de la boîte `|x|, |y| <= d` ; tout
/// ce qui est hors du domaine `x² + y² <= d²` vaut 1.0 par définition (la documentation : « tout site hors de Ω »).
pub struct Naif {
    pub d: i32,
    pub eta: u32,
    pub alea: Alea,
    pub sites: Vec<(i32, i32)>,
    pub phi: Vec<Vec<f64>>,
    occupe: Vec<Vec<bool>>,
    pub touche: bool,
}

impl Naif {
    pub fn new(graine: u64, d: i32, eta: u32) -> Naif {
        let cote = (2 * d + 1) as usize;
        let mut naif = Naif {
            d,
            eta,
            alea: Alea::new(graine),
            sites: Vec::new(),
            phi: vec![vec![1.0; cote]; cote],
            occupe: vec![vec![false; cote]; cote],
            touche: false,
        };
        naif.ajouter_site(0, 0);
        naif
    }

    /// Dans le domaine : `x² + y² <= d²` (comparaison entière, bord compris).
    pub fn dans_omega(&self, x: i32, y: i32) -> bool {
        i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y) <= i64::from(self.d) * i64::from(self.d)
    }

    /// Dans l'amas (faux hors de la boîte `|x|, |y| <= d`, où l'amas ne peut pas être).
    pub fn dans_amas(&self, x: i32, y: i32) -> bool {
        x.abs() <= self.d && y.abs() <= self.d && self.occupe[(y + self.d) as usize][(x + self.d) as usize]
    }

    /// Le potentiel : 1.0 hors du domaine, sinon le tableau (0.0 sur l'amas).
    pub fn valeur(&self, x: i32, y: i32) -> f64 {
        if !self.dans_omega(x, y) { 1.0 } else { self.phi[(y + self.d) as usize][(x + self.d) as usize] }
    }

    fn poser(&mut self, x: i32, y: i32, v: f64) {
        self.phi[(y + self.d) as usize][(x + self.d) as usize] = v;
    }

    /// La relaxation de la documentation : sur-relaxation rouge-noir jusqu'à un écart de passe sous le seuil.
    pub fn relaxer(&mut self) {
        let omega = omega_doc(self.d);
        loop {
            let mut ecart = 0.0_f64;
            for couleur in 0..2 {
                for y in -self.d..=self.d {
                    for x in -self.d..=self.d {
                        if !self.dans_omega(x, y) || self.dans_amas(x, y) || (x + y).rem_euclid(2) != couleur {
                            continue;
                        }
                        let m = (((self.valeur(x + 1, y) + self.valeur(x, y + 1)) + self.valeur(x - 1, y))
                            + self.valeur(x, y - 1))
                            / 4.0;
                        let ancien = self.valeur(x, y);
                        let v = (ancien + omega * (m - ancien)).clamp(0.0, 1.0);
                        ecart = ecart.max((v - ancien).abs());
                        self.poser(x, y, v);
                    }
                }
            }
            if ecart < SEUIL_DOC {
                break;
            }
        }
    }

    pub fn ajouter_site(&mut self, x: i32, y: i32) {
        self.sites.push((x, y));
        self.occupe[(y + self.d) as usize][(x + self.d) as usize] = true;
        self.poser(x, y, 0.0);
        if VOISINS.iter().any(|&(dx, dy)| !self.dans_omega(x + dx, y + dy)) {
            self.touche = true;
        }
        if self.eta >= 1 {
            self.relaxer();
        }
    }

    /// Les candidates dans l'ordre canonique (`y` croissant puis `x` croissant) avec leur poids `φ^η` par multiplications
    /// répétées.
    pub fn candidates(&self) -> Vec<((i32, i32), f64)> {
        let mut liste = Vec::new();
        for y in -self.d..=self.d {
            for x in -self.d..=self.d {
                if !self.dans_omega(x, y) || self.dans_amas(x, y) {
                    continue;
                }
                if VOISINS.iter().any(|&(dx, dy)| self.dans_amas(x + dx, y + dy)) {
                    let mut poids = 1.0;
                    for _ in 0..self.eta {
                        poids *= self.valeur(x, y);
                    }
                    liste.push(((x, y), poids));
                }
            }
        }
        liste
    }

    /// Un pas : `None` (sans tirage) si l'amas touche la frontière, n'a pas de candidate ou si `W == 0.0` ; sinon un tirage
    /// `suivant_f64`, `u = t * W`, la première somme cumulée qui dépasse STRICTEMENT `u`.
    pub fn pas(&mut self) -> Option<(i32, i32)> {
        if self.touche {
            return None;
        }
        let candidates = self.candidates();
        let mut total = 0.0;
        for &(_, w) in &candidates {
            total += w;
        }
        if candidates.is_empty() || total == 0.0 {
            return None;
        }
        let u = self.alea.suivant_f64() * total;
        let mut cumul = 0.0;
        for &((x, y), w) in &candidates {
            cumul += w;
            if cumul > u {
                self.ajouter_site(x, y);
                return Some((x, y));
            }
        }
        panic!("u < W devrait toujours trouver une candidate");
    }

    /// Fait croître jusqu'à `n` sites ou jusqu'à l'arrêt ; rend vrai si `n` sites sont atteints.
    pub fn croitre(&mut self, n: usize) -> bool {
        while self.sites.len() < n {
            if self.pas().is_none() {
                return false;
            }
        }
        true
    }

    /// Les 16 octets de l'état du générateur (`Alea::octets`).
    pub fn octets_alea(&self) -> [u8; 16] {
        self.alea.octets()
    }

    /// Le plus petit entier `r >= 0` tel que tous les sites vérifient `x² + y² <= r²`.
    pub fn rayon(&self) -> u32 {
        let mut r = 0_i64;
        while !self.sites.iter().all(|&(x, y)| i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y) <= r * r) {
            r += 1;
        }
        r as u32
    }
}

/// Les 16 derniers octets d'une écriture canonique : l'état du générateur.
pub fn etat_du_generateur(ecriture: &[u8]) -> [u8; 16] {
    ecriture[ecriture.len() - 16..].try_into().expect("16 octets")
}

/// Le texte hexadécimal minuscule d'une suite d'octets.
pub fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// La moyenne et l'écart-type d'UNE valeur (diviseur `n - 1`).
pub fn moyenne_et_ecart_type(valeurs: &[f64]) -> (f64, f64) {
    let n = valeurs.len() as f64;
    let moyenne = valeurs.iter().sum::<f64>() / n;
    let variance = valeurs.iter().map(|v| (v - moyenne) * (v - moyenne)).sum::<f64>() / (n - 1.0);
    (moyenne, variance.sqrt())
}
