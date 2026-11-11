//! Étape G2 : le balayage (`balayage::balayage`) fait grandir un cristal par valeur d'un réglage, dans l'ordre de la liste, et rend ses mesures, son étiquette et son empreinte ; ses refus sont des codes
//! d'erreur, jamais des paniques ; les trois jeux de l'article, le CSV et les noms sont fixés par la doc. Valeurs attendues : `references/etape-g2-valeurs.md`, identifiants `V-G2-BAL-xx`, `V-G2-JEU-xx`, `V-G2-CSV-xx`.

mod etape_g2_commun;

use etape_g2_commun::*;
use givre_lab::balayage::{
    EN_TETE_CSV, ErreurBalayage, Forme, JeuDeBalayage, Mesures, PAS_MAX_BALAYAGE, RAYON_MAX_BALAYAGE, Reglage,
    Resultat, VALEURS_MAX, balayage, classer, jeu_beta, jeu_mu, jeu_rho, ligne_csv, mesurer,
};
use givre_lab::flocon::lire_decimal;
use givre_lab::math::Fixe;
use givre_lab::neige::{Neige, Parametres};

/// Les sept réglages d'un jeu, tous différents (aucun ne se confond avec un autre), pour voir lequel `fixer` change.
fn base_distincte() -> Parametres {
    parametres("0.45", "1.35", "0.07", "0.03", "0.004", "0.06", "0.00007")
}

/// Les mêmes réglages, un seul champ remplacé, écrit EN TOUTES LETTRES (pas par `fixer`).
fn avec(reglage: Reglage, p: Parametres, v: Fixe) -> Parametres {
    match reglage {
        Reglage::Rho => Parametres { rho: v, ..p },
        Reglage::Beta => Parametres { beta: v, ..p },
        Reglage::Alpha => Parametres { alpha: v, ..p },
        Reglage::Theta => Parametres { theta: v, ..p },
        Reglage::Kappa => Parametres { kappa: v, ..p },
        Reglage::Mu => Parametres { mu: v, ..p },
        Reglage::Gamma => Parametres { gamma: v, ..p },
    }
}

const TOUS: [Reglage; 7] =
    [Reglage::Rho, Reglage::Beta, Reglage::Alpha, Reglage::Theta, Reglage::Kappa, Reglage::Mu, Reglage::Gamma];

fn d(texte: &str) -> Fixe {
    lire_decimal(texte).expect("décimal valide")
}

/// juste : `Reglage::fixer` change UN SEUL des sept champs, celui du réglage, et laisse les six autres à l'identique (V-G2-BAL-01), pour chacun des sept réglages et plusieurs valeurs ; `Reglage::nom` rend
/// les noms que `Parametres::verifier` emploie (`"rho"`, `"beta"`, `"alpha"`, `"theta"`, `"kappa"`, `"mu"`, `"gamma"`). Un `fixer` qui lit le `β` pour le `ρ`, qui oublie de remplacer, ou qui en remplace deux échoue.
///
/// Ne couvre pas : la vérification des plages (voir `les_refus`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn fixer_ne_change_que_le_reglage_dit_et_les_noms() {
    let noms = ["rho", "beta", "alpha", "theta", "kappa", "mu", "gamma"];
    let base = base_distincte();
    for (reglage, nom) in TOUS.into_iter().zip(noms) {
        assert_eq!(reglage.nom(), nom);
        for valeur in [Fixe::ZERO, d("0.5"), d("0.99999"), Fixe::UN, d("3.3"), Fixe(-5)] {
            assert_eq!(reglage.fixer(base, valeur), avec(reglage, base, valeur), "{nom} = {valeur:?}");
        }
    }
    // Chaque champ de `base` est différent des six autres : `fixer` ne peut pas se tromper de champ sans être vu.
    let valeurs = [base.rho, base.beta, base.alpha, base.theta, base.kappa, base.mu, base.gamma];
    for i in 0..7 {
        for j in 0..i {
            assert_ne!(valeurs[i], valeurs[j]);
        }
    }
    // Un défaut d'un nom : celui que `Parametres::verifier` donne pour le premier réglage hors plage.
    for (reglage, nom) in TOUS.into_iter().zip(noms) {
        let hors_plage = avec(reglage, base, Fixe(-1));
        assert_eq!(hors_plage.verifier(), Err(givre_lab::neige::ErreurNeige::ParametreHorsPlage(nom)));
    }
}

