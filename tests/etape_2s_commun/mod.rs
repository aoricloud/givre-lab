//! Aides communes aux tests de l'étape 2s (la croissance de surface) : les simulateurs naïfs des quatre modèles, écrits ici
//! en boucles simples, sans rien partager avec `givre_lab::surface` (ni ses types, ni sa structure : l'occupation d'Eden est
//! un `Vec<Vec<bool>>`, l'appartenance au périmètre une recherche linéaire dans le `Vec`), et quelques aides d'écriture.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use givre_lab::alea::Alea;

/// Les quatre modèles, sous leur nom d'écriture (ceux de la directive `surface`).
pub const NOMS: [&str; 4] = ["depot", "relaxation", "balistique", "eden"];

/// La capacité d'Eden de la documentation (`HAUTEUR_EDEN`), recopiée ici : un test ne doit pas se contenter de lire la
/// constante du code qu'il juge.
pub const HAUTEUR_EDEN: usize = 4096;

/// Ce que chaque branche de la règle a fait, pour vérifier que les tests exercent toutes les branches.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Branches {
    /// Relaxation : la particule reste en `i` parce que `hi < hg` et `hi < hd` (strictement plus bas que les deux).
    pub reste_strict: u64,
    /// Relaxation : la particule reste en `i` avec une égalité (`hi == hg` ou `hi == hd`, et `hi <= les deux`).
    pub reste_egalite: u64,
    /// Relaxation : la particule va à gauche sans tirage, ou à droite sans tirage.
    pub gauche_sans_tirage: u64,
    pub droite_sans_tirage: u64,
    /// Relaxation : `hg == hd < hi`, pièce tirée ; pile (0) donne la gauche, face (1) la droite.
    pub pile_gauche: u64,
    pub pile_droite: u64,
    /// Balistique : le maximum vient de `h[i] + 1`, de la gauche seule, de la droite seule (strictement au-dessus des deux autres).
    pub max_vertical: u64,
    pub max_gauche: u64,
    pub max_droite: u64,
    /// Eden : le site sorti du périmètre était le dernier (`k == len - 1`) ou un autre.
    pub eden_dernier: u64,
    pub eden_autre: u64,
}

/// Le simulateur naïf : la règle de la documentation de `surface`, mot pour mot, et la même consommation du hasard.
pub struct Naif {
    pub modele: &'static str,
    pub l: usize,
    pub alea: Alea,
    pub h: Vec<u64>,
    pub particules: u64,
    /// Eden : `occ[y][x]` ; les rangées sont ajoutées quand il faut.
    pub occ: Vec<Vec<bool>>,
    /// Eden : le périmètre, un `Vec` ordonné ; « dans le périmètre » se teste par `contains`.
    pub perimetre: Vec<(usize, usize)>,
    pub branches: Branches,
}

impl Naif {
    pub fn new(modele: &'static str, l: usize, graine: u64) -> Naif {
        assert!(NOMS.contains(&modele));
        let perimetre = if modele == "eden" { (0..l).map(|x| (x, 0)).collect() } else { Vec::new() };
        Naif {
            modele,
            l,
            alea: Alea::new(graine),
            h: vec![0; l],
            particules: 0,
            occ: Vec::new(),
            perimetre,
            branches: Branches::default(),
        }
    }

    /// Une surface de départ écrite à la main (pas pour Eden).
    pub fn depuis(modele: &'static str, hauteurs: &[u64], graine: u64) -> Naif {
        let mut naif = Naif::new(modele, hauteurs.len(), graine);
        naif.h = hauteurs.to_vec();
        naif.particules = hauteurs.iter().sum();
        naif
    }

    /// Un pas : le tirage, puis la règle.
    pub fn pas(&mut self) {
        let choix = if self.modele == "eden" {
            let n = self.perimetre.len();
            self.alea.borne(n as u32) as usize
        } else {
            self.alea.borne(self.l as u32) as usize
        };
        self.appliquer(choix);
    }

    /// La règle, après le tirage du choix (la colonne, ou l'indice dans le périmètre).
    pub fn appliquer(&mut self, choix: usize) {
        let l = self.l;
        match self.modele {
            "depot" => self.h[choix] += 1,
            "relaxation" => {
                let i = choix;
                let (g, d) = ((i + l - 1) % l, (i + 1) % l);
                let (hg, hi, hd) = (self.h[g], self.h[i], self.h[d]);
                let cible;
                if hi <= hg && hi <= hd {
                    cible = i;
                    if hi < hg && hi < hd {
                        self.branches.reste_strict += 1;
                    } else {
                        self.branches.reste_egalite += 1;
                    }
                } else if hg < hd {
                    cible = g;
                    self.branches.gauche_sans_tirage += 1;
                } else if hd < hg {
                    cible = d;
                    self.branches.droite_sans_tirage += 1;
                } else {
                    let pile = self.alea.suivant() >> 31;
                    if pile == 0 {
                        cible = g;
                        self.branches.pile_gauche += 1;
                    } else {
                        cible = d;
                        self.branches.pile_droite += 1;
                    }
                }
                self.h[cible] += 1;
            }
            "balistique" => {
                let i = choix;
                let (g, d) = ((i + l - 1) % l, (i + 1) % l);
                let (hg, hi, hd) = (self.h[g], self.h[i], self.h[d]);
                let mut nouveau = hi + 1;
                if hi + 1 >= hg && hi + 1 >= hd {
                    self.branches.max_vertical += 1;
                } else if hg > hd {
                    self.branches.max_gauche += 1;
                } else {
                    self.branches.max_droite += 1;
                }
                if hg > nouveau {
                    nouveau = hg;
                }
                if hd > nouveau {
                    nouveau = hd;
                }
                self.h[i] = nouveau;
            }
            _ => self.appliquer_eden(choix),
        }
        self.particules += 1;
    }

