//! La vitrine (étape 5) : la DLA, les flocons de Reiter et (étape D3) le claquage diélectrique pilotés par des nombres seulement, pour une page web ; depuis l'étape V7 elle donne
//! aussi, pour les modèles sur la grille hexagonale, une image nette (chaque case dessinée comme un hexagone lisse) ; depuis l'étape G3 elle fait aussi croître la neige de
//! Gravner et Griffeath (étapes G1 et G2), réglée par quatre nombres ou par un jeu de réglages nommé d'après une figure de l'article.
//!
//! Ce module est compilé deux fois : en natif (les tests appellent ses fonctions comme n'importe quelle fonction Rust) et en
//! WebAssembly (`wasm32-unknown-unknown`, module sans importation). Chaque fonction `givre_*` est exportée sous son nom
//! (`#[unsafe(no_mangle)] pub extern "C" fn`) et ne prend et ne rend que des nombres : un `u32` devient un `i32` côté
//! WebAssembly (JavaScript le lit signé : `>>> 0` le relit non signé), un `f64` reste un `f64`. Il n'y a ni pointeur ni
//! mémoire à manipuler côté JavaScript. **Trente-deux fonctions, ni plus ni moins** (les onze de l'étape 5, les quatre de l'étape Z1, voir
//! « La mesure de la corrélation », les huit de l'étape D3, voir « La courbe D(η) à la demande », les quatre de l'étape V7, voir « L'image nette », et les cinq de l'étape G3, voir « La neige »), toutes de ce
//! fichier ; aucune autre n'est exportée (le module exporte aussi `memory`, que la cible ajoute, et rien d'autre).
//!
//! **Aucune de ces fonctions ne panique, jamais** : une panique dans une `extern "C" fn` fait avorter le processus (le
//! binaire de test tout entier, ou, en WebAssembly, une instruction `unreachable` que la page ne peut pas rattraper). Pas de
//! `unwrap`, pas de `expect`, pas d'indice `[i]` hors de portée, pas de `todo!()` : `.get()`, des bornes vérifiées, et les
//! valeurs « hors de portée » dites plus bas. Les fonctions de la bibliothèque qui paniquent (`Amas::ajouter`, `Claquage::ajouter`,
//! `point_de_la_courbe`) ne sont pas appelées ici : on emploie `Amas::essayer_ajouter`, `Claquage::essayer_ajouter` et les entrées
//! `essayer_*` de [`crate::courbe`], qui rendent une erreur au lieu de paniquer (étape D3). Un cas absurde donne une valeur « hors de portée » ou
//! un **code d'erreur** (voir « La courbe D(η) à la demande »), jamais une panique.
//!
//! # Le module WebAssembly
//!
//! `docs/givre.wasm` est commité. Il se construit, depuis la racine du dépôt, par :
//!
//! ```text
//! cargo rustc --release --lib --target wasm32-unknown-unknown --crate-type cdylib
//! ```
//!
//! puis `target/wasm32-unknown-unknown/release/givre_lab.wasm` se copie en `docs/givre.wasm` (et **pas** `cargo build` : il
//! produirait le module du binaire `givre`, qui exporte aussi `main`). `Cargo.toml` ne change pas. Les octets du module
//! portent les lignes des `panic!` de la bibliothèque : tout changement de `src/` change le module ; les tests comparent
//! son **comportement**, pas ses octets. `tests/etape_5_artefact.rs` fait tourner ce fichier avec Node et compare à ceux du
//! code natif son empreinte et ce que lisent ses accesseurs : si le code change sans que le module soit reconstruit, il
//! échoue.
//!
//! # Les huit modèles
//!
//! `givre_demarrer(modele, graine)` choisit un modèle par son numéro. Les réglages des modèles 0 à 3 sont ceux des scénarios d'exemple du
//! dépôt (les tests relisent ces fichiers et vérifient qu'ils disent bien ceci) ; ceux des modèles 4 à 7 sont écrits ici (étape D3) :
//!
//! | `modele` | quoi | réglages (le scénario) | `duree` |
//! |---|---|---|---|
//! | 0 | la DLA, [`crate::dla::Amas`] | `scenarios/dla.txt` : `boite 401 401`, donc `Amas::new(graine, 200)` ; `dla 3000` : 3000 sites, germe compris | 2999 |
//! | 1 | Reiter, la plaque, [`crate::flocon::Flocon`] | `scenarios/reiter-plaque.txt` : `hexagone 30`, `reiter 1 0 0.05 400` | 400 |
//! | 2 | Reiter, l'étoile | `scenarios/reiter-etoile.txt` : `hexagone 40`, `reiter 1 0.45 0.1 120` | 120 |
//! | 3 | Reiter, la dendrite | `scenarios/reiter-dendrite.txt` : `hexagone 60`, `reiter 1 0.4 0.001 800` | 800 |
//! | 4 | le claquage diélectrique `η = 0`, [`crate::claquage::Claquage`] | `Claquage::new(graine, 70, 0)`, jusqu'à 400 sites, germe compris | 399 |
//! | 5 | le claquage `η = 1` | `Claquage::new(graine, 52, 1)`, jusqu'à 250 sites | 249 |
//! | 6 | le claquage `η = 2` | `Claquage::new(graine, 49, 2)`, jusqu'à 150 sites | 149 |
//! | 7 | le claquage `η = 3` | `Claquage::new(graine, 48, 3)`, jusqu'à 100 sites | 99 |
//!
//! Pour les modèles 1 à 3, le flocon est `Flocon::new(rayon, Parametres::lire(alpha, beta, gamma))` avec les trois textes
//! du scénario (`"1"`, `"0.45"`, `"0.1"` par exemple : la lecture exacte de `Parametres::lire`, comme `lancer_reiter`).
//!
//! **Le claquage (étape D3).** Le modèle `4 + η` (`η` de 0 à 3) est le claquage d'exposant `η` : `Claquage::new(u64::from(graine), demi_cote, η)` (la
//! graine est lue comme celle de la DLA : un `u32` entier, de 0 à `u32::MAX`), avec les **sites visés** et le **demi-côté** du tableau (le demi-côté est
//! `demi_cote_de_la_courbe(η, sites)` de [`crate::courbe`], la règle de l'étape D2 : 70, 52, 49 et 48). Le claquage coûte cher (le champ de Laplace est
//! relaxé après chaque site, `MODELE.md` étape D1) : **la taille de l'amas est bornée** par ce tableau, et aucun modèle de la page ne le dépasse (`η = 0`
//! n'a pas de champ et coûte presque rien ; `η = 3` vise 100 sites). Le temps natif d'une croissance entière est mesuré dans « Le coût » de `MODELE.md`
//! (étape D3). La **graine** compte pour ces modèles (comme pour la DLA) ; les flocons l'ignorent.
//!
//! # Le temps
//!
//! Une **unité de temps** est une particule pour la DLA (un appel de `Amas::essayer_ajouter`), un site de plus pour le claquage (un appel de
//! `Claquage::essayer_ajouter`), un pas pour un flocon (un appel de `Flocon::pas`) ou pour la neige (un appel de `Neige::pas`, étape G3). Le **temps** ([`givre_temps`]) est le nombre d'unités jouées depuis
//! le démarrage : pour la DLA et le claquage le nombre de particules ajoutées, c'est-à-dire le nombre de sites **moins un** (le germe n'est pas une
//! particule) ; pour un flocon `Flocon::temps()` ; pour la neige `Neige::temps()`. La **durée** ([`givre_duree`]) est le temps où s'arrête le scénario (dernière colonne du tableau) :
//! `givre_avancer` ne joue jamais au-delà. Une unité de claquage coûte la relaxation du champ (de 0,000 s à `η = 0` à quelques dixièmes de seconde au
//! pire à `η >= 1`, natif, `release` : voir `MODELE.md`) : **un appel de `givre_avancer` qui joue beaucoup d'unités peut durer, pour les modèles 4 à 7,
//! jusqu'au temps de la croissance entière** ; la page joue ces modèles unité par unité, en s'arrêtant quand une courte durée est écoulée.
//!
//! # L'état global
//!
//! Un seul modèle est en cours : un `static` protégé par un [`std::sync::Mutex`] (la cible WebAssembly n'a pas de fil
//! d'exécution ; en natif le verrou sérialise les appels des tests ; un verrou empoisonné par une panique se reprend par
//! `into_inner`). [`givre_demarrer`] et [`givre_neige_demarrer`] (étape G3) le (re)construisent **entièrement** : rien de l'état précédent ne survit (ni le temps, ni
//! l'image, ni l'empreinte, ni le tampon de l'image nette, ni, pour la neige, ses réglages).
//!
//! # Avant tout démarrage
//!
//! Tant que [`givre_demarrer`] n'a pas réussi une fois (ni [`givre_neige_demarrer`]), les fonctions qui rendent un entier rendent 0 (sauf [`givre_pixel`] et [`givre_nette_pixel`] :
//! `u32::MAX`, [`givre_nette_preparer`] : le code 2, et [`givre_neige_reglage`] et [`givre_neige_mesure`] : `u32::MAX`), [`givre_compacite`] rend `NaN` et [`givre_avancer`] ne fait rien (rend 0). Ce contrat n'est éprouvé que par
//! Node (`tests/etape_5/etat-wasm.mjs`, sur un module neuf) : en natif l'état global est déjà posé par les autres tests du
//! processus. Pour l'image nette il l'est aussi en natif : `tests/etape_v7_neuf.rs` est un fichier d'**un seul test**, donc un processus neuf (le code 2, le tampon vide) ; pour la neige de même : `tests/etape_g3_neuf.rs` (les lecteurs à `u32::MAX`, un démarrage refusé qui ne démarre rien).
//!
//! # L'image
//!
//! L'image de l'état courant est celle de l'aperçu de la bibliothèque, et la page la dessine telle quelle :
//! - DLA : `amas_colore(amas.sites(), COTE_APERCU)` de [`crate::dla`] (sa largeur et sa hauteur **changent** quand l'amas
//!   grandit : la fenêtre suit l'amas ; au temps 0, un site, l'image est de 512 × 512 pixels tous blancs) ;
//! - claquage (modèles 4 à 7) : `amas_colore(claquage.sites(), COTE_APERCU)`, **la même fonction que la DLA** (c'est aussi l'aperçu de `lancer` pour la
//!   directive `claquage`) : une image carrée dont le côté suit l'amas (au temps 0, un site : 512 × 512 pixels, le pixel du germe de la couleur du rang 0) ;
//! - flocon : `Flocon::apercu()`, l'image carrée de côté `2R + 1` (61, 81 ou 121 pixels ; la case `(q, r)` est le pixel
//!   `(q + R + floor(r / 2), r + R)`) ;
//! - neige (étape G3) : `Neige::apercu()`, l'image carrée de côté `2R + 1` (201 pixels pour le domaine de rayon 100), de la même façon (voir [`crate::neige`], « L'aperçu »).
//!
//! [`givre_largeur`] et [`givre_hauteur`] donnent ses dimensions, [`givre_pixel`] la couleur d'un pixel `(x, y)` (`x` vers la
//! droite, `y` vers le bas, `(0, 0)` en haut à gauche, ligne par ligne de haut en bas comme les octets de l'aperçu) **empaquetée
//! en `0x00RRGGBB`** : `(R << 16) | (V << 8) | B`, les trois octets de l'aperçu à cette position. `u32::MAX`
//! (`0xFFFFFFFF`, jamais une couleur : son octet de poids fort n'est pas nul) dit « hors de l'image » : `x >= largeur`, ou
//! `y >= hauteur`, ou rien de démarré.
//!
//! # L'image nette (étape V7)
//!
//! Quatre fonctions de plus donnent, pour les modèles **sur la grille hexagonale** (les flocons de Reiter, modèles 1, 2 et 3, et la neige, étape G3), une seconde image du modèle en cours : non plus « un pixel
//! par case » (voir « L'image ») mais **chaque case dessinée comme un hexagone** à sa vraie géométrie, avec une couleur continue tirée d'un champ du modèle, par [`crate::rendu_hex::rendre`] (étape G1 :
//! hexagones lisses, dégradé de bleus clair puis foncé, fond blanc, suréchantillonnage en entiers ; aucun pixel n'y est un agrandissement au plus proche voisin). Un modèle sur une grille carrée (la DLA
//! et le claquage, modèles 0 et 4 à 7) garde ses cases-carrés : pour lui l'image nette n'existe pas (code 1). **Rien de ce qui existe ne change** : [`givre_largeur`], [`givre_hauteur`] et [`givre_pixel`]
//! rendent toujours l'image d'un pixel par case de « L'image » (octet pour octet), l'état, le temps et l'empreinte ne bougent pas, et préparer une image nette ne touche à aucun d'eux. La neige
//! de Gravner et Griffeath (étape G3) est aussi sur la grille hexagonale : elle y est par son champ et son `plein` (voir « Le champ dessiné de la neige » dans « La neige »), sans nouvelle exportation.
//!
//! - [`givre_nette_preparer`]`(cote, sur)` **prépare** l'image nette du modèle en cours, à `cote × cote` pixels et `sur × sur` échantillons par pixel, la garde dans un **tampon** et rend un code (la
//!   table plus bas ; 0 : l'image est prête). C'est la seule des quatre qui calcule : elle coûte `(cote × sur)²` échantillons (voir « Les bornes »).
//! - [`givre_nette_largeur`] et [`givre_nette_hauteur`] rendent les dimensions du tampon (`cote` et `cote` : l'image est carrée) ; [`givre_nette_pixel`]`(x, y)` rend la couleur du pixel `(x, y)`
//!   **empaquetée en `0x00RRGGBB`, exactement comme [`givre_pixel`]** (`x` vers la droite, `y` vers le bas, `(0, 0)` en haut à gauche, les octets R, V, B de l'image), ou `u32::MAX` si `x >= largeur`,
//!   si `y >= hauteur` ou sans tampon. **Une lecture ne recalcule rien.** Sans tampon, la largeur et la hauteur valent 0.
//! - **Le tampon appartient à l'état qu'il montre** : [`givre_demarrer`] qui réussit l'efface, [`givre_avancer`] l'efface dès qu'il joue **au moins une unité** (un appel qui ne joue rien le laisse :
//!   scénario fini, `n = 0`), et une préparation qui échoue l'efface (avant de rendre son code). Une préparation qui réussit remplace le tampon précédent. Un `givre_demarrer` refusé (modèle
//!   inexistant) ne change rien, tampon compris. Jamais l'image d'un autre état.
//!
//! **Le champ dessiné** (les flocons de Reiter). C'est une valeur entière `>= 0` par case de l'hexagone, **dans l'ordre canonique de [`crate::reseau::Hexagone`]** (celui de `Flocon::masses`), tirée de la
//! masse d'eau de la case. `brut` désigne le champ `.0` d'un `Fixe` (un `i64`) et `UN = 2^32` :
//!
//! ```text
//! E      = γ.brut × durée        le brut de γ du flocon fois la durée du modèle (givre_duree()), en i64 saturé, au moins 1 et au plus 2^58
//! valeur = 0                     si la masse est inférieure à 1 (brut < UN) : le fond, la vapeur, les cases de bord
//! valeur = 3 × (brut − UN) + E   si la masse est au moins 1 (la case est gelée)
//! plein  = 4 × E
//! ```
//!
//! Ces trois nombres se calculent **sans jamais déborder** : en arithmétique saturée (`saturating_mul`, `saturating_add`) ou dans un type plus large ramené à `i64::MAX`. Avec les trois modèles de la table rien ne sature
//! (un `brut` reste sous `2^37`), mais un `*` nu paniquerait s'il débordait (les vérifications de débordement sont actives), et aucune de ces fonctions ne panique.
//!
//! `E` est ce que le germe gagne en tout pendant le scénario : une case gelée gagne exactement `γ` à chaque pas et rien d'autre, donc sa masse **suit son âge**, à ce qu'elle avait en trop en gelant près (une case
//! qui gèle peut dépasser 1 : dans la dendrite, quelques cases gelées tard atteignent 1,84 quand le germe finit à 1,8, et saturent comme lui au bleu foncé). Le dégradé montre le cristal qui s'épaissit. Avec `t = min(valeur,
//! plein) × 1024 / plein` (le dégradé de `rendu_hex`), une case **tout juste gelée** (`brut = UN`) vaut `E` : `t = 256`, le premier bleu `(198, 219, 239)` (un bord de cristal se voit sur le fond blanc) ;
//! une case qui a gagné `E` depuis qu'elle a gelé (le germe à la fin du scénario) vaut `4 E` : `t = 1024`, le bleu foncé `(8, 48, 107)` ; en général `t = 256 + 768 × (brut − UN) / E` (division
//! entière), plafonné à 1024. La masse d'une case non gelée ne compte pour rien : le fond reste blanc même pour `β = 0,45`. Pour la table des modèles, `E` vaut `214 748 365 × 400 = 85 899 346 000`
//! (modèle 1, plaque), `429 496 730 × 120 = 51 539 607 600` (modèle 2, étoile) et `4 294 967 × 800 = 3 435 973 600` (modèle 3, dendrite).
//!
//! **L'appel de la bibliothèque.** `rendre(R, &valeurs, &ReglagesImage { cote, rayon_vue: R, sur, plein })` avec `R = Flocon::rayon()` (30, 40 ou 60), `valeurs` le champ ci-dessus et `plein = 4 E` :
//! la vue est le carré de `2 R + 1` cases centré sur la case `(0, 0)`, qui contient tout l'hexagone du flocon et une marge ; l'image est les octets R, V, B rendus par `rendre`, ligne par ligne de haut
//! en bas (`3 × cote × cote`), que [`givre_nette_pixel`] relit.
//!
//! **Les bornes** (mesurées : `MODELE.md`, étape V7) : `cote` est **pair**, de 2 à [`NETTE_COTE_MAX`] (2048) ; `sur` va de 1 à [`NETTE_SUR_MAX`] (4) ; et `cote × sur` (le côté de la grille d'échantillons) ne
//! dépasse pas [`NETTE_GRILLE_MAX`] (4096), soit au plus 16 777 216 échantillons par préparation (de 0,24 à 0,30 s en natif sur le poste de l'écrivain au calme, jusqu'à 0,7 s quand il est chargé ; une image
//! d'animation de la page, 384 pixels à 2 × 2, en coûte 589 824 : 10 à 11 ms au calme, 27 ms chargé). Une image d'au plus 2048 × 2048 pixels pèse au plus 12 Mio. **Le module sous Node** (mesuré par le pilote sur
//! la VM de preuve, Linux, Node 20) : 26 ms pour préparer `(384, 2)` (28 ms avec la lecture pixel par pixel), 375 ms pour `(1024, 3)` (388 ms avec la lecture), 770 ms pour `(2048, 2)`, la borne.
//!
//! **Les codes** de [`givre_nette_preparer`] (les vérifications se font **dans cet ordre** : le premier refus rend son code) :
//!
//! | code | sens |
//! |---|---|
//! | 0 | l'image est prête (le tampon la garde) |
//! | 2 | rien n'est démarré (aucun tampon à effacer) |
//! | 1 | le modèle en cours n'est pas sur la grille hexagonale (0, 4, 5, 6 ou 7) |
//! | 3 | `cote` hors borne : impair, inférieur à 2 ou supérieur à [`NETTE_COTE_MAX`] |
//! | 4 | `sur` hors borne : 0 ou supérieur à [`NETTE_SUR_MAX`] |
//! | 5 | trop d'échantillons : `cote × sur` supérieur à [`NETTE_GRILLE_MAX`] |
//! | 6 | `rendre` a refusé (n'arrive jamais avec les bornes ci-dessus : le champ est valide, le rayon aussi) |
//!
//! # Les mesures
//!
//! - [`givre_sites`] : DLA et claquage, le nombre de sites de l'amas, **germe compris** (1 au démarrage) ; flocon, `Flocon::gelees()` ;
//! - [`givre_rayon`] : DLA, `Amas::rayon()` ; claquage, `Claquage::rayon()` ; flocon, `Flocon::rayon_cristal()` ;
//! - [`givre_compacite`] : flocon, `Flocon::compacite().vers_f64()` (1 pour un hexagone plein, et au temps 0 : une case gelée
//!   sur une case) ; DLA et claquage, `NaN` (sans objet : l'amas n'a pas de compacité).
//!
//! # L'empreinte
//!
//! [`givre_empreinte_mot`]`(i)` est le `i`-ième mot (de 0 à 7) de 32 bits, gros-boutiste, du SHA-256 de l'écriture canonique
//! de l'état courant : l'empreinte de `Amas::empreinte()` (DLA), de `Claquage::empreinte()` (claquage) ou de `Flocon::empreinte()` (flocon). Les huit mots, chacun
//! écrit sur 8 chiffres hexadécimaux minuscules et mis bout à bout dans l'ordre de 0 à 7, redonnent ces 64 caractères. Pour la
//! DLA de graine 1 jouée jusqu'au bout, c'est le contenu de `references/dla-graine-1.empreinte` ; pour la plaque jouée jusqu'au
//! bout, celui de `references/reiter-plaque.empreinte` : **l'empreinte d'une graine dans la page est celle du code natif.**
//! Chaque appel recalcule l'empreinte de l'état (huit appels = huit SHA-256 de tout l'état) : la page ne la lit pas à chaque
//! image.
//!
//! # La mesure de la corrélation (étape Z1)
//!
//! Quatre fonctions de plus, qui mesurent la dimension de l'amas de la DLA (et, depuis l'étape D3, du claquage) par la corrélation densité-densité
//! de [`crate::correlation`] et rendent la courbe `C(r)` pour la page. Elles ne paniquent jamais non plus.
//!
//! - [`givre_mesurer`] mesure **l'amas du moment** (jamais à chaque image : le coût est en `O(n²)`) et garde le résultat (la fenêtre, les
//!   `C(r)` et la dimension) : pour la DLA par [`crate::correlation::mesure`], **pour le claquage (modèles 4 à 7, étape D3) par
//!   [`crate::courbe::mesure_petit_amas`]** (la fenêtre des petits amas de l'étape D2 : des rayons de 2 à `max(4, étendue / 6)`, au lieu de 4 à
//!   `étendue / 8` pour la DLA). Elle rend le **nombre de points**
//!   de la courbe, c'est-à-dire le nombre de rayons de la fenêtre (`mesure.rayons.len()`), ou **0 pour « pas de mesure »** : avant tout
//!   démarrage, pour un flocon (modèles 1, 2 ou 3), pour un amas trop petit (la fenêtre est vide : pour la DLA, voir
//!   [`crate::correlation::fenetre`], une étendue de moins de 72, soit de 465 à 774 sites sur 40 graines mesurées ; pour le claquage, voir
//!   [`crate::courbe::fenetre_petit_amas`], une étendue de moins de 8), pour un amas de plus de
//!   [`MESURE_SITES_MAX`] sites, ou si la mesure rend `None`. Un appel qui rend 0 laisse « pas de mesure » (aucune mesure n'est
//!   gardée).
//! - [`givre_correlation_rayon`]`(i)` est le `i`-ième rayon de la fenêtre de la mesure gardée (`i` de 0 à `points - 1`, dans l'ordre
//!   croissant : 4, 5, 6, … pour la DLA ; 2, 3, 4, … pour le claquage) ; [`givre_correlation_valeur`]`(i)` est le `C(r)` du même rang, **bit pour bit**
//!   celui de [`crate::correlation::mesure`] (DLA) ou de [`crate::courbe::mesure_petit_amas`] (claquage) ; [`givre_dimension`] est la dimension de la
//!   mesure gardée, bit pour bit celle de `mesure` ou de `mesure_petit_amas`.
//!   Sans mesure gardée (y compris avant tout démarrage), ou pour `i >= points` : `givre_correlation_rayon` rend 0, `givre_correlation_valeur` et `givre_dimension`
//!   rendent `NaN`.
//! - **La mesure gardée appartient à l'amas qu'elle a mesuré** : [`givre_demarrer`] (qui reconstruit tout l'état) l'efface, et
//!   [`givre_avancer`] l'efface dès qu'il joue **au moins une unité** ; un `givre_avancer` qui ne joue rien (scénario fini, `n = 0`) ne
//!   l'efface pas.
//! - [`MESURE_SITES_MAX`] (10 000) est le plafond de taille d'amas de la mesure dans la page. La DLA de la vitrine s'arrête à 3000
//!   sites : le plafond n'est atteint par aucun modèle de la table ci-dessus, il garde la page vive si un modèle plus grand est ajouté.
//!   Le temps de la mesure en natif (profil `release`, une graine) est de 0,008 s à 2000 sites, 0,040 s à 5000, 0,17 s à 10 000, 0,69 s à
//!   20 000 et 1,67 s à 30 000 : il croît comme `n²`. Le module WebAssembly est plus lent d'un facteur qui n'a pas été mesuré (ni Node ni la
//!   cible `wasm32-unknown-unknown` n'étaient là) : 10 000 sites laisse de la marge sur la seconde et demie visée, 20 000 sites n'en laisse
//!   que si ce facteur est inférieur à 2 ; le plafond se relève sur une mesure du module.
//!
//! # La courbe D(η) à la demande (étape D3)
//!
//! Huit fonctions de plus calculent la courbe `D(η)` de [`crate::courbe`] (la dimension de petits amas de claquage pour `η = 0, 1, 2, 3`, moyennée sur
//! des graines) **un amas par appel**, de façon que la page ne se fige jamais : un amas de la taille permise coûte moins d'une seconde en natif (`release`,
//! mesuré : `MODELE.md`, étape D3), et la page rend la main entre deux appels. Elles ne prennent et ne rendent que des nombres, **ne paniquent jamais**,
//! et appellent les entrées `essayer_*` de [`crate::courbe`] : chaque échec devient un **code d'erreur** (la table plus bas).
//!
//! **Le travail.** [`givre_courbe_demarrer`]`(taille, graines)` démarre un travail : quatre points (`η = 0, 1, 2, 3`), chacun fait de `graines` amas de
//! `taille` sites (germe compris), de graines `1, 2, …, graines` (la graine de rang `k`, en partant de 1, est `k`) et de demi-côté
//! `essayer_demi_cote_de_la_courbe(η, taille)`. Les amas se font **dans cet ordre** : tous ceux de `η = 0` (graines `1` à `graines`), puis ceux de
//! `η = 1`, puis `η = 2`, puis `η = 3`. Le **total** de amas est `4 × graines` ; le `k`-ième amas (`k` de 0 à `total - 1`) est celui de `η = k / graines`
//! (division entière) et de graine `k % graines + 1`. [`givre_courbe_pas`] fait **un** amas, le suivant : [`essayer_dimension_d_une_graine`] avec ces
//! `η`, graine, `taille` et demi-côté, et garde sa dimension ; quand le dernier amas d'un `η` est fait, le point de cet `η` est la
//! [`moyenne_et_erreur_type`] des `graines` dimensions de cet `η`, dans l'ordre des graines : **bit pour bit** le `moyenne` et l'`erreur_type` de
//! `essayer_point_de_la_courbe(η, [1, 2, …, graines], taille, demi_cote)`. [`givre_courbe_fait`] est le nombre d'amas faits (0 au démarrage, `total` à
//! la fin) et [`givre_courbe_total`] le total. Un travail fini rend 0 à tout `givre_courbe_pas` de plus et ne change plus.
//!
//! **Les bornes** (la page borne la taille des amas) : [`COURBE_TAILLE_MAX`] (100) sites, de [`COURBE_GRAINES_MIN`] (2) à [`COURBE_GRAINES_MAX`] (8)
//! graines. `taille` doit être au moins 1 et au plus 100, `graines` au moins 2 et au plus 8 : sinon `givre_courbe_demarrer` rend un code d'erreur
//! (1 ou 2) et **ne démarre aucun travail**. Un amas de la taille maximale coûte, au pire de nos mesures, moins d'une seconde en natif (`MODELE.md`).
//!
//! **Un nouveau `givre_courbe_demarrer` efface l'ancien travail** (même s'il est refusé : il n'y a alors aucun travail, `fait` et `total` valent 0).
//! `givre_demarrer` (le modèle en cours) **ne touche pas** au travail de la courbe : les deux sont indépendants, et un `givre_courbe_pas` ne touche pas
//! au modèle en cours.
//!
//! **Les valeurs.** [`givre_courbe_moyenne`]`(eta)` et [`givre_courbe_erreur_type`]`(eta)` rendent la moyenne et l'erreur-type du point de `eta` (0 à 3) **une
//! fois ses `graines` amas faits**, bit pour bit ceux de la bibliothèque, et `NaN` avant (et sans travail, et pour `eta > 3`, jusqu'à `u32::MAX`).
//! [`givre_courbe_repere`]`(eta)` rend le repère publié de `eta` : `crate::courbe::publie(eta)` (`1,71` pour `η = 1`, `1,433` pour `η = 2`, `1,263` pour
//! `η = 3`, voir [`crate::courbe`] : le rappel de la DLA et Hastings 2001) et `NaN` quand il n'y en a pas (`η = 0`, `η >= 4`). **Il vient de la
//! bibliothèque : ni ce fichier ni la page n'écrivent une valeur publiée.** Il ne dépend d'aucun travail (la page le lit aussi pour le modèle en cours).
//!
//! **Les codes d'erreur.** [`givre_courbe_demarrer`] et [`givre_courbe_pas`] rendent un code ; [`givre_courbe_code`] rend **le dernier** (celui de
//! l'appel le plus récent de l'une ou l'autre ; 0 avant tout appel) :
//!
//! | code | sens | qui le rend |
//! |---|---|---|
//! | 0 | aucune erreur | démarrage accepté ; un amas fait ; travail fini |
//! | 1 | `taille` hors borne (0 ou plus de 100) | `givre_courbe_demarrer` |
//! | 2 | `graines` hors borne (moins de 2 ou plus de 8) | `givre_courbe_demarrer` |
//! | 3 | un amas a touché la frontière avant d'avoir `taille` sites ([`ErreurCourbe::FrontiereTouchee`]) | `givre_courbe_pas` |
//! | 4 | un amas n'a pas de dimension : fenêtre vide ou `C(r)` nul ([`ErreurCourbe::FenetreVide`]), par exemple `taille` de 1 ou 5 sites (mesuré : à 32 sites, la graine 1 de `η = 0` et la graine 5 de `η = 1` n'ont pas de dimension ; dès 34 sites les graines 1 à 8 des quatre `η` en ont toutes) | `givre_courbe_pas` |
//! | 5 | aucun travail : rien n'est démarré, ou le démarrage a été refusé | `givre_courbe_pas` |
//! | 6 | une autre erreur de la bibliothèque (un paramètre que ce module ne passe jamais : `η`, demi-côté, graines) | `givre_courbe_pas` |
//!
//! La **première** erreur d'un `givre_courbe_pas` (3, 4 ou 6) **arrête** le travail : il garde les amas déjà faits (les points des `η` finis restent lisibles ;
//! `fait` ne compte pas l'amas qui a échoué et ne bouge plus), n'avance plus, et tout `givre_courbe_pas` de plus rend le même code sans rien faire (`givre_courbe_code`
//! le redit). Un code 1, 2 ou 5 ne change aucun travail existant (il n'y en a pas). Un travail **fini** (`fait == total`) rend 0 à tout `givre_courbe_pas` de plus, et
//! `givre_courbe_code` vaut alors 0.
//!
//! # La neige (étape G3)
//!
//! Cinq fonctions de plus (`givre_neige_*`) font croître **la neige de Gravner et Griffeath** de [`crate::neige`] (étape G1 : sur la grille hexagonale, en virgule fixe, sans hasard ni bruit) **dans le même état global** que les autres modèles : une neige démarrée est « le modèle en cours », et
//! **toutes les fonctions qui existent déjà la jouent et la lisent** (le tableau plus bas) : la page n'a ni seconde boucle d'animation ni seconde image. **[`givre_demarrer`] ne change pas** : il n'a toujours que les modèles 0 à 7 (le modèle 8 et les suivants restent « inconnus » : ils rendent 0 et ne changent rien) ;
//! la neige se démarre par [`givre_neige_demarrer`], qui remplace l'état entier comme lui, et un [`givre_demarrer`] qui réussit remplace une neige entièrement. Une neige est réglée par **sept réglages** (`ρ β α θ κ μ γ`, voir [`crate::neige`], « Les réglages ») dont **quatre sont passés** (`ρ`, `β`, `κ`, `μ`) ;
//! les trois autres (`α`, `θ`, `γ`), le rayon du domaine et le nombre de pas viennent du **jeu nommé** qu'on démarre.
//!
//! **Ce que les fonctions qui existent rendent pour la neige.** Rien de ce qu'elles rendent pour les modèles 0 à 7 ne change.
//!
//! | fonction | pour la neige |
//! |---|---|
//! | [`givre_avancer`] | joue des pas de `Neige::pas`, en s'arrêtant au mur : voir « La fin et le mur » |
//! | [`givre_temps`] | `Neige::temps()` (en `u32`) : le nombre de pas joués |
//! | [`givre_duree`] | le nombre de pas du jeu démarré (`givre_neige_reglage(9)`) : le scénario s'arrête au plus tard là |
//! | [`givre_largeur`], [`givre_hauteur`], [`givre_pixel`] | l'image de `Neige::apercu()` : carrée, de côté `2R + 1` (201 pour `R = 100`) ; la page ne la lit que si la préparation de l'image nette est refusée |
//! | [`givre_sites`] | `Neige::cases_rattachees()` : le nombre de cases du cristal |
//! | [`givre_rayon`] | `Neige::rayon_cristal()` |
//! | [`givre_compacite`] | `NaN` (sans objet : la compacité de la neige est une mesure de [`givre_neige_mesure`]) |
//! | [`givre_empreinte_mot`] | les mots du SHA-256 de `Neige::ecriture_canonique()`, donc de `Neige::empreinte()` : **l'empreinte d'un jeu nommé joué jusqu'au bout est celle de [`crate::balayage::balayage`]** pour la même valeur |
//! | [`givre_mesurer`], [`givre_correlation_rayon`], [`givre_correlation_valeur`], [`givre_dimension`] | pas de mesure de corrélation : [`givre_mesurer`] rend 0, comme pour un flocon |
//! | [`givre_nette_preparer`] et les trois autres | l'image nette : voir « Le champ dessiné de la neige » ; jamais le code 1 (la neige est sur la grille hexagonale) |
//!
//! **Les réglages, en millionièmes.** Un réglage passe la frontière **en entier, en millionièmes** (« ppm ») : `ρ = 0,40` s'écrit 400 000, `β = 1,09` s'écrit 1 090 000, `γ = 0,00005` s'écrit 50. **Vers `Fixe`** : `Fixe::rapport(ppm, 1_000_000)`, c'est-à-dire
//! `brut = floor((2 · ppm · 2^32 + 10^6) / (2 · 10^6))`, en `i128` : au plus proche (il n'y a jamais de demi-cas : `ppm · 2^32 / 10^6 = ppm · 2^26 / 5^6` a pour dénominateur `5^6`, impair). C'est exactement ce que `Parametres::lire` fait d'un décimal écrit avec six chiffres au plus après la virgule
//! (`lire_decimal("0.00005")` et `Fixe::rapport(50, 1_000_000)` ont le même `brut`, 214 748). **Depuis `Fixe`** (ce que les fonctions rendent) : `ppm = floor((2_000_000 · brut + 2^32) / 2^33)` en `i128`, comme la valeur du CSV de [`crate::balayage::ligne_csv`] ; l'aller-retour redonne tout entier de 0 à 8 000 000. **Aucune panique** : `Fixe::rapport` ne panique que si le résultat sort de `[-2^31 ; 2^31[`, et le plus grand `u32`
//! donne 4 294,967295 (la conversion se fait **avant** toute vérification de plage : c'est `Parametres::verifier` qui refuse ensuite) ; dans l'autre sens un réglage d'une neige ou d'un jeu est toujours dans sa plage (de 0 à 8), donc le résultat tient dans un `u32` (au plus 8 000 000).
//!
//! **Les jeux nommés** (une seule source : [`crate::balayage`], jamais recopiés ici ni dans la page). Il y en a [`NEIGE_JEUX`] = **19**, numérotés de 0 à 18 :
//!
//! | jeu | figure de l'article | quoi |
//! |---|---|---|
//! | 0 à 5 | 9 | [`crate::balayage::jeu_rho`] : `ρ = 0,40 ; 0,41 ; 0,42 ; 0,44 ; 0,46 ; 0,50` (rang 1 à 6) |
//! | 6 à 11 | 10 | [`crate::balayage::jeu_beta`] : `β = 1,9 ; 2,2 ; 2,4 ; 2,6 ; 2,7 ; 2,8` |
//! | 12 à 17 | 12 | [`crate::balayage::jeu_mu`] : `μ = 0,04 ; 0,05 ; 0,06 ; 0,07 ; 0,08 ; 0,09` |
//! | 18 | 13 | le cristal du milieu de la figure 13 (« a stellar crystal with plate ends ») : `Parametres::lire("0.36", "1.09", "0.01", "0.0745", "0.0001", "0.14", "0.00001")` (dans l'ordre `ρ β α θ κ μ γ`) |
//!
//! Le jeu `j` de 0 à 17 a pour réglages `reglage.fixer(base, valeurs[j % 6])` du `JeuDeBalayage` de sa figure (le réglage balayé prend la valeur de son rang), pour rayon de domaine `JeuDeBalayage::rayon` (100) et pour nombre de pas `JeuDeBalayage::pas`
//! (2000 pour la figure 9, 1300 pour la figure 10, 2000 pour la figure 12) ; le jeu 18 a le rayon 100 et [`NEIGE_PAS_FIGURE_13`] = 3000 pas (un choix à nous : la figure 13 demande plus de 100 000 pas, voir `MODELE.md`). **Lu ou déduit** : les réglages des jeux 0 à 5 et 18 sont lus (le jeu 18 : le texte de l'article et la ligne « 13m » de son annexe disent les mêmes sept nombres ; les jeux 0 à 5 : les légendes de la figure 9 donnent `ρ`, la ligne « 9 » de l'annexe
//! les six autres, **sans sa colonne `ρ`**, et la ligne « 15tr » redonne les sept avec `ρ = .5` : rien n'est à deviner) ;
//! ceux des jeux 6 à 17 sont **déduits** (dans les lignes « 10 » et « 12 » de l'annexe la colonne du réglage qui varie est absente et aucune autre ligne ne les recoupe en entier : voir `references/etape-g2-valeurs.md`, V-G2-02 et V-G2-03).
//!
//! [`givre_neige_jeux`]`()` rend [`NEIGE_JEUX`]. [`givre_neige_jeu`]`(jeu, champ)` lit un champ du jeu `jeu` (de 0 à 18) **sans toucher à l'état** et sans rien démarrer :
//!
//! | `champ` | valeur |
//! |---|---|
//! | 0 | le numéro de la figure : 9, 10, 12 ou 13 |
//! | 1 | le rang du cristal dans sa figure, à partir de 1 (1 à 6 ; 1 pour la figure 13) |
//! | 2 | le nombre de cristaux de sa figure (6, 6, 6 ou 1) |
//! | 3 | le réglage qui varie dans la figure, par son numéro de réglage ci-dessous (0 pour `ρ`, 1 pour `β`, 5 pour `μ`) ; 7 pour la figure 13 (aucun) |
//! | 4 à 10 | `ρ`, `β`, `α`, `θ`, `κ`, `μ`, `γ` du jeu, en millionièmes |
//! | 11 | le rayon du domaine (100) |
//! | 12 | le nombre de pas |
//! | 13 | 0 si les réglages sont lus, 1 s'ils sont déduits (jeux 6 à 17) |
//!
//! Hors de portée (`jeu >= 19` ou `champ >= 14`, `u32::MAX` compris) : `u32::MAX` (aucune valeur du tableau n'est `u32::MAX`). **Les numéros de réglage** : 0 `ρ`, 1 `β`, 2 `α`, 3 `θ`, 4 `κ`, 5 `μ`, 6 `γ` (l'ordre de [`crate::neige::Parametres`]).
//!
//! **Démarrer.** [`givre_neige_demarrer`]`(jeu, rho, beta, kappa, mu)` démarre une neige : les réglages du jeu `jeu`, **sauf** `ρ`, `β`, `κ` et `μ`, qui prennent les quatre valeurs passées (en millionièmes, **dans l'ordre `ρ β κ μ`** : celui de [`crate::neige::Parametres`]) ; `Neige::essayer_new(rayon du jeu, réglages)` ; la durée est le nombre de pas du jeu.
//! L'état est remplacé **entièrement** (le temps 0, l'image, l'empreinte, le tampon de l'image nette, la mesure gardée de corrélation : après une DLA ou un claquage mesurés, [`givre_correlation_rayon`] rend 0 et [`givre_dimension`] `NaN` dès le démarrage de la neige, sans attendre un [`givre_mesurer`]).
//! **L'état n'est touché qu'une fois tout vérifié** (le jeu, les réglages, la neige construite) : c'est ce qui fait qu'un refus ne change rien.
//! Elle rend un code, **dans cet ordre** : 0 (démarrée), 1 (`jeu >= 19`), puis, par `Parametres::verifier` des réglages ainsi formés (le **premier** réglage hors plage dans l'ordre `ρ β α θ κ μ γ`) : 2 (`ρ` hors de `[0 ; 1]`, c'est-à-dire plus de 1 000 000), 3 (`β` hors de `[1 ; 8]`, c'est-à-dire moins de 1 000 000 ou plus de 8 000 000),
//! 4 (`κ` hors de `[0 ; 1]`), 5 (`μ` hors de `[0 ; 1]`), et 6 (tout autre refus : un autre nom rendu par `Parametres::verifier`, ou un `Err` de `Neige::essayer_new` ; inatteignable, `α`, `θ` et `γ` viennent d'un jeu valide et le rayon vaut 100). Un `ρ` fautif et un `κ` fautif donnent donc 2, un `κ` et un `μ` fautifs donnent 4. **Un refus ne change rien** (ni l'état, ni le tampon de l'image nette : comme un `givre_demarrer` refusé) et ne panique jamais, pour aucun `u32` (`u32::MAX` compris). Les quatre valeurs
//! sont **indépendantes du jeu** : `givre_neige_demarrer(j, ρ_j, β_j, κ_j, μ_j)` où ces quatre sont les champs 4, 5, 8 et 9 du jeu `j` redonne le jeu nommé tel quel (même état, même empreinte à chaque pas) ; le jeu ne fixe alors que `α`, `θ`, `γ`, le rayon et les pas.
//!
//! **La fin et le mur.** Le mur du domaine réfléchit (étape G1) : un cristal qui l'atteint n'est plus celui de l'article. Le scénario d'une neige s'arrête donc **au premier de deux événements** : la durée (le nombre de pas du jeu) ou **le mur**. La limite du mur est `limite = R · NEIGE_MUR_POURCENT / 100` (division entière ; `R` est le rayon du domaine : 65 pour 100)
//! avec [`NEIGE_MUR_POURCENT`] = 65 : la neige est **au mur** quand `Neige::rayon_cristal() >= limite`. [`givre_avancer`]`(n)` d'une neige joue `Neige::pas` au plus `min(n, durée − temps)` fois, en **vérifiant avant chaque pas** si la neige est au mur : si oui il s'arrête sans jouer ce pas, et rend le nombre de pas joués (0 si le scénario est fini :
//! durée atteinte ou au mur). Comme pour les autres modèles, un appel de `n` pas puis un de `m` donne exactement l'état d'un appel de `n + m` (la vérification se fait pas à pas : le découpage ne change rien), et un appel qui joue au moins un pas efface la mesure gardée, le tampon de l'image nette et rafraîchit l'image d'un pixel par case.
//! Mesuré (`MODELE.md`, étape G3) : les dix-huit cristaux des jeux 0 à 17 ont à la fin de leurs pas un rayon de 25 à 62 : **aucun n'est au mur** (ils jouent tous leurs pas) ; poursuivis au-delà de leurs pas, les trois plus rapides (les jeux 5, 6 et 12) ont exactement les mêmes cases rattachées que dans un
//! domaine de rayon 140, au même nombre de pas, **jusqu'au rayon 65** (le jeu 5 en diffère dès 70, le jeu 12 dès 75, le jeu 6 dès 85) ; sur trente tirages de réglages libres, six atteignent le rayon 60 et ont tous les mêmes cases jusqu'à 80. **La limite du mur n'est donc pas une garantie
//! de fidélité à l'article** : un arrêt, pas un certificat. Les dix-neuf jeux ont tous `R = 100` : la division entière de la limite (`R · 65 / 100`, par défaut) n'est donc éprouvée que là où elle tombe juste.
//! **Écriture** : la règle est « avant chaque pas » ; une boucle d'un pas à la fois (`Neige::rayon_cristal()`, puis `Neige::pas()`) la suit mot pour mot. Comme le rayon croît d'au plus 1 par pas, on peut jouer d'un coup `limite − rayon` pas sans revérifier (même état, moins de
//! parcours de `rayon_cristal`, qui lit toutes les cases) : **ce lot ne doit jamais être de 0 pas** (un `>` pour le `>=` de la règle : `limite − rayon = 0`, la boucle ne finit pas et le test non plus).
//!
//! **Lire.** [`givre_neige_reglage`]`(champ)` lit la neige **en cours** (démarrée, c'est-à-dire que le modèle en cours est une neige) : `champ` de 0 à 6 : `ρ`, `β`, `α`, `θ`, `κ`, `μ`, `γ` de la neige, en millionièmes (les quatre passés, et les trois du jeu) ; 7 : le numéro du jeu démarré ; 8 : le rayon du domaine ;
//! 9 : le nombre de pas (la durée). [`givre_neige_mesure`]`(champ)` mesure la neige en cours **à l'instant** (elle ne garde rien et ne change rien ; chaque appel recalcule : la page ne l'appelle pas à chaque image) : `champ` de 0 à 5 : les six mesures de [`crate::balayage::mesurer`] dans l'ordre de [`crate::balayage::Mesures`]
//! (`rattachees`, `rayon`, `compacite`, `pointes`, `aiguilles`, `hexagonalite`) ; 6 : **l'étiquette** de [`crate::balayage::classer`] de ces mesures, par son numéro (0 aucune, 1 plaque, 2 secteurs, 3 dendrite, 4 fougère : l'ordre de [`crate::balayage::Forme`] ; **c'est notre classement, pas celui de l'article** : `MODELE.md`, étape G2) ; 7 : 1 si la neige est au mur, 0 sinon.
//! Sans neige en cours (rien de démarré, ou le modèle en cours est un autre), ou pour un `champ` hors de portée : `u32::MAX` (aucune valeur du tableau n'est `u32::MAX`).
//!
//! **Le champ dessiné de la neige** (pour [`givre_nette_preparer`]). C'est `Neige::valeurs_du_cristal()` (étape G1 : le `brut` de la masse cristalline `c` si la case est rattachée, 0 sinon, **dans l'ordre canonique** de [`crate::reseau::Hexagone`]), `plein = `[`NEIGE_PLEIN`]` = 6 442 450 944` (1,5 : la masse cristalline `c`
//! qui donne le bleu le plus foncé, comme les images de G1 et G2) et **la vue de rayon** `rayon_vue = R · NEIGE_MUR_POURCENT / 100 + NEIGE_MARGE_VUE` (69 pour `R = 100` : la vue est **fixe**, elle ne suit pas le cristal : une neige arrêtée au mur y tient entière ; [`NEIGE_MARGE_VUE`] = 4) :
//! `rendre(R, &valeurs, &ReglagesImage { cote, rayon_vue, sur, plein })` avec `R` le rayon du domaine. Les codes et les bornes de [`givre_nette_preparer`] sont ceux de « L'image nette » (le modèle n'est jamais refusé par le code 1).
//!
//! **Le coût** (mesuré, natif : `MODELE.md`, étape G3). un pas à `R = 100` coûte de l'ordre de 0,9 ms (1000 pas : 0,87 à 0,89 s ; 1,7 à 1,8 ms quand les pas sont demandés un par un, parce que chaque appel qui joue rafraîchit l'image d'un pixel par case) ; l'image nette `(384, 2)` coûte 15 ms à préparer et 1,5 ms à lire, `(1024, 3)` 214 ms et 10 ms ; les huit mots de l'empreinte 25 à 28 ms (huit SHA-256 de 750 Ko) ; les huit champs de `givre_neige_mesure` 1 à 3 ms (chaque appel recalcule les mesures) ; un démarrage 3 ms. Le module sous Node coûte de l'ordre du double du natif (V7) : **non mesuré pour la neige** (le pilote le mesure). Un cristal de 2000 pas coûte donc de l'ordre de 1,8 s de calcul natif.
//!
//! **Ce que la neige de la vitrine n'est pas.** Ce n'est pas la neige de l'article à sa taille (400 à 600 cases de diamètre, 10 000 à 100 000 pas) : des stades précoces (25 à 62 de rayon) ; le mur est à nous ; ni le bruit `σ`, ni la 3D, ni la forme selon la température ; l'étiquette est notre classement. Aucun nombre publié n'est testé (l'article ne donne que des images et des réglages).
//!
//! # Hors de portée
//!
//! [`givre_pixel`] rend `u32::MAX` hors de l'image ; [`givre_empreinte_mot`] rend 0 pour `i >= 8` ; [`givre_demarrer`] rend 0
//! pour un modèle qui n'existe pas (et ne change rien : tout modèle au-delà de 7) ; [`givre_avancer`] ne joue rien au-delà de la durée ; les
//! fonctions de la courbe sont dites plus haut (`NaN` hors de `η` de 0 à 3, code 5 sans travail) ; [`givre_nette_pixel`] rend `u32::MAX` hors du tampon, et
//! [`givre_nette_preparer`] rend un code (jamais une panique) pour un `cote` ou un `sur` absurde, `u32::MAX` compris ; les cinq fonctions de la neige rendent un code ou `u32::MAX` (jamais une panique) pour un
//! jeu, un réglage ou un champ absurde, `u32::MAX` compris (voir « La neige »).
//!
//! # La page
//!
//! `docs/index.html`, `docs/app.js` et `docs/lecture.js` font tourner ce module dans un navigateur ; leurs règles sont écrites
//! en tête de `tests/etape_5_page.rs` et, pour l'image nette (le canvas des modèles sur la grille hexagonale, qui n'a **pas** `image-rendering: pixelated`, et celui des modèles carrés, qui
//! l'a), de `tests/etape_v7_page.rs` et, pour la neige (le panneau, les quatre curseurs, le menu des jeux nommés, les textes), de `tests/etape_g3_page.rs`. `lecture.js` (l'empreinte, l'image, l'image nette et la graine que la page lit, sans DOM) est éprouvé
//! sous Node contre le code natif ; le reste de la page ne l'est que par sa forme : voir « Ce que cela n'établit pas » dans
//! `MODELE.md`.
//!
//! # Un conseil d'écriture (pas une règle : seule la description ci-dessus est imposée)
//!
//! L'état est un `enum` (un `Amas` ou un `Flocon`) avec la durée du scénario ; l'image se garde en mémoire et se refait quand
//! l'état a changé (après [`givre_avancer`] et [`givre_demarrer`]) : la page lit `largeur × hauteur` pixels une fois par
//! image, et `amas_colore` coûte à chaque appel une image entière. **Garder l'image est en pratique obligatoire** : sans cela
//! chaque `givre_pixel` refait l'image entière, et les tests, qui lisent des centaines de milliers de pixels, ne finissent plus
//! en dix minutes (mesuré sur le prototype sans cache : plus de 600 s, contre 4 s avec). Une image gardée d'avant un `avancer`
//! est la faute typique : les tests lisent l'image après chaque morceau.
//!
//! Pour l'image nette : le tampon est une image de plus (un champ `Option` de l'état, ou un second `static`), remise à « rien » aux trois endroits que « L'image nette » dit (un `givre_demarrer` qui réussit, un
//! `givre_avancer` qui joue, le début d'une préparation une fois l'état trouvé) et **nulle part ailleurs** (ni `givre_mesurer` ni une lecture ne l'effacent) ; une préparation recalcule toujours (jamais « le tampon a
//! déjà ce côté, je le rends ») ; `cote × sur` se compare à la borne après les vérifications de `cote` et de `sur` (ou en `u64`) : `u32::MAX × u32::MAX` déborde un `u32`.
//!
//! Pour la neige : une variante de plus de l'`enum` (la neige et le numéro de son jeu), donc un bras de plus dans chaque `match` sur le modèle (l'image, le temps, `givre_avancer`, les sites, le rayon, la compacité, l'empreinte, `givre_mesurer`, la préparation de l'image nette) ; rien d'autre ne change dans ces fonctions
//! (`givre_mesurer` rend 0 pour une neige et n'efface pas le tampon ; le tampon et l'image d'un pixel par case se refont aux mêmes endroits que pour un flocon). Deux `Parametres` existent (celui du flocon, celui de la neige) : en renommer un à l'importation. Clippy refuse un type écrit trop long
//! (`type_complexity` : un `match` plutôt qu'un tableau de fonctions) et une comparaison de bornes à la main (`manual_range_contains`).