/// juste : un balayage rend, pour chaque valeur, EXACTEMENT ce que donnent une `Neige` faite à la main (`essayer_new`, `avancer`) et les mesures appelées une à une (V-G2-BAL-02) : pour chacun des sept réglages, trois valeurs
/// permises, un domaine de rayon 12 et 90 pas, les `valeur`, `mesures` (celles du calcul naïf du test sur les cases rattachées), `forme` (`classer` de ces mesures) et `empreinte` (`Neige::empreinte` de la neige à la main)
/// sont ceux du résultat, dans l'ordre de la liste. Un balayage qui garde le réglage d'avant, qui échange deux valeurs, qui lit le `ρ` pour le `β`, qui rend l'empreinte d'un autre cristal ou qui ne fait pas avancer
/// de `pas` pas échoue. Les cristaux ont grandi (au moins trois ont plus de 20 cases), et chaque réglage donne des empreintes différentes pour des valeurs différentes.
///
/// Ne couvre pas : de grands cristaux (voir `etape_g2_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn un_balayage_rend_ce_que_donnent_une_neige_a_la_main_et_les_mesures() {
    let base = base_distincte();
    let valeurs_par_reglage: [(Reglage, [&str; 3]); 7] = [
        (Reglage::Rho, ["0.30", "0.55", "0.9"]),
        (Reglage::Beta, ["1.1", "1.8", "2.6"]),
        (Reglage::Alpha, ["0", "0.2", "0.6"]),
        (Reglage::Theta, ["0", "0.1", "2"]),
        (Reglage::Kappa, ["0", "0.05", "0.5"]),
        (Reglage::Mu, ["0.01", "0.09", "0.4"]),
        (Reglage::Gamma, ["0", "0.002", "0.1"]),
    ];
    let mut a_grandi = 0;
    for (reglage, textes) in valeurs_par_reglage {
        let valeurs: Vec<Fixe> = textes.iter().map(|t| d(t)).collect();
        let resultats = balayage(12, 90, base, reglage, &valeurs).expect("balayage valide");
        assert_eq!(resultats.len(), 3, "{reglage:?}");
        let mut empreintes = Vec::new();
        for (valeur, resultat) in valeurs.iter().zip(&resultats) {
            let mut neige = Neige::essayer_new(12, avec(reglage, base, *valeur)).expect("neige valide");
            neige.avancer(90);
            let mesures = mesures_naives(&rattachees_de(&neige));
            assert_eq!(resultat.valeur, *valeur, "{reglage:?} = {valeur:?} : valeur");
            assert_eq!(resultat.mesures, mesures, "{reglage:?} = {valeur:?} : mesures");
            assert_eq!(resultat.mesures, mesurer(&neige), "{reglage:?} = {valeur:?} : mesures (la bibliothèque)");
            assert_eq!(resultat.forme, classer(&mesures), "{reglage:?} = {valeur:?} : forme");
            assert_eq!(resultat.empreinte, neige.empreinte(), "{reglage:?} = {valeur:?} : empreinte");
            assert_eq!(neige.temps(), 90);
            a_grandi += usize::from(mesures.rattachees > 20);
            empreintes.push(resultat.empreinte.clone());
        }
        empreintes.sort();
        empreintes.dedup();
        assert_eq!(empreintes.len(), 3, "{reglage:?} : trois valeurs, trois empreintes");
    }
    assert!(a_grandi >= 3, "au moins trois cristaux doivent avoir grandi, seuls {a_grandi} ont plus de 20 cases");
}

