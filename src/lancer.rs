//! Lancer un scénario : ses sorties (empreinte, mesures, aperçu) (étape 0, puis chaque étape). Le binaire `givre`
//! n'est qu'une coque autour de [`lancer`] : les tests appellent `lancer` sans lancer de processus.
//!
//! # Ce que fait l'étape 0
//!
//! Le scénario vide : une forme (`boite` ou `hexagone`) et rien d'autre. Il n'y a encore rien à simuler : l'état est la
//! forme et la graine. L'étape 0 ne comprend **aucune directive** en plus de la forme (voir
//! [`crate::scenario::Scenario::directives`]) : la première directive restante fait refuser le scénario
//! ([`ErreurLancer::DirectiveInconnue`]). Les étapes suivantes feront consommer leurs directives par `lancer`.
//!
//! # L'écriture canonique de l'état du scénario vide
//!
//! L'empreinte est le SHA-256 (`empreinte::sha256_hex`) de cette suite d'octets, dans cet ordre :
//!
//! ```text
//! 8 octets   l'étiquette ASCII "GIVRE000"  (47 49 56 52 45 30 30 30)
//! 1 octet    la forme : 0 pour une boîte, 1 pour un hexagone
//! boîte      la largeur puis la hauteur, chacune en u32 petit-boutiste (4 octets chacune)
//! hexagone   le rayon en u32 petit-boutiste (4 octets)
//! 16 octets  Alea::new(graine).octets()  (le générateur tout juste amorcé, sans aucun tirage)
//! ```
//!
//! Soit 33 octets pour une boîte et 29 pour un hexagone. Deux graines différentes, deux formes différentes ou deux
//! boîtes de largeur et hauteur échangées donnent des empreintes différentes.
//!
//! # Les mesures
//!
//! Le texte CSV `temps,cases\n0,<cases>\n` : l'en-tête, puis une ligne. `<cases>` est, en décimal sans zéro de tête, le
//! nombre de cases : `largeur * hauteur` pour une boîte, `3 * rayon * (rayon + 1) + 1` pour un hexagone. Chaque ligne,
//! la dernière comprise, se termine par un seul `\n` (pas de `\r`).
//!
//! # L'aperçu
//!
//! Le PNG (`image::ecrire_png`) de `rendu::apercu_graine(l, h, graine)` : pour une boîte `l = largeur` et `h = hauteur` ;
//! pour un hexagone de rayon `R`, `l = h = 2R + 1`.
//!
//! # Le binaire `givre`
//!
//! `givre lancer <scénario> --graine <n> --sortie <dossier>` (voir `src/main.rs`) lit le fichier de scénario, appelle
//! [`lancer`] et écrit `empreinte.txt` (l'empreinte puis `\n`), `mesures.csv` et `apercu.png` dans le dossier.

use crate::scenario::Scenario;

/// Ce que produit un lancer.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Sortie {
    /// L'empreinte de l'état : 64 chiffres hexadécimaux minuscules (voir la documentation du module).
    pub empreinte: String,
    /// Les mesures, en CSV (voir la documentation du module).
    pub mesures: String,
    /// L'aperçu, un fichier PNG complet (voir la documentation du module).
    pub apercu: Vec<u8>,
}

/// Pourquoi un scénario n'a pas pu être lancé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurLancer {
    /// Le scénario contient une directive que ce lancer ne comprend pas ; le numéro de sa ligne (la première des
    /// directives restantes, dans l'ordre du fichier).
    DirectiveInconnue(usize),
}

/// Lance le scénario avec cette graine (voir la documentation du module). Pour le même scénario et la même graine, le
/// résultat est toujours le même, octet pour octet, sur toutes les machines.
///
/// # Erreurs
///
/// `ErreurLancer::DirectiveInconnue(ligne)` si `scenario.directives` n'est pas vide : `ligne` est le numéro de ligne
/// de sa première directive. Rien d'autre n'est calculé alors.
#[allow(unused_variables)]
pub fn lancer(scenario: &Scenario, graine: u64) -> Result<Sortie, ErreurLancer> {
    todo!()
}