use std::sync::{Mutex, MutexGuard};

use crate::claquage::Claquage;
use crate::courbe::ErreurCourbe;
use crate::dla::{Amas, COTE_APERCU, amas_colore};
use crate::empreinte::sha256;
use crate::flocon::{Flocon, Parametres};
use crate::math::Fixe;
use crate::rendu_hex::{ReglagesImage, rendre};

/// Une image d'aperçu gardée en mémoire : ses dimensions et ses octets R, V, B, ligne par ligne de haut en bas.
struct Image {
    largeur: u32,
    hauteur: u32,
    rvb: Vec<u8>,
}

/// Le modèle en cours : l'amas de DLA, le flocon de Reiter ou l'amas de claquage.
enum Modele {
    Dla(Amas),
    Flocon(Flocon),
    Claquage(Claquage),
}

/// L'état global : le modèle, la durée de son scénario, l'image de l'état courant (refaite quand l'état change), la mesure
/// gardée de l'amas du moment (effacée dès que l'amas change) et le tampon de l'image nette (effacé dès que l'état change : voir « L'image nette »).
struct Etat {
    modele: Modele,
    duree: u32,
    image: Image,
    mesure: Option<crate::correlation::Mesure>,
    nette: Option<Image>,
}

/// Le plus grand amas que [`givre_mesurer`] mesure : 10 000 sites (voir « La mesure de la corrélation »).
pub const MESURE_SITES_MAX: usize = 10_000;