/// juste : le balayage est déterministe, ses cristaux sont indépendants et l'ordre de la liste est celui du résultat (V-G2-BAL-03) : deux appels donnent les mêmes résultats (`==` sur les structures, empreintes
/// comprises) ; la liste inversée donne les résultats dans l'ordre inverse, chacun inchangé (rien ne passe d'une valeur à la suivante) ; une valeur répétée donne deux résultats égaux ; une liste d'un seul élément
/// est le premier résultat de la liste entière ; le réglage balayé de `base` n'est jamais lu (le même balayage avec `base.rho = 0` ou `base.rho = 1` rend les mêmes résultats quand on balaie `ρ`, même si `base.rho` n'était pas permis).
///
/// Ne couvre pas : la valeur des mesures (voir `un_balayage_rend_ce_que_donnent_une_neige_a_la_main_et_les_mesures`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn determinisme_independance_et_ordre() {
    let base = base_distincte();
    let valeurs = [d("0.4"), d("0.5"), d("0.35"), d("0.5")];
    let un = balayage(10, 60, base, Reglage::Rho, &valeurs).expect("valide");
    let deux = balayage(10, 60, base, Reglage::Rho, &valeurs).expect("valide");
    assert_eq!(un, deux);
    assert_eq!(un[1], un[3], "une valeur répétée donne un résultat égal");
    assert_ne!(un[0], un[1]);
    assert_ne!(un[1], un[2]);
    let inversee: Vec<Fixe> = valeurs.iter().rev().copied().collect();
    let a_l_envers = balayage(10, 60, base, Reglage::Rho, &inversee).expect("valide");
    let mut un_inverse = un.clone();
    un_inverse.reverse();
    assert_eq!(a_l_envers, un_inverse);
    let seul = balayage(10, 60, base, Reglage::Rho, &valeurs[..1]).expect("valide");
    assert_eq!(seul, un[..1].to_vec());
    // Le réglage balayé de `base` n'est jamais lu.
    let autre = Parametres { rho: Fixe::ZERO, ..base };
    assert_eq!(balayage(10, 60, autre, Reglage::Rho, &valeurs).expect("valide"), un);
    let hors_plage = Parametres { rho: Fixe::entier(5), ..base };
    assert_eq!(balayage(10, 60, hors_plage, Reglage::Rho, &valeurs).expect("valide : le ρ de base n'est pas lu"), un);
}

