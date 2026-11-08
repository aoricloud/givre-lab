//! Étape 3 : les valeurs PUBLIÉES par Reiter (la page de matériaux auxiliaires de son article, tables β-γ, et le texte de la préimpression).
//! Valeurs attendues : `references/etape-3-valeurs.md`, identifiants `V-PUB-xx`, avec leur source et ce qui n'a pas pu être lu.
//!
//! **La source.** C. A. Reiter, « A local cellular model for snow crystal growth », *Chaos, Solitons & Fractals* 23 (2005) 1111-1119 ; la
//! préimpression `webbox.lafayette.edu/~reiterc/mvp/sfn/sfn_pp.pdf` et la page `webbox.lafayette.edu/~reiterc/mvp/sfn/nn/index.html` (« Beta-Gamma
//! Table 100 : A β-γ Table of Radius 100 »), lues le 8/11/2026. La table a 240 vignettes, une par couple `(β, γ)` avec `β` de 0 à 0,95 par pas de
//! 0,05 et `γ` dans `{0, 0.0001, 0.001, 0.002, 0.0035, 0.005, 0.01, 0.02, 0.05, 0.1, 0.5, 1}` ; le nom de chaque image est
//! `th_nn_h<β>_x<γ>_<n>.png` (`o` pour la virgule), et sa légende « β = …, γ = … ».
//!
//! **Le nombre `n`, lu dans le code de l'auteur** (`webbox.lafayette.edu/~reiterc/mvp/sfn/sfn.html`, le script J `sfn.ijs`, lu par le juge le
//! 8/11/2026) : la fonction `mk_dla` (rayon de région 100, au plus 10 000 pas) fait un pas tant qu'aucune glace n'a atteint le cercle intérieur
//! `icir`, puis rend `k - 1` où `k` est le nombre de pas faits : `n` est donc le nombre de pas moins un (9999 au plafond). La géométrie de l'auteur
//! n'est pas celle de ce modèle : son tableau de `202 x 202` cases a des rangées espacées de `sin 60°` ; le bord fixé à `β` est l'ensemble des cases
//! à distance euclidienne `d >= 100 sin 60° ≈ 86,6` du germe (`ocir`), et l'arrêt a lieu dès qu'une glace a `d >= 98 sin 60° ≈ 84,9` (`icir`), soit
//! le rayon hexagonal 85 sur les axes des branches. Un modèle indépendant du juge, écrit avec cette géométrie et les arrondis de la doc de
//! `flocon`, redonne **les 240 numéros de la table, tous exactement**. Ici le bord est la couronne d'un hexagone de rayon 100 (les cases à
//! distance euclidienne 86,6 à 100), plus loin du cristal le long des branches que le cercle de 86,6 de l'auteur : la vapeur arrive moins vite.
//! Pour `β <= 0.5` et `γ <= 0.05` (59 cases : les dendrites que la vapeur nourrit), ce modèle met donc 0,1 % à 4,9 % de pas en plus que la table ;
//! **ces cas ne sont pas ici**, et la raison en est établie : la géométrie du bord, non une règle. Pour les 160 autres cases non plafonnées (toutes
//! celles où `β >= 0.55` ou `γ >= 0.1`, et la ligne `β = 0` pour `γ >= 0.01`) et les 21 plafonnées, les deux géométries donnent le même nombre
//! (vérifié par le juge sur toute la table) : le bord n'y change rien, et ces cases sont une valeur publiée que ce modèle doit atteindre.
//! (`β = 0` : `n = ceil(85 / γ) - 1` pour `γ >= 0.01`, `9999` pour `γ <= 0.005` car `85 / γ > 10000` ; `β >= 0.9` : `n = 84`, un anneau par pas.)

mod etape_3_commun;

use etape_3_commun::parametres;
use givre_lab::flocon::Flocon;
use givre_lab::math::Fixe;