/// Les sites (germe compris) que les modèles 4 à 7 (le claquage `η = 0` à `3`) font croître : 400, 250, 150 et 100 (voir « Les huit modèles »). La
/// `duree` d'un modèle est ce nombre moins un.
pub const CLAQUAGE_SITES: [u32; 4] = [400, 250, 150, 100];

/// Les demi-côtés des modèles 4 à 7 : `demi_cote_de_la_courbe(η, CLAQUAGE_SITES[η])`, soit 70, 52, 49 et 48 (voir « Les huit modèles »).
pub const CLAQUAGE_DEMI_COTE: [u32; 4] = [70, 52, 49, 48];

/// La plus grande `taille` (sites par amas, germe compris) d'un travail de la courbe : 100 (voir « La courbe D(η) à la demande »).
pub const COURBE_TAILLE_MAX: u32 = 100;

/// Le plus petit nombre de graines d'un travail de la courbe : 2 (une erreur-type n'existe pas pour une seule).
pub const COURBE_GRAINES_MIN: u32 = 2;

/// Le plus grand nombre de graines d'un travail de la courbe : 8.
pub const COURBE_GRAINES_MAX: u32 = 8;

/// Le plus grand `cote` d'une image nette : 2048 pixels (voir « L'image nette »).
pub const NETTE_COTE_MAX: u32 = 2048;