/// juste : chaque refus est un code d'erreur et dans l'ordre de la doc, jamais une panique (V-G2-BAL-04). Le rayon de domaine 0 ou 401 (`RayonHorsBorne`), le nombre de pas 0 ou 100 001 (`PasHorsBorne`), la liste
/// vide (`ListeVide`), 17 valeurs (`ListeTropLongue`), une valeur hors de la plage de son réglage : `ReglageHorsPlage { nom, indice }` avec le NOM du réglage fautif et le RANG de la première valeur fautive dans la
/// liste (pas de la dernière, pas de la valeur) ; un réglage de `base` hors plage donne son nom avec l'indice 0 quand on balaie un autre réglage ; si deux réglages sont fautifs (la valeur et `base`), c'est
/// le premier dans l'ordre `rho`, `beta`, `alpha`, `theta`, `kappa`, `mu`, `gamma`. L'ORDRE des refus : le rayon avant le nombre de pas, le nombre de pas avant la liste vide, la liste vide avant la liste trop longue,
/// la liste trop longue avant les plages (une liste de 17 valeurs dont une est hors plage est `ListeTropLongue`).
///
/// Ne couvre pas : les valeurs extrêmes permises (voir `valeurs_extremes_permises_et_bornes_maximales`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn les_refus() {
    let base = base_distincte();
    let bonnes = [d("0.4"), d("0.5")];
    let un = [d("0.4")];
    assert_eq!(balayage(0, 10, base, Reglage::Rho, &bonnes), Err(ErreurBalayage::RayonHorsBorne));
    assert_eq!(balayage(401, 10, base, Reglage::Rho, &bonnes), Err(ErreurBalayage::RayonHorsBorne));
    assert_eq!(balayage(u32::MAX, 10, base, Reglage::Rho, &bonnes), Err(ErreurBalayage::RayonHorsBorne));
    assert_eq!(balayage(5, 0, base, Reglage::Rho, &bonnes), Err(ErreurBalayage::PasHorsBorne));
    assert_eq!(balayage(5, 100_001, base, Reglage::Rho, &bonnes), Err(ErreurBalayage::PasHorsBorne));
    assert_eq!(balayage(5, u64::MAX, base, Reglage::Rho, &bonnes), Err(ErreurBalayage::PasHorsBorne));
    assert_eq!(balayage(5, 10, base, Reglage::Rho, &[]), Err(ErreurBalayage::ListeVide));
    let dix_sept: Vec<Fixe> = (0..17).map(|k| Fixe(1_000_000 * (k + 1))).collect();
    assert_eq!(balayage(5, 10, base, Reglage::Rho, &dix_sept), Err(ErreurBalayage::ListeTropLongue));
    let mut dix_sept_dont_une_fautive = dix_sept.clone();
    dix_sept_dont_une_fautive[9] = Fixe::entier(3);
    assert_eq!(balayage(5, 10, base, Reglage::Rho, &dix_sept_dont_une_fautive), Err(ErreurBalayage::ListeTropLongue));
    // L'ordre : le rayon avant le pas avant la liste.
    assert_eq!(balayage(0, 0, base, Reglage::Rho, &[]), Err(ErreurBalayage::RayonHorsBorne));
    assert_eq!(balayage(5, 0, base, Reglage::Rho, &[]), Err(ErreurBalayage::PasHorsBorne));
    let base_fautive = Parametres { mu: Fixe::entier(2), ..base };
    assert_eq!(balayage(5, 10, base_fautive, Reglage::Rho, &[]), Err(ErreurBalayage::ListeVide));
    assert_eq!(balayage(5, 10, base_fautive, Reglage::Rho, &dix_sept), Err(ErreurBalayage::ListeTropLongue));
    // Une valeur hors plage : le nom du réglage balayé et le rang de la première valeur fautive.
    let valeurs = [d("0.4"), d("0.5"), Fixe(Fixe::UN.0 + 1), d("0.45"), Fixe(-1)];
    assert_eq!(
        balayage(5, 10, base, Reglage::Rho, &valeurs),
        Err(ErreurBalayage::ReglageHorsPlage { nom: "rho", indice: 2 })
    );
    let beta = [d("1.2"), d("0.9999"), d("8.5")];
    assert_eq!(
        balayage(5, 10, base, Reglage::Beta, &beta),
        Err(ErreurBalayage::ReglageHorsPlage { nom: "beta", indice: 1 })
    );
    let beta_haut = [d("1.2"), d("8"), Fixe(8 * Fixe::UN.0 + 1)];
    assert_eq!(
        balayage(5, 10, base, Reglage::Beta, &beta_haut),
        Err(ErreurBalayage::ReglageHorsPlage { nom: "beta", indice: 2 })
    );
    for (reglage, nom) in TOUS.into_iter().zip(["rho", "beta", "alpha", "theta", "kappa", "mu", "gamma"]) {
        let trop_haut = Fixe::entier(9);
        assert_eq!(
            balayage(5, 10, base, reglage, &[trop_haut]),
            Err(ErreurBalayage::ReglageHorsPlage { nom, indice: 0 }),
            "{nom} = 9"
        );
        let negatif = Fixe(-1);
        assert_eq!(
            balayage(5, 10, base, reglage, &[negatif]),
            Err(ErreurBalayage::ReglageHorsPlage { nom, indice: 0 }),
            "{nom} = -1"
        );
    }
    // Un réglage de `base` hors plage, pour un autre réglage que le balayé : son nom, à l'indice 0.
    assert_eq!(
        balayage(5, 10, base_fautive, Reglage::Rho, &bonnes),
        Err(ErreurBalayage::ReglageHorsPlage { nom: "mu", indice: 0 })
    );
    let base_gamma = Parametres { gamma: Fixe::entier(4), ..base };
    assert_eq!(
        balayage(5, 10, base_gamma, Reglage::Beta, &[d("1.5")]),
        Err(ErreurBalayage::ReglageHorsPlage { nom: "gamma", indice: 0 })
    );
    // Deux réglages fautifs : le premier dans l'ordre de `verifier`.
    let base_beta_fautif = Parametres { beta: Fixe::ZERO, ..base };
    assert_eq!(
        balayage(5, 10, base_beta_fautif, Reglage::Rho, &[Fixe::entier(3)]),
        Err(ErreurBalayage::ReglageHorsPlage { nom: "rho", indice: 0 })
    );
    assert_eq!(
        balayage(5, 10, base_beta_fautif, Reglage::Mu, &[Fixe::entier(3)]),
        Err(ErreurBalayage::ReglageHorsPlage { nom: "beta", indice: 0 })
    );
    // La première valeur bonne mais la base fautive pour la deuxième seulement n'existe pas : la base est la même pour toutes les valeurs.
    assert_eq!(balayage(5, 10, base, Reglage::Rho, &un).map(|r| r.len()), Ok(1));
}

