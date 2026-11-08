//! Lancer un scénario : ses sorties (empreinte, mesures, aperçu) (étape 0, puis chaque étape). Le binaire `givre`
//! n'est qu'une coque autour de [`lancer`] : les tests appellent `lancer` sans lancer de processus.
//!
//! # Ce que fait l'étape 0
//!
//! Le scénario vide : une forme (`boite` ou `hexagone`) et rien d'autre. Il n'y a encore rien à simuler : l'état est la
//! forme et la graine. Sans directive (voir [`crate::scenario::Scenario::directives`]), `lancer` produit l'empreinte, les
//! mesures et l'aperçu décrits ci-dessous.
//!
//! # Ce que fait l'étape 2s : la directive `surface`
//!
//! Une fois la forme lue (elle est **ignorée** de `surface`), les directives doivent être exactement une
//! `surface <modele> <largeur> <couches>` : `modele` est l'un des noms de [`crate::surface::Modele::nom`], `largeur`
//! vaut de 1 à [`crate::surface::LARGEUR_MAX`], `couches` de 1 à [`crate::surface::COUCHES_MAX`]. La première directive
//! doit être cette `surface` bien formée, et il ne doit y en avoir aucune autre ; sinon `lancer` rend
//! [`ErreurLancer::DirectiveInconnue`] avec la ligne de la première directive fautive (la première si elle est mal formée
//! ou n'est pas `surface`, sinon la seconde). Le détail des sorties de `surface` est dans la documentation du module
//! [`crate::surface`], § « Le scénario : la directive `surface` et ses sorties » : `Surface::new` puis `couches` appels à
//! `deposer_couche`, l'empreinte de l'état final, le CSV `couches,rugosite` aux instants 1, 2, 4, … puis `couches`, et
//! l'aperçu de la surface finale vue de côté.
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

use crate::alea::Alea;
use crate::scenario::{Directive, Forme, Scenario};
use crate::surface::{COUCHES_MAX, LARGEUR_MAX, Modele, Surface};
use crate::{empreinte, image, rendu};

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
/// Sans directive, c'est le scénario vide de l'étape 0. Avec des directives, la première doit être une `surface` bien
/// formée et il ne doit y en avoir aucune autre.
///
/// # Erreurs
///
/// `ErreurLancer::DirectiveInconnue(ligne)` si les directives ne forment pas exactement une `surface` bien formée : `ligne`
/// est celle de la première directive fautive. C'est la seule erreur de `lancer`, et rien d'autre n'est calculé alors.
pub fn lancer(scenario: &Scenario, graine: u64) -> Result<Sortie, ErreurLancer> {
    if let Some(directive) = scenario.directives.first() {
        let Some((modele, largeur, couches)) = surface_bien_formee(directive) else {
            return Err(ErreurLancer::DirectiveInconnue(directive.ligne));
        };
        if let Some(autre) = scenario.directives.get(1) {
            return Err(ErreurLancer::DirectiveInconnue(autre.ligne));
        }
        return Ok(lancer_surface(modele, largeur, couches, graine));
    }

    // L'écriture canonique : l'étiquette, la forme, puis l'état du générateur tout juste amorcé.
    let mut octets = Vec::with_capacity(33);
    octets.extend_from_slice(b"GIVRE000");
    match scenario.forme {
        Forme::Boite { largeur, hauteur } => {
            octets.push(0);
            octets.extend_from_slice(&largeur.to_le_bytes());
            octets.extend_from_slice(&hauteur.to_le_bytes());
        }
        Forme::Hexagone { rayon } => {
            octets.push(1);
            octets.extend_from_slice(&rayon.to_le_bytes());
        }
    }
    octets.extend_from_slice(&Alea::new(graine).octets());
    let empreinte = empreinte::sha256_hex(&octets);

    // Les mesures et l'aperçu.
    let (cases, largeur, hauteur) = match scenario.forme {
        Forme::Boite { largeur, hauteur } => (u64::from(largeur) * u64::from(hauteur), largeur, hauteur),
        Forme::Hexagone { rayon } => {
            let r = u64::from(rayon);
            let cote = 2 * rayon + 1;
            (3 * r * (r + 1) + 1, cote, cote)
        }
    };
    let mesures = format!("temps,cases\n0,{cases}\n");
    let apercu = image::ecrire_png(largeur, hauteur, &rendu::apercu_graine(largeur, hauteur, graine));

    Ok(Sortie { empreinte, mesures, apercu })
}