/// Le plus grand `sur` d'une image nette : 4 (soit 16 échantillons par pixel).
pub const NETTE_SUR_MAX: u32 = 4;

/// Le plus grand côté de la grille d'échantillons d'une image nette, `cote × sur` : 4096 (soit 16 777 216 échantillons).
pub const NETTE_GRILLE_MAX: u32 = 4096;

/// Le nombre de jeux de réglages nommés de la neige : 19, de 0 à 18 (les trois figures 9, 10 et 12 à six cristaux, puis le cristal du milieu de la figure 13 ; voir « La neige »).
pub const NEIGE_JEUX: u32 = 19;

/// Le nombre de pas du jeu 18 (le cristal du milieu de la figure 13) : 3000 (voir « La neige »).
pub const NEIGE_PAS_FIGURE_13: u32 = 3000;

/// La limite du mur, en pourcentage du rayon du domaine : une neige est au mur quand le rayon de son cristal atteint `R · 65 / 100` (voir « La fin et le mur »).
pub const NEIGE_MUR_POURCENT: u32 = 65;

/// La marge de la vue de l'image nette de la neige, en cases, au-delà de la limite du mur : 4 (voir « Le champ dessiné de la neige »).
pub const NEIGE_MARGE_VUE: u32 = 4;

/// La masse cristalline qui donne le bleu le plus foncé de l'image nette de la neige : 1,5, soit `6 442 450 944` en `brut` (voir « Le champ dessiné de la neige »).
pub const NEIGE_PLEIN: i64 = 6_442_450_944;