/// juste : les valeurs extrêmes PERMISES et les bornes maximales (V-G2-BAL-05). Chaque réglage à chacune des bornes de sa plage, bornes incluses (`ρ`, `α`, `κ`, `μ`, `γ` : 0 et 1 ; `β` : 1 et 8 ; `θ` : 0 et 8), est
/// accepté et rend les mêmes résultats qu'une neige faite à la main ; un pas de plus que la borne est refusé. Le rayon de domaine maximal (400, un seul pas : 481 201 cases) et le nombre de pas maximal (100 000, sur un
/// domaine de rayon 1) sont acceptés ; 16 valeurs sont acceptées. (Ni le rayon 400 avec 100 000 pas ni 16 cristaux grands ne sont joués : des heures.) Sur le domaine de rayon 1 et 100 000 pas, le cristal est le
/// domaine entier ou aucun germe ne se rattache : le test ne vérifie que l'acceptation, l'empreinte à la main et la cohérence des mesures.
///
/// Ne couvre pas : les refus (voir `les_refus`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn valeurs_extremes_permises_et_bornes_maximales() {
    let base = base_distincte();
    let bornes: [(Reglage, [Fixe; 2]); 7] = [
        (Reglage::Rho, [Fixe::ZERO, Fixe::UN]),
        (Reglage::Beta, [Fixe::UN, Fixe::entier(8)]),
        (Reglage::Alpha, [Fixe::ZERO, Fixe::UN]),
        (Reglage::Theta, [Fixe::ZERO, Fixe::entier(8)]),
        (Reglage::Kappa, [Fixe::ZERO, Fixe::UN]),
        (Reglage::Mu, [Fixe::ZERO, Fixe::UN]),
        (Reglage::Gamma, [Fixe::ZERO, Fixe::UN]),
    ];
    for (reglage, bords) in bornes {
        let resultats =
            balayage(6, 25, base, reglage, &bords).unwrap_or_else(|e| panic!("{reglage:?} aux bornes : {e:?}"));
        assert_eq!(resultats.len(), 2);
        for (valeur, resultat) in bords.iter().zip(&resultats) {
            let mut neige = Neige::essayer_new(6, avec(reglage, base, *valeur)).expect("neige valide");
            neige.avancer(25);
            assert_eq!(resultat.empreinte, neige.empreinte(), "{reglage:?} = {valeur:?}");
            assert_eq!(resultat.mesures, mesures_naives(&rattachees_de(&neige)), "{reglage:?} = {valeur:?}");
        }
    }
    // Juste au-delà des bornes hautes : refusé.
    for (reglage, borne) in [
        (Reglage::Rho, 1),
        (Reglage::Beta, 8),
        (Reglage::Alpha, 1),
        (Reglage::Theta, 8),
        (Reglage::Kappa, 1),
        (Reglage::Mu, 1),
        (Reglage::Gamma, 1),
    ] {
        let au_dessus = Fixe(Fixe::entier(borne).0 + 1);
        assert!(
            matches!(
                balayage(4, 5, base, reglage, &[au_dessus]),
                Err(ErreurBalayage::ReglageHorsPlage { indice: 0, .. })
            ),
            "{reglage:?} juste au-dessus de {borne}"
        );
    }
    // Le rayon maximal, un pas ; le nombre de pas maximal, un petit domaine ; seize valeurs.
    let grand = balayage(400, 1, base, Reglage::Rho, &[d("0.5")]).expect("le rayon 400 est permis");
    assert_eq!(grand.len(), 1);
    // Un seul pas depuis le germe : une case voisine ne se rattache qu'avec une masse de bord d'au moins `β > 1`, or elle porte au plus la vapeur `ρ = 0,5` : le cristal est encore le germe seul.
    assert_eq!(
        grand[0].mesures,
        Mesures { rattachees: 1, rayon: 0, compacite: 1000, pointes: 1, aiguilles: 1, hexagonalite: 1000 }
    );
    assert_eq!(grand[0].forme, Forme::Aucune);
    let mut neige = Neige::essayer_new(400, Parametres { rho: d("0.5"), ..base }).expect("neige valide");
    neige.avancer(1);
    assert_eq!(grand[0].empreinte, neige.empreinte());
    let long = balayage(1, 100_000, base, Reglage::Rho, &[d("0.5")]).expect("100 000 pas sont permis");
    let mut petite = Neige::essayer_new(1, Parametres { rho: d("0.5"), ..base }).expect("neige valide");
    petite.avancer(100_000);
    assert_eq!(long[0].empreinte, petite.empreinte());
    assert_eq!(long[0].mesures, mesures_naives(&rattachees_de(&petite)));
    let seize: Vec<Fixe> = (0..16).map(|k| Fixe(Fixe::UN.0 / 20 * (k + 1))).collect();
    assert_eq!(seize.len(), VALEURS_MAX);
    let seize_resultats = balayage(3, 5, base, Reglage::Rho, &seize).expect("seize valeurs sont permises");
    assert_eq!(seize_resultats.len(), 16);
    for (v, r) in seize.iter().zip(&seize_resultats) {
        assert_eq!(r.valeur, *v);
    }
    assert_eq!((RAYON_MAX_BALAYAGE, PAS_MAX_BALAYAGE), (400, 100_000));
}

