//! Étape P2 : des fautes plausibles, qui DOIVENT échouer. Chaque faute est une variante du naïf de `etape_p2_commun` (voir `Faute` : le signe
//! de l'anisotropie, `epsilon` sur `W` mais pas sur `tau`, les termes croisés oubliés, la garde de `|∇φ| = 0` absente ou à un seuil faux, le
//! `1/2` de la chaleur latente oublié, une constante calculée sous deux écritures, une mise à jour en place, un bord qui fuit, des sommes
//! sans paires, `a1` pour `a2` dans `tau0`). Valeurs attendues : `references/etape-p2-valeurs.md`, identifiants `V-P2-FAUTE-xx`.

mod etape_p2_commun;

use etape_p2_commun::*;

/// Les trois « vues » d'une faute, dans cet ordre : `[bits, symétries, front d'axe]` :
/// - `bits` : sur au moins un de trois petits cas (`etape_p2_commun::bits_differents`), les bits du naïf fautif diffèrent de ceux de la
///   bibliothèque (c'est ce qui fait échouer un code qui aurait cette faute contre le naïf de `etape_p2_naif.rs`) ;
/// - `symétries` : le naïf fautif perd les huit symétries pour chacun des huit couples (`n`, graine) de `symetrie_perdue` (c'est ce que voit
///   `etape_p2_symetrie.rs`) ;
/// - `front d'axe` : le naïf fautif sort de la tolérance du front droit d'axe (c'est ce que voit `etape_p2_exact.rs`).
fn vues(f: Faute) -> [bool; 3] {
    let perdues = symetrie_perdue(Some(f));
    assert!(
        perdues.iter().all(|x| *x == perdues[0]),
        "la perte de symétrie de {f:?} devait être la même pour les huit couples : {perdues:?}"
    );
    [bits_differents(Some(f)), perdues[0], axe_hors_tolerance(Some(f))]
}

/// La table attendue (mesurée sur le prototype) : `(faute, [bits, symétries, front d'axe])`.
fn attendue() -> Vec<(Faute, [bool; 3])> {
    vec![
        (Faute::SigneAnisotropie, [true, false, true]),
        (Faute::TauSansAnisotropie, [true, false, true]),
        // sur un front droit ET sur un front en diagonale les termes croisés sont nuls : seuls les bits (et une pointe courbe) la voient
        (Faute::TermesCroisesOublies, [true, false, false]),
        (Faute::GardeAbsente, [true, false, true]),
        // la garde à 1e-12 ne change que les cases de gradient minuscule : seuls les bits la voient
        (Faute::GardeSeuilFaux, [true, false, false]),
        // le front d'axe est à `u` gelé : le `1/2` ne s'y voit pas
        (Faute::DemiOublie, [true, false, false]),
        (Faute::ConstanteDeuxFois, [true, true, false]),
        (Faute::EnPlace, [true, true, true]),
        (Faute::BordFuit, [true, false, true]),
        (Faute::SansPairesU, [true, true, false]),
        (Faute::SansPairesGt, [true, true, false]),
        // `a1` pour `a2` ne change que `tau0` de `depuis_capillarite` : seuls les bits des cas qui l'emploient la voient
        (Faute::A1PourA2, [true, false, false]),
    ]
}

/// juste : (V-P2-FAUTE-01) la table des fautes est AFFIRMÉE EN ENTIER : pour chacune des douze fautes de `TOUTES_LES_FAUTES`, dans l'ordre,
/// ses trois vues (`[bits, symétries, front d'axe]`, voir `vues`) sont exactement celles de `attendue()` (`assert_eq!` sur toute la table,
/// pas seulement ses entrées vraies), et le naïf SANS faute n'est vu par aucun des trois (le point de départ : sinon les fautes ne
/// prouveraient rien). Toute faute est vue par au moins un test (la colonne `bits`), et plusieurs ne le sont que par lui : c'est
/// l'égalité bit à bit au naïf (`etape_p2_naif.rs`) qui est le vrai garde-fou.
///
/// Ne couvre pas : les fautes que personne n'a imaginées ; des fautes combinées ; la vue des fautes par la pointe (voir `etape_p2_pointe.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn la_table_des_fautes_en_entier() {
    let sans_faute = [bits_differents(None), symetrie_perdue(None)[0], axe_hors_tolerance(None)];
    assert_eq!(symetrie_perdue(None), vec![false; 8]);
    assert_eq!(sans_faute, [false, false, false], "le naïf sans faute devait passer les trois vues");
    let vue: Vec<(Faute, [bool; 3])> = TOUTES_LES_FAUTES.iter().map(|&f| (f, vues(f))).collect();
    assert_eq!(vue, attendue());
}

/// juste : (V-P2-FAUTE-02) chacune des douze fautes donne des bits DIFFÉRENTS de ceux de la bibliothèque sur au moins un des trois petits cas
/// de `etape_p2_commun::bits_differents` (un départ tiré, la graine ronde, un gradient minuscule ; leurs états de départ sont aussi
/// ceux de la table), tandis que le naïf SANS faute les donne identiques sur les trois cas (vérifié aussi ici, pour qu'une bibliothèque
/// fausse ne passe pas ce test à vide). C'est la colonne `bits` de la table, rejouée faute par faute avec un message qui nomme la faute.
///
/// Ne couvre pas : la tolérance (voir le test précédent).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn chaque_faute_change_les_bits() {
    assert!(!bits_differents(None), "le naïf sans faute devait donner les bits de la bibliothèque");
    for f in TOUTES_LES_FAUTES {
        assert!(bits_differents(Some(f)), "la faute {f:?} n'a pas changé les bits");
    }
}