/// Le seul état de la vitrine ; `None` avant tout démarrage réussi.
static ETAT: Mutex<Option<Etat>> = Mutex::new(None);

/// Le travail de la courbe `D(η)` : ses paramètres, son avancement et les dimensions déjà obtenues, dans l'ordre des amas.
struct TravailCourbe {
    /// Le nombre de sites (germe compris) de chaque amas.
    taille: usize,
    /// Le nombre de graines par `η` (de 2 à 8).
    graines: u32,
    /// Le nombre d'amas faits (0 au démarrage, `total` à la fin).
    fait: u32,
    /// Le nombre total d'amas : `4 × graines`.
    total: u32,
    /// Le code d'arrêt (3, 4 ou 6) si un amas a échoué ; le travail n'avance alors plus.
    arret: Option<u32>,
    /// Les dimensions obtenues, dans l'ordre des amas (donc des `η`, puis des graines).
    dimensions: Vec<f64>,
}

/// L'état de la courbe : le travail en cours (aucun après un démarrage refusé ou avant tout appel) et le dernier code rendu par
/// `givre_courbe_demarrer` ou `givre_courbe_pas`. Indépendant de [`ETAT`] : `givre_demarrer` n'y touche pas.
struct EtatCourbe {
    travail: Option<TravailCourbe>,
    code: u32,
}