/// juste : les trois jeux de balayage sont ceux que la doc et la fiche écrivent (V-G2-JEU-01 à V-G2-JEU-03) : `jeu_rho()`, `jeu_beta()`, `jeu_mu()` (le nom, le réglage balayé, les six valeurs lues exactement par `lire_decimal`, les sept
/// réglages de base, le rayon de domaine et le nombre de pas). Les valeurs balayées sont celles des légendes des figures 9, 10 et 12 de Gravner et Griffeath 2008 (vues sur les images de la préimpression), les autres réglages
/// sont ceux de l'annexe 8, lignes « 9 », « 10 » et « 12 » (voir la fiche : la colonne du réglage qui varie manque dans ces lignes ; l'attribution est une déduction recoupée). Un chiffre faux dans un jeu (un `0.044` pour
/// `0.04`, un `γ` de la ligne « 12 » pris pour le `μ`) échoue ici.
///
/// Ne couvre pas : que ces lignes de l'annexe soient les bonnes (V-G2-05 : ambiguïté de la source dite dans la fiche) ; ce que les jeux donnent (voir `etape_g2_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn les_trois_jeux_sont_ceux_de_la_fiche() {
    fn verifier(
        jeu: JeuDeBalayage,
        nom: &str,
        reglage: Reglage,
        valeurs: [&str; 6],
        base: [&str; 7],
        rayon: u32,
        pas: u64,
    ) {
        assert_eq!(jeu.nom, nom);
        assert_eq!(jeu.reglage, reglage);
        let attendues: Vec<Fixe> = valeurs.iter().map(|v| d(v)).collect();
        assert_eq!(jeu.valeurs, attendues, "{nom} : valeurs");
        let p = parametres(base[0], base[1], base[2], base[3], base[4], base[5], base[6]);
        assert_eq!(jeu.base, p, "{nom} : réglages de base");
        assert_eq!(
            jeu.reglage.fixer(jeu.base, jeu.valeurs[0]),
            jeu.base,
            "{nom} : la base porte la première valeur du réglage balayé"
        );
        assert_eq!((jeu.rayon, jeu.pas), (rayon, pas), "{nom} : taille");
        assert!(jeu.base.verifier().is_ok());
    }
    verifier(
        jeu_rho(),
        "rho",
        Reglage::Rho,
        ["0.40", "0.41", "0.42", "0.44", "0.46", "0.50"],
        ["0.40", "1.3", "0.08", "0.025", "0.003", "0.07", "0.00005"],
        100,
        2000,
    );
    verifier(
        jeu_beta(),
        "beta",
        Reglage::Beta,
        ["1.9", "2.2", "2.4", "2.6", "2.7", "2.8"],
        ["0.8", "1.9", "0.004", "0.001", "0.05", "0.015", "0.0001"],
        100,
        1300,
    );
    verifier(
        jeu_mu(),
        "mu",
        Reglage::Mu,
        ["0.04", "0.05", "0.06", "0.07", "0.08", "0.09"],
        ["0.5", "1.4", "0.1", "0.005", "0.001", "0.04", "0.001"],
        100,
        2000,
    );
}