/// La directive `surface` bien formée : le modèle, la largeur et le nombre de couches, ou `None` (mauvais nom, nombre
/// d'arguments, modèle, entier ou bornes).
fn surface_bien_formee(directive: &Directive) -> Option<(Modele, usize, u64)> {
    if directive.nom != "surface" || directive.args.len() != 3 {
        return None;
    }
    let modele = Modele::depuis_nom(&directive.args[0])?;
    let largeur = directive.entier(1).ok()?;
    let couches = directive.entier(2).ok()?;
    if !(1..=LARGEUR_MAX as u64).contains(&largeur) || !(1..=COUCHES_MAX).contains(&couches) {
        return None;
    }
    Some((modele, largeur as usize, couches))
}

/// Le lancer de la directive `surface` : `couches` couches complètes, puis l'empreinte, les mesures CSV et l'aperçu.
fn lancer_surface(modele: Modele, largeur: usize, couches: u64, graine: u64) -> Sortie {
    let mut surface = Surface::new(modele, largeur, graine);

    // Les instants des mesures : les puissances de 2 au plus égales à `couches`, puis `couches` s'il n'y figure pas.
    let mut instants = Vec::new();
    let mut instant = 1_u64;
    while instant <= couches {
        instants.push(instant);
        instant *= 2;
    }
    if instants.last() != Some(&couches) {
        instants.push(couches);
    }

    // Pour l'aperçu : `couche_de[x][r]` est la première couche complète après laquelle `h[x] > r` (la hauteur ne décroît
    // jamais, donc chaque rangée n'est franchie qu'une fois).
    let mut couche_de: Vec<Vec<u32>> = vec![Vec::new(); largeur];
    let mut precedent = vec![0_u64; largeur];

    let mut mesures = String::from("couches,rugosite\n");
    let mut prochain = 0;
    for t in 1..=couches {
        surface.deposer_couche();
        let hauteurs = surface.hauteurs();
        for x in 0..largeur {
            while precedent[x] < hauteurs[x] {
                couche_de[x].push(t as u32);
                precedent[x] += 1;
            }
        }
        if prochain < instants.len() && instants[prochain] == t {
            mesures.push_str(&format!("{t},{:.6}\n", surface.rugosite()));
            prochain += 1;
        }
    }

    // L'aperçu : la surface finale vue de côté, `largeur` × `max(1, h_max)`.
    let finale = surface.hauteurs();
    let hauteur_max = finale.iter().copied().max().expect("une surface a au moins une colonne");
    let hauteur_image = hauteur_max.max(1) as u32;
    let diviseur = couches.saturating_sub(1).max(1);
    let mut rvb = Vec::with_capacity(3 * largeur * hauteur_image as usize);
    for y in 0..hauteur_image {
        let r = u64::from(hauteur_image - 1 - y);
        for x in 0..largeur {
            if r >= finale[x] {
                rvb.extend_from_slice(&[240, 240, 240]);
            } else {
                let a = u64::from(couche_de[x][r as usize]);
                let v = (a - 1) * 255 / diviseur;
                rvb.extend_from_slice(&[v as u8, 96, (255 - v) as u8]);
            }
        }
    }
    let apercu = image::ecrire_png(largeur as u32, hauteur_image, &rvb);

    Sortie { empreinte: surface.empreinte(), mesures, apercu }
}