/// L'état de la courbe ; `None` avant tout démarrage de la courbe (un travail refusé laisse `travail` à `None`).
static COURBE: Mutex<EtatCourbe> = Mutex::new(EtatCourbe { travail: None, code: 0 });

/// Prend le verrou de l'état ; un verrou empoisonné par une panique se reprend par `into_inner` (jamais de panique ici).
fn verrouiller() -> MutexGuard<'static, Option<Etat>> {
    ETAT.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Prend le verrou de l'état de la courbe ; un verrou empoisonné par une panique se reprend par `into_inner`.
fn verrouiller_courbe() -> MutexGuard<'static, EtatCourbe> {
    COURBE.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// L'image de l'aperçu d'un modèle : `amas_colore` pour la DLA et le claquage, `Flocon::apercu` pour un flocon.
fn image_de(modele: &Modele) -> Image {
    match modele {
        Modele::Dla(amas) => {
            let (largeur, hauteur, rvb) = amas_colore(amas.sites(), COTE_APERCU);
            Image { largeur, hauteur, rvb }
        }
        Modele::Claquage(claquage) => {
            let (largeur, hauteur, rvb) = amas_colore(claquage.sites(), COTE_APERCU);
            Image { largeur, hauteur, rvb }
        }
        Modele::Flocon(flocon) => {
            let cote = 2 * flocon.rayon() + 1;
            Image { largeur: cote, hauteur: cote, rvb: flocon.apercu() }
        }
    }
}

/// Le temps du modèle en cours (voir « Le temps ») : les sites moins le germe pour la DLA et le claquage, `Flocon::temps` sinon.
fn temps_de(modele: &Modele) -> u32 {
    match modele {
        Modele::Dla(amas) => amas.sites().len().saturating_sub(1) as u32,
        Modele::Claquage(claquage) => claquage.sites().len().saturating_sub(1) as u32,
        Modele::Flocon(flocon) => u32::try_from(flocon.temps()).unwrap_or(u32::MAX),
    }
}

/// Le modèle de claquage `4 + η` d'exposant `η` (0 à 3) pour `graine`, avec son demi-côté et sa durée (voir « Les huit modèles ») ;
/// `None` si `modele` n'est pas de 4 à 7 (le calcul est alors refusé sans panique).
fn claquage_de(modele: u32, graine: u32) -> Option<(Modele, u32)> {
    let eta = modele.checked_sub(4)?;
    let &demi_cote = CLAQUAGE_DEMI_COTE.get(eta as usize)?;
    let &sites = CLAQUAGE_SITES.get(eta as usize)?;
    Some((Modele::Claquage(Claquage::new(u64::from(graine), demi_cote, eta)), sites.saturating_sub(1)))
}

/// Construit un flocon d'après les trois textes de paramètres du scénario ; `None` s'ils ne se lisent pas (ils se lisent).
fn flocon_de(rayon: u32, alpha: &str, beta: &str, gamma: &str) -> Option<Flocon> {
    let parametres = Parametres::lire(alpha, beta, gamma)?;
    Some(Flocon::new(rayon, parametres))
}

/// (Re)construit le modèle `modele` (0 à 7, voir « Les huit modèles ») au temps 0 et rend 1 ; `graine` n'est lue que par la
/// DLA (`Amas::new(u64::from(graine), 200)`) et le claquage (`Claquage::new(u64::from(graine), demi_cote, η)`) : un `u32` entier, de 0 à
/// `u32::MAX`, sans troncature ni signe ; elle est ignorée par les flocons, qui n'ont pas de hasard (la même graine n'importe laquelle
/// donne le même flocon).
///
/// Remplace entièrement l'état précédent : rappelée après des pas, elle repart de zéro. Elle ne touche pas au travail de la courbe. Pour
/// un `modele` qui n'est pas de 0 à 7, ne change rien (l'état précédent reste en place, ou l'absence d'état) et rend 0.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_demarrer(modele: u32, graine: u32) -> u32 {
    // Les réglages sont ceux des scénarios d'exemple (voir « Les quatre modèles ») : ils sont écrits ici, sans fichier.
    let construit = match modele {
        0 => Some((Modele::Dla(Amas::new(u64::from(graine), 200)), 2999)),
        1 => flocon_de(30, "1", "0", "0.05").map(|flocon| (Modele::Flocon(flocon), 400)),
        2 => flocon_de(40, "1", "0.45", "0.1").map(|flocon| (Modele::Flocon(flocon), 120)),
        3 => flocon_de(60, "1", "0.4", "0.001").map(|flocon| (Modele::Flocon(flocon), 800)),
        4..=7 => claquage_de(modele, graine),
        _ => None,
    };
    let Some((modele, duree)) = construit else { return 0 };
    let image = image_de(&modele);
    let mut garde = verrouiller();
    *garde = Some(Etat { modele, duree, image, mesure: None, nette: None });
    1
}

