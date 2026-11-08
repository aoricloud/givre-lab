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
//! # Ce que fait l'étape 1 : la directive `dla`
//!
//! **Cette section est le contrat de [`lancer`] à l'étape 1** (reprise de la documentation de [`crate::dla`], § « La directive
//! `dla` et la sortie de `lancer` »). Une directive `dla <particules>` (lue par [`crate::dla::particules`]) sur une
//! `boite largeur hauteur` lance une DLA : `lancer` fait croître un [`crate::dla::Amas`] de graine `graine` et de
//! `demi_cote = demi_cote_de_la_boite(largeur, hauteur)`, par des appels de [`crate::dla::Amas::essayer_ajouter`] (jamais
//! [`crate::dla::Amas::ajouter`], qui paniquerait), jusqu'à `particules` sites (germe compris), et rend, **à la place** des
//! sorties du scénario vide de l'étape 0 :
//!
//! - **l'empreinte** : [`crate::dla::Amas::empreinte`] de l'amas final (deux boîtes de même `demi_cote` ont donc la même
//!   empreinte) ;
//! - **les mesures** : le CSV `particules,rayon,rayon_giration`, la ligne d'en-tête puis une ligne par nombre `k` de sites,
//!   dans cet ordre croissant : les puissances de 2 `2, 4, 8, …` qui sont `<= particules`, puis `particules` lui-même s'il
//!   n'est pas déjà dans la liste (jamais deux fois la même ligne ; `dla 1` ne donne que la ligne `1`). La ligne `k` est
//!   `k,<rayon>,<giration>` : `<rayon>` est [`crate::dla::Amas::rayon`] de l'amas quand il avait `k` sites (en décimal),
//!   `<giration>` est [`crate::mesures::rayon_de_giration`] de ses `k` premiers sites, écrit avec **six décimales**
//!   (`format!("{:.6}", g)` : `0.000000` pour le germe seul). Chaque ligne, la dernière comprise, finit par un seul `\n`
//!   (pas de `\r`) ;
//! - **l'aperçu** : le PNG ([`crate::image::ecrire_png`]) de [`crate::dla::amas_colore`]`(amas.sites(), COTE_APERCU)`.
//!
//! **Les refus**, tous `ErreurLancer::DirectiveInconnue(ligne)` (l'énumération de l'étape 0 n'a pas d'autre variante ; ni elle
//! ni `src/main.rs` ne changent), où `ligne` est la **plus petite** des lignes refusées : une directive dont le nom n'est pas
//! `dla` (sa ligne) ; une directive `dla` que [`crate::dla::particules`] refuse (la ligne que `particules` rend) ; une
//! directive `dla` sur un `hexagone` (la ligne de la première `dla`). Puis, si rien n'est refusé, un amas qui sort du réseau
//! avant d'avoir `particules` sites (`essayer_ajouter` rend `None`) : la ligne de la directive `dla`. Rien n'est rendu d'autre
//! dans ces cas. Un scénario sans directive `dla` (et sans autre directive) se comporte exactement comme à l'étape 0.
//!
//! L'aperçu est [`crate::dla::amas_colore`], avec la palette [`crate::dla::PALETTE`], le fond [`crate::dla::FOND`] et
//! [`crate::dla::couleur_du_rang`].
//!
//! # Ce que fait l'étape 3 : la directive `reiter`
//!
//! **Cette section est le contrat de [`lancer`] à l'étape 3** (reprise de la documentation de [`crate::flocon`], § « La
//! directive `reiter` et ses sorties »). Le scénario est `hexagone <rayon>` et **une seule** directive,
//! `reiter <alpha> <beta> <gamma> <pas>`. Quand la **première** directive s'appelle `reiter`, [`lancer`] appelle
//! [`crate::flocon::lancer_reiter`] et rend sa [`Sortie`] ; son `Err(ligne)` devient
//! [`ErreurLancer::DirectiveInconnue`]`(ligne)`. La graine n'est pas utilisée (le modèle n'a aucun hasard : deux graines
//! donnent la même sortie, octet pour octet). Toute autre première directive se comporte comme avant (les étapes 0, 1 et
//! 2s ci-dessus).
//!
//! Les sorties de `reiter` sont : **l'empreinte** de [`crate::flocon::Flocon::empreinte`] de l'état final ; **les
//! mesures**, le CSV `temps,gelees,rayon,compacite`, avec une ligne à l'instant 0, puis à chaque instant `t` multiple de
//! `intervalle = max(1, pas / 20)` (division entière) tel que `1 <= t <= pas`, puis une dernière ligne à `t = pas` si ce
//! n'est pas déjà un multiple ; `gelees`, `rayon` et `compacite` sont [`crate::flocon::Flocon::gelees`],
//! [`crate::flocon::Flocon::rayon_cristal`] et la compacité écrite avec **six décimales exactes** (calculée en entiers,
//! voir la documentation de [`crate::flocon`]) ; chaque ligne finit par un seul `\n` ; **l'aperçu**, le PNG
//! ([`crate::image::ecrire_png`]) de [`crate::flocon::Flocon::apercu`], un carré de côté `2 * rayon + 1`.
//!
//! **Les refus**, tous `ErreurLancer::DirectiveInconnue(ligne)` : la ligne de la **première directive fautive dans l'ordre
//! du fichier**. C'est la ligne de la directive `reiter` elle-même si la forme n'est pas un `hexagone`, si elle n'a pas
//! exactement quatre arguments, si `alpha`, `beta` ou `gamma` ne se lisent pas par
//! [`crate::flocon::Parametres::lire`] ou si `pas` n'est pas un entier décimal de 1 à [`crate::flocon::PAS_MAX`] ; sinon
//! c'est la ligne de la directive suivante, quelle qu'elle soit (il n'y a qu'une directive en plus de la forme). Rien
//! n'est calculé alors. Les refus des étapes 0, 1 et 2s restent ceux d'avant (par exemple une `dla` sur un `hexagone`),
//! et une première directive qui n'est ni `reiter`, ni `dla`, ni `surface` est refusée à sa ligne.
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
use crate::dla::{Amas, COTE_APERCU, amas_colore, demi_cote_de_la_boite, particules};
use crate::mesures::rayon_de_giration;
use crate::scenario::{Directive, ErreurLecture, Forme, Scenario};
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
/// La première directive choisit le lancer : `reiter` appelle [`crate::flocon::lancer_reiter`] (étape 3) ; sinon, si une
/// directive `dla` est présente, c'est le lancer DLA de l'étape 1 ; sinon, avec des directives, la première doit être une
/// `surface` bien formée et il ne doit y en avoir aucune autre (étape 2s) ; sans directive, c'est le scénario vide de
/// l'étape 0.
///
/// # Erreurs
///
/// `ErreurLancer::DirectiveInconnue(ligne)` si les directives ne forment pas exactement ce que l'étape attend. Pour
/// `reiter`, `ligne` est celle de la première directive fautive dans l'ordre du fichier (voir « Ce que fait l'étape 3 :
/// la directive `reiter` ») ; pour `dla`, la plus petite des lignes refusées ; pour `surface`, la première directive
/// fautive. C'est la seule erreur de `lancer`, et rien d'autre n'est calculé alors.
pub fn lancer(scenario: &Scenario, graine: u64) -> Result<Sortie, ErreurLancer> {
    if scenario.directives.first().is_some_and(|directive| directive.nom == "reiter") {
        return crate::flocon::lancer_reiter(scenario).map_err(ErreurLancer::DirectiveInconnue);
    }
    if scenario.directives.iter().any(|directive| directive.nom == "dla") {
        return lancer_dla(scenario, graine);
    }
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

/// Le lancer de la directive `dla` (étape 1) : les refus de la documentation du module, puis l'amas, l'empreinte, les
/// mesures CSV et l'aperçu.
fn lancer_dla(scenario: &Scenario, graine: u64) -> Result<Sortie, ErreurLancer> {
    // La plus petite ligne refusée : une directive dont le nom n'est pas `dla`, une `dla` que `particules` refuse (une
    // seconde `dla` comprise, à sa propre ligne), une `dla` sur un `hexagone`.
    let mut refus: Option<usize> = None;
    let mut refuser = |ligne: usize| {
        refus = Some(refus.map_or(ligne, |autre| autre.min(ligne)));
    };
    let mut ligne_dla: Option<usize> = None;
    for directive in &scenario.directives {
        if directive.nom == "dla" {
            if ligne_dla.is_none() {
                ligne_dla = Some(directive.ligne);
            }
        } else {
            refuser(directive.ligne);
        }
    }
    if let Err(ErreurLecture::LigneInvalide(ligne)) = particules(scenario) {
        refuser(ligne);
    }
    if matches!(scenario.forme, Forme::Hexagone { .. })
        && let Some(ligne) = ligne_dla
    {
        refuser(ligne);
    }
    if let Some(ligne) = refus {
        return Err(ErreurLancer::DirectiveInconnue(ligne));
    }

    // Ici, au moins une `dla`, toutes les directives sont des `dla`, une seule (sinon `particules` refuserait), elle est
    // bien formée, et la forme est une boîte.
    let Some(ligne_dla) = ligne_dla else {
        return Err(ErreurLancer::DirectiveInconnue(scenario.directives.first().map_or(1, |d| d.ligne)));
    };
    let Ok(Some(nombre)) = particules(scenario) else {
        return Err(ErreurLancer::DirectiveInconnue(ligne_dla));
    };
    let Forme::Boite { largeur, hauteur } = scenario.forme else {
        return Err(ErreurLancer::DirectiveInconnue(ligne_dla));
    };

    // Les instants des mesures : les puissances de 2 au plus égales à `nombre`, puis `nombre` s'il n'y figure pas.
    let mut instants = Vec::new();
    let mut instant = 2_usize;
    while instant <= nombre {
        instants.push(instant);
        instant *= 2;
    }
    if instants.last() != Some(&nombre) {
        instants.push(nombre);
    }

    // L'amas, site par site, en relevant les mesures aux instants voulus ; un débordement du réseau est le refus à la ligne
    // de la directive `dla`.
    let mut amas = Amas::new(graine, demi_cote_de_la_boite(largeur, hauteur));
    let mut mesures = String::from("particules,rayon,rayon_giration\n");
    let mut prochain = 0;
    for k in 1..=nombre {
        while amas.sites().len() < k {
            if amas.essayer_ajouter().is_none() {
                return Err(ErreurLancer::DirectiveInconnue(ligne_dla));
            }
        }
        if prochain < instants.len() && instants[prochain] == k {
            let rayon = amas.rayon();
            let giration = rayon_de_giration(&amas.sites()[..k]);
            mesures.push_str(&format!("{k},{rayon},{giration:.6}\n"));
            prochain += 1;
        }
    }

    let empreinte = amas.empreinte();
    let (largeur_image, hauteur_image, rvb) = amas_colore(amas.sites(), COTE_APERCU);
    let apercu = image::ecrire_png(largeur_image, hauteur_image, &rvb);
    Ok(Sortie { empreinte, mesures, apercu })
}