    fn est_occupe(&self, x: usize, y: usize) -> bool {
        y < self.occ.len() && self.occ[y][x]
    }

    fn appliquer_eden(&mut self, k: usize) {
        let l = self.l;
        // swap_remove(k) à la main : le dernier prend la place de k
        let dernier = self.perimetre.len() - 1;
        if k == dernier {
            self.branches.eden_dernier += 1;
        } else {
            self.branches.eden_autre += 1;
        }
        let site = self.perimetre[k];
        self.perimetre[k] = self.perimetre[dernier];
        self.perimetre.pop();
        let (x, y) = site;
        while self.occ.len() <= y {
            self.occ.push(vec![false; l]);
        }
        self.occ[y][x] = true;
        // voisins dans l'ordre : (x+1, y), (x, y+1), (x-1, y), (x, y-1)
        let voisins: [(i64, i64); 4] =
            [(x as i64 + 1, y as i64), (x as i64, y as i64 + 1), (x as i64 - 1, y as i64), (x as i64, y as i64 - 1)];
        for (vx, vy) in voisins {
            if vy < 0 {
                continue; // le substrat
            }
            let vx = vx.rem_euclid(l as i64) as usize;
            let vy = vy as usize;
            assert!(vy < HAUTEUR_EDEN, "capacité d'Eden dépassée");
            if !self.est_occupe(vx, vy) && !self.perimetre.contains(&(vx, vy)) {
                self.perimetre.push((vx, vy));
            }
        }
        // la hauteur de la colonne x
        let mut haut = 0;
        for (r, rangee) in self.occ.iter().enumerate() {
            if rangee[x] {
                haut = r as u64 + 1;
            }
        }
        self.h[x] = haut;
    }

    /// Les hauteurs (pour Eden : le profil du dessus).
    pub fn hauteurs(&self) -> Vec<u64> {
        self.h.clone()
    }

    /// Eden : l'occupation du site `(x, y)`. Pour les autres modèles : `y < h[x]`.
    pub fn occupe(&self, x: usize, y: usize) -> bool {
        if self.modele == "eden" { self.est_occupe(x, y) } else { (y as u64) < self.h[x] }
    }

    /// Le nombre de sites occupés d'Eden.
    pub fn nombre_occupes(&self) -> u64 {
        self.occ.iter().map(|r| r.iter().filter(|&&o| o).count() as u64).sum()
    }

    /// L'écriture canonique de l'état, d'après la documentation du module `surface`, écrite ici octet par octet.
    pub fn octets(&self) -> Vec<u8> {
        let mut o: Vec<u8> = Vec::new();
        o.extend_from_slice(b"GIVRE2S0");
        o.push(NOMS.iter().position(|&n| n == self.modele).unwrap() as u8);
        o.extend_from_slice(&(self.l as u32).to_le_bytes());
        o.extend_from_slice(&self.particules.to_le_bytes());
        for h in &self.h {
            o.extend_from_slice(&h.to_le_bytes());
        }
        o.extend_from_slice(&self.alea.octets());
        if self.modele == "eden" {
            o.extend_from_slice(&(self.perimetre.len() as u32).to_le_bytes());
            for &(x, y) in &self.perimetre {
                o.extend_from_slice(&(x as u32).to_le_bytes());
                o.extend_from_slice(&(y as u32).to_le_bytes());
            }
            let r = *self.h.iter().max().unwrap();
            o.extend_from_slice(&(r as u32).to_le_bytes());
            for y in 0..r as usize {
                for x in 0..self.l {
                    o.push(u8::from(self.est_occupe(x, y)));
                }
            }
        }
        o
    }
}

/// La rugosité au carré, exacte, calculée à la main : `Σ (h - moyenne)²` en rationnels, ici avec des entiers :
/// `(L · Σh² - (Σh)², L²)` (même définition que la doc, recalculée par une boucle simple).
pub fn rugosite_naive(h: &[u64]) -> (u128, u128) {
    let l = h.len() as u128;
    let mut s1: u128 = 0;
    let mut s2: u128 = 0;
    for &x in h {
        s1 += u128::from(x);
        s2 += u128::from(x) * u128::from(x);
    }
    (l * s2 - s1 * s1, l * l)
}

/// Un hexadécimal minuscule.
pub fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// Le décalage, dans `Surface::octets()` d'un modèle à hauteurs, des 16 octets du générateur : 8 + 1 + 4 + 8 + 8·L.
pub fn debut_alea(l: usize) -> usize {
    8 + 1 + 4 + 8 + 8 * l
}