/// Joue `min(n, duree - temps)` unités de temps (voir « Le temps ») et rend ce nombre : `n` si le scénario n'est pas fini, moins
/// (jusqu'à 0) s'il s'arrête avant, 0 pour `n = 0`. Un appel de `n` unités puis un de `m` donne exactement l'état d'un appel de
/// `n + m` (la dynamique ne dépend pas du découpage). Rend 0 et ne fait rien avant tout démarrage.
///
/// Pour la DLA, si une particule collait hors du réseau (un amas de 3000 sites tient largement dans le carré de côté 401 : cela
/// n'arrive pas en pratique), `givre_avancer` s'arrête là, sans panique, et rend ce qu'elle a joué ; de même pour le claquage si l'amas touchait la
/// frontière avant sa durée (aucun amas mesuré ne l'a fait, voir `MODELE.md`). Pour le claquage, chaque unité coûte la relaxation du champ : un appel de
/// beaucoup d'unités peut durer, au pire, le temps de la croissance entière (voir « Le temps »).
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_avancer(n: u32) -> u32 {
    let mut garde = verrouiller();
    let Some(etat) = garde.as_mut() else { return 0 };
    let reste = etat.duree.saturating_sub(temps_de(&etat.modele));
    let a_jouer = n.min(reste);
    if a_jouer == 0 {
        return 0;
    }
    let jouees = match &mut etat.modele {
        Modele::Dla(amas) => {
            let mut jouees = 0;
            while jouees < a_jouer {
                if amas.essayer_ajouter().is_none() {
                    break;
                }
                jouees += 1;
            }
            jouees
        }
        Modele::Claquage(claquage) => {
            let mut jouees = 0;
            while jouees < a_jouer {
                if claquage.essayer_ajouter().is_none() {
                    break;
                }
                jouees += 1;
            }
            jouees
        }
        Modele::Flocon(flocon) => {
            flocon.avancer(u64::from(a_jouer));
            a_jouer
        }
    };
    if jouees > 0 {
        etat.mesure = None;
        etat.nette = None;
        etat.image = image_de(&etat.modele);
    }
    jouees
}

/// Le temps : les unités jouées depuis le démarrage (voir « Le temps »), de 0 à [`givre_duree`] ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_temps() -> u32 {
    let garde = verrouiller();
    garde.as_ref().map_or(0, |etat| temps_de(&etat.modele))
}

/// La durée du scénario du modèle en cours : 2999, 400, 120, 800, 399, 249, 149 ou 99 (voir « Les huit modèles ») ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_duree() -> u32 {
    let garde = verrouiller();
    garde.as_ref().map_or(0, |etat| etat.duree)
}

/// La largeur, en pixels, de l'image de l'état courant (voir « L'image ») ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_largeur() -> u32 {
    let garde = verrouiller();
    garde.as_ref().map_or(0, |etat| etat.image.largeur)
}

/// La hauteur, en pixels, de l'image de l'état courant (voir « L'image ») ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_hauteur() -> u32 {
    let garde = verrouiller();
    garde.as_ref().map_or(0, |etat| etat.image.hauteur)
}

/// La couleur du pixel `(x, y)` de l'image de l'état courant, empaquetée en `0x00RRGGBB` (voir « L'image »).
///
/// Rend `u32::MAX` si `x >= givre_largeur()`, si `y >= givre_hauteur()` et avant tout démarrage.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_pixel(x: u32, y: u32) -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return u32::MAX };
    let image = &etat.image;
    if x >= image.largeur || y >= image.hauteur {
        return u32::MAX;
    }
    let indice = (y as usize)
        .checked_mul(image.largeur as usize)
        .and_then(|ligne| ligne.checked_add(x as usize))
        .and_then(|pixel| pixel.checked_mul(3));
    let Some(indice) = indice else { return u32::MAX };
    let Some(fin) = indice.checked_add(3) else { return u32::MAX };
    match image.rvb.get(indice..fin) {
        Some([r, v, b]) => (u32::from(*r) << 16) | (u32::from(*v) << 8) | u32::from(*b),
        _ => u32::MAX,
    }
}

/// Le nombre de sites de l'amas (germe compris) pour la DLA et le claquage, de cases gelées pour un flocon (voir « Les mesures ») ; 0 avant
/// tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_sites() -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    match &etat.modele {
        Modele::Dla(amas) => amas.sites().len() as u32,
        Modele::Claquage(claquage) => claquage.sites().len() as u32,
        Modele::Flocon(flocon) => flocon.gelees() as u32,
    }
}

/// Le rayon de l'amas (DLA et claquage) ou du cristal (flocon), voir « Les mesures » ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_rayon() -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    match &etat.modele {
        Modele::Dla(amas) => amas.rayon(),
        Modele::Claquage(claquage) => claquage.rayon(),
        Modele::Flocon(flocon) => flocon.rayon_cristal(),
    }
}

/// La compacité du flocon (`Flocon::compacite().vers_f64()`, de 0 à 1) ; `NaN` pour la DLA, le claquage et avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_compacite() -> f64 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return f64::NAN };
    match &etat.modele {
        Modele::Dla(_) | Modele::Claquage(_) => f64::NAN,
        Modele::Flocon(flocon) => flocon.compacite().vers_f64(),
    }
}

/// Le `i`-ième mot (0 à 7) de 32 bits, gros-boutiste, du SHA-256 de l'écriture canonique de l'état (voir « L'empreinte »).
///
/// Les huit mots, chacun écrit sur 8 chiffres hexadécimaux minuscules et mis bout à bout dans l'ordre de 0 à 7, redonnent
/// `Amas::empreinte()` ou `Flocon::empreinte()` (64 caractères). Rend 0 si `i >= 8` et avant tout démarrage.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_empreinte_mot(i: u32) -> u32 {
    if i >= 8 {
        return 0;
    }
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    let ecriture = match &etat.modele {
        Modele::Dla(amas) => amas.ecriture_canonique(),
        Modele::Claquage(claquage) => claquage.ecriture_canonique(),
        Modele::Flocon(flocon) => flocon.ecriture_canonique(),
    };
    let empreinte = sha256(&ecriture);
    let debut = 4 * i as usize;
    let Some(fin) = debut.checked_add(4) else { return 0 };
    match empreinte.get(debut..fin) {
        Some([a, b, c, d]) => u32::from_be_bytes([*a, *b, *c, *d]),
        _ => 0,
    }
}

/// Mesure la dimension de l'amas du moment par la corrélation densité-densité (la fenêtre de [`crate::correlation`] pour la DLA, celle des petits amas de
/// [`crate::courbe`] pour le claquage), garde la courbe et rend son nombre de points ; 0 pour « pas de mesure » (voir « La mesure de la corrélation » :
/// flocon, amas trop petit ou de plus de [`MESURE_SITES_MAX`] sites, avant tout démarrage).
#[unsafe(no_mangle)]
pub extern "C" fn givre_mesurer() -> u32 {
    let mut garde = verrouiller();
    let Some(etat) = garde.as_mut() else { return 0 };
    let mesure = match &etat.modele {
        Modele::Dla(amas) => {
            if amas.sites().len() > MESURE_SITES_MAX {
                None
            } else {
                crate::correlation::mesure(amas.sites())
            }
        }
        Modele::Claquage(claquage) => {
            if claquage.sites().len() > MESURE_SITES_MAX {
                None
            } else {
                crate::courbe::mesure_petit_amas(claquage.sites())
            }
        }
        Modele::Flocon(_) => None,
    };
    match mesure {
        Some(mesure) => {
            let points = mesure.rayons.len() as u32;
            etat.mesure = Some(mesure);
            points
        }
        None => {
            etat.mesure = None;
            0
        }
    }
}

/// Le `i`-ième rayon de la fenêtre de la mesure gardée ; 0 sans mesure ou pour `i` hors de la courbe (voir « La mesure de la
/// corrélation »).
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_correlation_rayon(i: u32) -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    etat.mesure.as_ref().and_then(|mesure| mesure.rayons.get(i as usize)).copied().unwrap_or(0)
}

/// Le `C(r)` du `i`-ième rayon de la mesure gardée, bit pour bit celui de `correlation::mesure` ; `NaN` sans mesure ou pour `i` hors
/// de la courbe.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_correlation_valeur(i: u32) -> f64 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return f64::NAN };
    etat.mesure.as_ref().and_then(|mesure| mesure.valeurs.get(i as usize)).copied().unwrap_or(f64::NAN)
}

/// La dimension de la mesure gardée, bit pour bit celle de `correlation::mesure` ; `NaN` sans mesure.
#[unsafe(no_mangle)]
pub extern "C" fn givre_dimension() -> f64 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return f64::NAN };
    etat.mesure.as_ref().map_or(f64::NAN, |mesure| mesure.dimension)
}

/// Démarre un travail de la courbe `D(η)` : `graines` amas de `taille` sites pour chacun des `η = 0, 1, 2, 3` (voir « La courbe D(η) à la demande »).
/// Efface d'abord le travail précédent, quoi qu'il arrive. Rend 0 (accepté ; `givre_courbe_fait` vaut 0 et `givre_courbe_total` vaut `4 × graines`),
/// 1 si `taille` n'est pas de 1 à [`COURBE_TAILLE_MAX`] ou 2 si `graines` n'est pas de [`COURBE_GRAINES_MIN`] à [`COURBE_GRAINES_MAX`] (la taille est
/// vérifiée d'abord ; aucun travail n'est alors démarré). Ne fait croître aucun amas : c'est `givre_courbe_pas` qui calcule. Ne panique jamais.
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_demarrer(taille: u32, graines: u32) -> u32 {
    let mut garde = verrouiller_courbe();
    garde.travail = None;
    if taille == 0 || taille > COURBE_TAILLE_MAX {
        garde.code = 1;
        return 1;
    }
    if !(COURBE_GRAINES_MIN..=COURBE_GRAINES_MAX).contains(&graines) {
        garde.code = 2;
        return 2;
    }
    garde.travail = Some(TravailCourbe {
        taille: taille as usize,
        graines,
        fait: 0,
        total: 4 * graines,
        arret: None,
        dimensions: Vec::new(),
    });
    garde.code = 0;
    0
}

/// Le code d'erreur rendu pour une [`ErreurCourbe`] (voir la table de « La courbe D(η) à la demande »).
fn code_de_la_courbe(erreur: &ErreurCourbe) -> u32 {
    match erreur {
        ErreurCourbe::FrontiereTouchee { .. } => 3,
        ErreurCourbe::FenetreVide { .. } => 4,
        _ => 6,
    }
}

/// Fait **un** amas du travail de la courbe, le suivant (voir « La courbe D(η) à la demande »), et rend un code : 0 si l'amas est fait (ou si le travail est
/// déjà fini : rien ne change), 3, 4 ou 6 si cet amas échoue (le travail est alors arrêté, et tout appel de plus rend le même code sans rien faire), 5
/// s'il n'y a aucun travail. Quand l'amas fait est le dernier d'un `η`, le point de cet `η` est calculé. Ne panique jamais.
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_pas() -> u32 {
    let mut garde = verrouiller_courbe();
    let Some(travail) = garde.travail.as_mut() else {
        garde.code = 5;
        return 5;
    };
    if let Some(code) = travail.arret {
        garde.code = code;
        return code;
    }
    if travail.fait >= travail.total {
        garde.code = 0;
        return 0;
    }
    let eta = travail.fait / travail.graines;
    let graine = u64::from(travail.fait % travail.graines + 1);
    let taille = travail.taille;
    let demi_cote = match crate::courbe::essayer_demi_cote_de_la_courbe(eta, taille) {
        Ok(demi_cote) => demi_cote,
        Err(erreur) => {
            let code = code_de_la_courbe(&erreur);
            travail.arret = Some(code);
            garde.code = code;
            return code;
        }
    };
    match crate::courbe::essayer_dimension_d_une_graine(eta, graine, taille, demi_cote) {
        Ok(dimension) => {
            travail.fait += 1;
            travail.dimensions.push(dimension);
            garde.code = 0;
            0
        }
        Err(erreur) => {
            let code = code_de_la_courbe(&erreur);
            travail.arret = Some(code);
            garde.code = code;
            code
        }
    }
}