/// Le nombre de pas pour que le rayon du cristal atteigne `cible`, en s'arrêtant à `limite` pas (rend `limite + 1` si ce n'est pas atteint). Les
/// pas sont comptés ici (un `temps` qui ne bougerait pas ne fait pas boucler le test) ; `temps` doit valoir ce compte.
fn pas_pour_atteindre(rayon_region: u32, alpha: &str, beta: &str, gamma: &str, cible: u32, limite: u64) -> u64 {
    let mut f = Flocon::new(rayon_region, parametres(alpha, beta, gamma));
    let mut faits = 0;
    while f.rayon_cristal() < cible {
        if faits >= limite {
            return limite + 1;
        }
        f.pas();
        faits += 1;
    }
    assert_eq!(f.temps(), faits, "temps compte les pas");
    faits
}

/// juste : la ligne `β = 0` de la table publiée (V-PUB-01). Sans vapeur (`β = 0`) la diffusion ne fait rien et un anneau de glace met `m =
/// ceil(1 / γ)` pas à geler (voir `etape_3_exact`) ; atteindre le rayon 85 prend donc `85 m` pas, soit un dernier numéro d'image `85 m - 1`. Les
/// numéros publiés (`th_nn_h0_x<γ>_<n>.png`) sont : `γ = 0.01` : 8499, `0.02` : 4249, `0.05` : 1699, `0.1` : 849, `0.5` : 169, `1` : 84 ;
/// et `9999` (10 000 pas, le plafond) pour `0.0001, 0.001, 0.002, 0.0035, 0.005` (car `85 m >= 10000`) et pour `γ = 0`. Le test recalcule `m`
/// en entiers sur le `brut` lu de `γ` (`m = ceil(2^32 / brut)`), vérifie ces 12 numéros, puis **simule** la région de rayon 100 pour
/// `γ = 1, 0.5, 0.1` et vérifie que le rayon 85 est atteint à `85 m` pas exactement (et pas un de moins).
///
/// Ne couvre pas : les autres lignes de la table (voir `table_publiee_rayon_100`), les lenteurs de la table qui plafonne à 10 000 pas.
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn ligne_beta_nul_de_la_table() {
    let publie: [(&str, u64); 12] = [
        ("0", 9999),
        ("0.0001", 9999),
        ("0.001", 9999),
        ("0.002", 9999),
        ("0.0035", 9999),
        ("0.005", 9999),
        ("0.01", 8499),
        ("0.02", 4249),
        ("0.05", 1699),
        ("0.1", 849),
        ("0.5", 169),
        ("1", 84),
    ];
    for (gamma, numero) in publie {
        let p = parametres("1", "0", gamma);
        let attendu = if p.gamma == Fixe::ZERO {
            9999
        } else {
            let m = (Fixe::UN.0 + p.gamma.0 - 1) / p.gamma.0;
            (85 * m - 1).min(9999) as u64
        };
        assert_eq!(attendu, numero, "gamma {gamma}");
    }
    for (gamma, m) in [("1", 1_u64), ("0.5", 2), ("0.1", 10)] {
        assert_eq!(pas_pour_atteindre(100, "1", "0", gamma, 85, 85 * m), 85 * m, "gamma {gamma}");
    }
}