/// juste : l'écriture du CSV est celle de la doc, octet pour octet (V-G2-CSV-01). `EN_TETE_CSV` est la ligne `balayage,valeur,rayon_domaine,pas,rattachees,rayon,compacite,pointes,aiguilles,hexagonalite,forme,empreinte` suivie
/// d'un seul `\n` ; `ligne_csv` écrit les douze champs séparés par des virgules, la valeur en six décimales EXACTES (`ppm = (2_000_000 * brut + 2^32) / 2^33`, arrondi au plus proche : `0.41` donne `0.410000`, un tiers
/// `0.333333`, deux tiers `0.666667`, `8` donne `8.000000`, `brut = 1` donne `0.000000`, le demi-cas vers le haut : `1/128 = 0,0078125` donne `0.007813`), le nom de la forme (`aucune`, `plaque`, `secteurs`, `dendrite`, `fougere`), et finit par un seul `\n` (jamais de `\r`).
///
/// Ne couvre pas : le fichier que l'exemple écrit ; les noms de balayage hors de `rho`, `beta`, `mu` (n'importe quel texte est recopié).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn ecriture_du_csv_octet_pour_octet() {
    assert_eq!(
        EN_TETE_CSV,
        "balayage,valeur,rayon_domaine,pas,rattachees,rayon,compacite,pointes,aiguilles,hexagonalite,forme,empreinte\n"
    );
    let empreinte = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff".to_string();
    let resultat = |valeur: Fixe, forme: Forme| Resultat {
        valeur,
        mesures: Mesures { rattachees: 1687, rayon: 26, compacite: 800, pointes: 18, aiguilles: 12, hexagonalite: 500 },
        forme,
        empreinte: empreinte.clone(),
    };
    let ligne = |valeur: Fixe, forme: Forme| ligne_csv("rho", 84, 2000, &resultat(valeur, forme));
    let fin = ",84,2000,1687,26,800,18,12,500";
    assert_eq!(ligne(d("0.41"), Forme::Secteurs), format!("rho,0.410000{fin},secteurs,{empreinte}\n"));
    assert_eq!(ligne(Fixe::rapport(1, 3), Forme::Plaque), format!("rho,0.333333{fin},plaque,{empreinte}\n"));
    assert_eq!(ligne(Fixe::rapport(2, 3), Forme::Dendrite), format!("rho,0.666667{fin},dendrite,{empreinte}\n"));
    assert_eq!(ligne(Fixe::entier(8), Forme::Fougere), format!("rho,8.000000{fin},fougere,{empreinte}\n"));
    assert_eq!(ligne(Fixe(1), Forme::Aucune), format!("rho,0.000000{fin},aucune,{empreinte}\n"));
    assert_eq!(ligne(Fixe::ZERO, Forme::Aucune), format!("rho,0.000000{fin},aucune,{empreinte}\n"));
    assert_eq!(ligne(d("2.8"), Forme::Plaque), format!("rho,2.800000{fin},plaque,{empreinte}\n"));
    assert_eq!(ligne(d("0.00005"), Forme::Plaque), format!("rho,0.000050{fin},plaque,{empreinte}\n"));
    // Le demi-cas va vers le haut : 1/128 = 0,0078125 s'écrit 0.007813 (et 3/128 = 0,0234375 s'écrit 0.023438), pas 0.007812 ; un cheveu en dessous (`brut - 1`) s'écrit 0.007812.
    assert_eq!(ligne(Fixe(1 << 25), Forme::Plaque), format!("rho,0.007813{fin},plaque,{empreinte}\n"));
    assert_eq!(ligne(Fixe(3 << 25), Forme::Plaque), format!("rho,0.023438{fin},plaque,{empreinte}\n"));
    assert_eq!(ligne(Fixe((1 << 25) - 1), Forme::Plaque), format!("rho,0.007812{fin},plaque,{empreinte}\n"));
    // La partie entière et les six chiffres : 7,000001 ; 0,9999995 s'arrondit à 1.000000.
    assert_eq!(ligne(d("7.000001"), Forme::Plaque), format!("rho,7.000001{fin},plaque,{empreinte}\n"));
    assert_eq!(ligne(d("0.9999995"), Forme::Plaque), format!("rho,1.000000{fin},plaque,{empreinte}\n"));
    // Le nom du balayage, le rayon, le pas et chaque mesure sont écrits à leur place.
    let autre = Resultat {
        valeur: d("0.09"),
        mesures: Mesures { rattachees: 2209, rayon: 30, compacite: 791, pointes: 30, aiguilles: 6, hexagonalite: 566 },
        forme: Forme::Secteurs,
        empreinte: "ab".repeat(32),
    };
    assert_eq!(
        ligne_csv("mu", 12, 7, &autre),
        format!("mu,0.090000,12,7,2209,30,791,30,6,566,secteurs,{}\n", "ab".repeat(32))
    );
    assert_eq!(ligne_csv("a b", 1, 1, &autre).matches('\n').count(), 1);
    assert!(!ligne_csv("mu", 12, 7, &autre).contains('\r'));
    // Les noms des formes, ceux du CSV.
    let noms = [
        (Forme::Aucune, "aucune"),
        (Forme::Plaque, "plaque"),
        (Forme::Secteurs, "secteurs"),
        (Forme::Dendrite, "dendrite"),
        (Forme::Fougere, "fougere"),
    ];
    for (forme, nom) in noms {
        assert_eq!(forme.nom(), nom);
    }
}