/// Le nombre d'amas faits du travail de la courbe : 0 au démarrage, `4 × graines` à la fin ; 0 sans travail.
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_fait() -> u32 {
    let garde = verrouiller_courbe();
    garde.travail.as_ref().map_or(0, |travail| travail.fait)
}

/// Le nombre total d'amas du travail de la courbe : `4 × graines` ; 0 sans travail.
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_total() -> u32 {
    let garde = verrouiller_courbe();
    garde.travail.as_ref().map_or(0, |travail| travail.total)
}

/// Le code du dernier appel de `givre_courbe_demarrer` ou de `givre_courbe_pas` (voir la table de « La courbe D(η) à la demande ») ; 0 avant tout appel.
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_code() -> u32 {
    verrouiller_courbe().code
}

/// La moyenne (`erreur_type` faux) ou l'erreur-type (vrai) du point de `eta` (0 à 3) une fois ses `graines` amas faits ; `NaN` avant, sans travail
/// et pour `eta > 3`.
fn statistique_de_la_courbe(eta: u32, erreur_type: bool) -> f64 {
    let garde = verrouiller_courbe();
    let Some(travail) = garde.travail.as_ref() else { return f64::NAN };
    if eta > 3 {
        return f64::NAN;
    }
    let graines = travail.graines as usize;
    let debut = eta as usize * graines;
    let Some(fin) = debut.checked_add(graines) else { return f64::NAN };
    if (travail.fait as usize) < fin {
        return f64::NAN;
    }
    let Some(valeurs) = travail.dimensions.get(debut..fin) else { return f64::NAN };
    if valeurs.len() < 2 {
        return f64::NAN;
    }
    let (moyenne, ecart_type) = crate::courbe::moyenne_et_erreur_type(valeurs);
    if erreur_type { ecart_type } else { moyenne }
}

/// La moyenne des dimensions du point de `eta` (0 à 3), bit pour bit celle de `essayer_point_de_la_courbe`, une fois les `graines` amas de cet `eta`
/// faits ; `NaN` avant, sans travail et pour `eta > 3`.
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_moyenne(eta: u32) -> f64 {
    statistique_de_la_courbe(eta, false)
}

/// L'erreur-type de la moyenne du point de `eta` (0 à 3), bit pour bit celle de `essayer_point_de_la_courbe`, une fois les `graines` amas de cet `eta`
/// faits ; `NaN` avant, sans travail et pour `eta > 3`.
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_erreur_type(eta: u32) -> f64 {
    statistique_de_la_courbe(eta, true)
}

/// Le repère publié de `eta` : `crate::courbe::publie(eta)` (`1,71`, `1,433`, `1,263` pour `η = 1, 2, 3`), `NaN` quand il n'y en a pas (`η = 0` et
/// `η >= 4`). Indépendant de tout travail et de tout modèle.
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_repere(eta: u32) -> f64 {
    crate::courbe::publie(eta).unwrap_or(f64::NAN)
}

/// Le champ dessiné d'un flocon et son `plein` (voir « Le champ dessiné ») : une valeur entière par case, dans l'ordre
/// canonique de `Flocon::masses`, et `4 E`. Aucun `f64`, aucun débordement (arithmétique saturée).
fn champ_du_flocon(flocon: &Flocon, duree: u32) -> (Vec<i64>, i64) {
    let e = flocon.parametres().gamma.0.saturating_mul(i64::from(duree)).clamp(1, 1 << 58);
    let valeurs = flocon
        .masses()
        .iter()
        .map(|masse| if masse.0 < Fixe::UN.0 { 0 } else { (masse.0 - Fixe::UN.0).saturating_mul(3).saturating_add(e) })
        .collect();
    (valeurs, e.saturating_mul(4))
}

/// Prépare l'image nette du modèle en cours à `cote × cote` pixels et `sur × sur` échantillons par pixel, la garde dans le tampon et rend un code : 0 (l'image est
/// prête), 2 (rien n'est démarré), 1 (le modèle n'est pas sur la grille hexagonale), 3 (`cote` impair, inférieur à 2 ou supérieur à [`NETTE_COTE_MAX`]), 4 (`sur`
/// nul ou supérieur à [`NETTE_SUR_MAX`]), 5 (`cote × sur` supérieur à [`NETTE_GRILLE_MAX`]) ou 6 (`rendre` a refusé : jamais avec ces bornes), vérifiés dans cet ordre
/// (voir « L'image nette »). Une préparation qui échoue efface le tampon ; l'état, le temps, l'empreinte et l'image d'un pixel par case ne changent jamais. Ne panique jamais.
#[unsafe(no_mangle)]
pub extern "C" fn givre_nette_preparer(cote: u32, sur: u32) -> u32 {
    let mut garde = verrouiller();
    let Some(etat) = garde.as_mut() else { return 2 };
    etat.nette = None;
    let Modele::Flocon(flocon) = &etat.modele else { return 1 };
    if !(2..=NETTE_COTE_MAX).contains(&cote) || !cote.is_multiple_of(2) {
        return 3;
    }
    if !(1..=NETTE_SUR_MAX).contains(&sur) {
        return 4;
    }
    if cote * sur > NETTE_GRILLE_MAX {
        return 5;
    }
    let (valeurs, plein) = champ_du_flocon(flocon, etat.duree);
    let rayon = flocon.rayon();
    match rendre(rayon, &valeurs, &ReglagesImage { cote, rayon_vue: rayon, sur, plein }) {
        Ok(rvb) => {
            etat.nette = Some(Image { largeur: cote, hauteur: cote, rvb });
            0
        }
        Err(_) => 6,
    }
}

/// La largeur, en pixels, de l'image nette gardée (`cote` de la dernière préparation réussie) ; 0 sans tampon (voir « L'image nette »).
#[unsafe(no_mangle)]
pub extern "C" fn givre_nette_largeur() -> u32 {
    let garde = verrouiller();
    garde.as_ref().and_then(|etat| etat.nette.as_ref()).map_or(0, |image| image.largeur)
}

/// La hauteur, en pixels, de l'image nette gardée (`cote` de la dernière préparation réussie) ; 0 sans tampon (voir « L'image nette »).
#[unsafe(no_mangle)]
pub extern "C" fn givre_nette_hauteur() -> u32 {
    let garde = verrouiller();
    garde.as_ref().and_then(|etat| etat.nette.as_ref()).map_or(0, |image| image.hauteur)
}

/// La couleur du pixel `(x, y)` de l'image nette gardée, empaquetée en `0x00RRGGBB` comme [`givre_pixel`] (voir « L'image nette »). Rend `u32::MAX` si
/// `x >= givre_nette_largeur()`, si `y >= givre_nette_hauteur()` et sans tampon. Ne recalcule rien.
#[unsafe(no_mangle)]
pub extern "C" fn givre_nette_pixel(x: u32, y: u32) -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return u32::MAX };
    let Some(image) = etat.nette.as_ref() else { return u32::MAX };
    if x >= image.largeur || y >= image.hauteur {
        return u32::MAX;
    }
    let indice = (y as usize)
        .checked_mul(image.largeur as usize)
        .and_then(|ligne| ligne.checked_add(x as usize))
        .and_then(|pixel| pixel.checked_mul(3));
    let Some(indice) = indice else { return u32::MAX };
    let Some(fin) = indice.checked_add(3) else { return u32::MAX };
    match image.rvb.get(indice..fin) {
        Some([r, v, b]) => (u32::from(*r) << 16) | (u32::from(*v) << 8) | u32::from(*b),
        _ => u32::MAX,
    }
}

/// Démarre une neige : les réglages du jeu nommé `jeu` (de 0 à 18, voir « La neige »), sauf `ρ`, `β`, `κ` et `μ` qui prennent `rho`, `beta`, `kappa` et `mu` (en millionièmes, dans cet ordre), le rayon et le nombre de pas du jeu ; remplace l'état entier
/// (le temps 0, l'image, l'empreinte, le tampon de l'image nette, la mesure gardée). Rend 0 (démarrée), 1 (`jeu >= 19`), 2 (`ρ` hors de `[0 ; 1]`), 3 (`β` hors de `[1 ; 8]`), 4 (`κ` hors de `[0 ; 1]`), 5 (`μ` hors de `[0 ; 1]`) ou 6 (tout autre refus : inatteignable), **dans cet ordre** ;
/// un refus ne change rien et rien ne panique, pour aucun `u32`.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_neige_demarrer(jeu: u32, rho: u32, beta: u32, kappa: u32, mu: u32) -> u32 {
    todo!("étape G3 : voir « La neige »")
}

/// Le nombre de jeux de réglages nommés de la neige : [`NEIGE_JEUX`], soit 19 (voir « La neige »).
#[unsafe(no_mangle)]
pub extern "C" fn givre_neige_jeux() -> u32 {
    todo!("étape G3 : voir « La neige »")
}

/// Un champ du jeu nommé `jeu` (de 0 à 18) : la figure, le rang, le nombre de cristaux, le réglage qui varie, les sept réglages en millionièmes, le rayon du domaine, le nombre de pas, « déduit » (voir « La neige » : le tableau des champs, de 0 à 13) ;
/// `u32::MAX` hors de portée. Ne dépend d'aucun état et n'en change aucun.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_neige_jeu(jeu: u32, champ: u32) -> u32 {
    todo!("étape G3 : voir « La neige »")
}

/// Un champ des réglages de la neige **en cours** : `ρ`, `β`, `α`, `θ`, `κ`, `μ`, `γ` en millionièmes (de 0 à 6), le numéro du jeu démarré (7), le rayon du domaine (8), le nombre de pas (9) ; `u32::MAX` sans neige en cours ou hors de portée (voir « La neige »).
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_neige_reglage(champ: u32) -> u32 {
    todo!("étape G3 : voir « La neige »")
}

/// Une mesure de la neige **en cours**, à l'instant : les six mesures de `balayage::mesurer` (de 0 à 5), son étiquette de `balayage::classer` (6 : 0 aucune, 1 plaque, 2 secteurs, 3 dendrite, 4 fougère), « au mur » (7 : 1 ou 0) ; `u32::MAX` sans neige en cours ou hors de portée
/// (voir « La neige »). Ne garde rien et ne change rien.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_neige_mesure(champ: u32) -> u32 {
    todo!("étape G3 : voir « La neige »")
}