/// juste : vingt-trois cases de la table publiée de rayon 100 (V-PUB-02), `α = 1`, simulées sur la région de rayon 100 : le nombre de pas pour que
/// le rayon du cristal atteigne 85 vaut `n + 1` où `n` est le numéro lu dans le nom de l'image (voir l'en-tête du fichier). Les cases couvrent
/// les régimes : des plaques (`β = 0`), des dendrites rapides (`β = 0.2` à `0.4` avec `γ` grand), des étoiles, et surtout les **égalités à un pas
/// près** du haut de la table : `(0.55, 0.005) : 671` contre `(0.55, 0.002) : 672`, `(0.7, 0.005) : 334` contre `(0.7, 0.002) : 335`, `(0.75, 0.1) : 168`
/// contre `(0.75, 0.02) : 169`, `(0.65, 0.0035) : 420`, `(0.85, 0.005) : 168` contre `(0.85, 0.01) : 84`. Ces valeurs de la page de l'auteur ne
/// dépendent d'aucun réglage du prototype : une règle de réceptivité, un seuil `>=`, une diffusion, un arrondi ou un `γ` différents les déplacent.
/// Les 240 noms de fichiers ont été relus par le juge dans le HTML de la page, téléchargé le 8/11/2026 : identiques à la lecture de l'écrivain.
///
/// Ne couvre pas : les cases `β <= 0.5` avec `γ <= 0.05` (ce modèle y est 0,1 à 4,9 % plus lent : valeur publiée non atteinte, à cause du bord
/// circulaire plus proche de l'auteur, voir l'en-tête), les autres rayons de région
/// (200 et 400 de l'auteur).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn table_publiee_rayon_100() {
    // (β, γ, numéro publié n)
    let table: [(&str, &str, u64); 23] = [
        ("0", "0.1", 849),
        ("0", "0.5", 169),
        ("0.2", "0.1", 593),
        ("0.3", "0.05", 762),
        ("0.4", "0.5", 168),
        ("0.45", "0.1", 338),
        ("0.5", "0.1", 254),
        ("0.55", "0.005", 671),
        ("0.55", "0.002", 672),
        ("0.6", "0.0035", 505),
        ("0.6", "0.01", 421),
        ("0.65", "0.0035", 420),
        ("0.65", "0.1", 169),
        ("0.7", "0.005", 334),
        ("0.7", "0.002", 335),
        ("0.75", "0.1", 168),
        ("0.75", "0.02", 169),
        ("0.8", "0.05", 168),
        ("0.8", "0.1", 84),
        ("0.85", "0.005", 168),
        ("0.85", "0.01", 84),
        ("0.9", "0", 84),
        ("0.95", "0", 84),
    ];
    let resultats = std::sync::Mutex::new(Vec::new());
    let suivant = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                loop {
                    let i = suivant.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(&(beta, gamma, n)) = table.get(i) else { break };
                    let pas = pas_pour_atteindre(100, "1", beta, gamma, 85, n + 2);
                    resultats.lock().unwrap().push((beta, gamma, n, pas));
                }
            });
        }
    });
    let resultats = resultats.into_inner().unwrap();
    assert_eq!(resultats.len(), table.len());
    for (beta, gamma, n, pas) in resultats {
        assert_eq!(pas, n + 1, "β = {beta}, γ = {gamma} : publié {n} (soit {} pas), mesuré {pas} pas", n + 1);
    }
}

/// juste : « When β = 0 and γ = 0, no growth appears » (la préimpression, section « Model and Parameter Diagram », V-PUB-03) : sans vapeur ni
/// masse ajoutée, `R = 10`, 300 pas : tout l'état reste celui du départ (le centre vaut 1, les autres cases 0), un seul gelé, rayon du cristal 0. Un
/// `γ` implicite, ou de la diffusion qui crée de la masse, échouent ici. La table publiée dit la même chose : `th_nn_h0_x0_09999.png` (le plafond de
/// 10 000 pas, rien ne grandit).
///
/// Ne couvre pas : le cas `β > 0, γ = 0` (voir `table_publiee_rayon_100`).
#[test]
#[cfg_attr(not(feature = "etape-3"), ignore)]
fn sans_vapeur_ni_masse_rien_ne_grandit() {
    for alpha in ["1", "0.5", "2"] {
        let mut f = Flocon::new(10, parametres(alpha, "0", "0"));
        let depart = f.masses();
        f.avancer(300);
        assert_eq!(f.masses(), depart, "alpha {alpha}");
        assert_eq!((f.gelees(), f.rayon_cristal()), (1, 0));
    }
}