/// juste : la `forme` d'un résultat est l'étiquette de SES mesures, aux endroits où l'étiquette bascule (V-G2-BAL-06). Trois petits balayages de `β` (réglages de la ligne « 10 », domaine de rayon 22, 150, 200 et 300 pas) donnent des cristaux
/// des cinq formes, dont un de rayon 9 (`Aucune` : la règle 1) et un de rayon 10 (la règle 1 ne s'applique plus). Pour chaque résultat : les mesures sont celles du calcul naïf du test sur une `Neige` faite à la main, et la forme est celle de la
/// table de la doc RÉÉCRITE ICI (`rayon < 10`, puis `compacite >= 950`, puis `>= 700`, puis `aiguilles >= 6`, sinon `Dendrite`) appliquée à ces mesures. Un balayage qui tirerait l'étiquette d'autres mesures que celles qu'il rend (d'un rayon
/// décalé, du cristal d'avant, d'avant le dernier pas) échoue. Les rayons 9 et 10 et les cinq formes sont exigés pour que le test ne soit pas à vide : ce sont des faits de notre modèle (G1), pas des valeurs publiées.
///
/// Ne couvre pas : que la table soit un bon classement (c'est un modèle à nous) ; de grands cristaux (voir `etape_g2_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-g2"), ignore)]
fn la_forme_d_un_resultat_est_celle_de_ses_mesures() {
    let table = |mesures: &Mesures| {
        if mesures.rayon < 10 {
            Forme::Aucune
        } else if mesures.compacite >= 950 {
            Forme::Plaque
        } else if mesures.compacite >= 700 {
            Forme::Secteurs
        } else if mesures.aiguilles >= 6 {
            Forme::Fougere
        } else {
            Forme::Dendrite
        }
    };
    let base = parametres("0.8", "1.9", "0.004", "0.001", "0.05", "0.015", "0.0001");
    let essais: [(u64, &[&str]); 3] = [(150, &["1.4"]), (200, &["1.4", "1.6", "2.0", "2.3"]), (300, &["2.6", "3.0"])];
    let mut vus: Vec<(u32, Forme)> = Vec::new();
    for (pas, textes) in essais {
        let valeurs: Vec<Fixe> = textes.iter().map(|t| d(t)).collect();
        let resultats = balayage(22, pas, base, Reglage::Beta, &valeurs).expect("balayage valide");
        assert_eq!(resultats.len(), valeurs.len());
        for (valeur, resultat) in valeurs.iter().zip(&resultats) {
            let mut neige = Neige::essayer_new(22, avec(Reglage::Beta, base, *valeur)).expect("neige valide");
            neige.avancer(pas);
            let mesures = mesures_naives(&rattachees_de(&neige));
            assert_eq!(resultat.mesures, mesures, "β = {valeur:?}, {pas} pas : mesures");
            assert_eq!(resultat.forme, table(&mesures), "β = {valeur:?}, {pas} pas : forme de {mesures:?}");
            vus.push((mesures.rayon, resultat.forme));
        }
    }
    // Sept cristaux ; le rayon 9 est `Aucune`, le rayon 10 ne l'est plus, et les cinq formes sont là.
    assert_eq!(vus.len(), 7);
    assert!(vus.contains(&(9, Forme::Aucune)), "{vus:?}");
    assert!(vus.iter().any(|&(rayon, forme)| rayon == 10 && forme != Forme::Aucune), "{vus:?}");
    for forme in [Forme::Aucune, Forme::Plaque, Forme::Secteurs, Forme::Dendrite, Forme::Fougere] {
        assert!(vus.iter().any(|&(_, f)| f == forme), "{forme:?} manque : {vus:?}");
    }
}
