# Étape G3 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape G3 (la neige de Gravner et Griffeath sur la page : cinq exportations de plus au module, un panneau et un menu de jeux nommés, des mesures et une étiquette) attend, d'où elle vient, avec quelle tolérance et quel critère de réussite.
Chaque identifiant (`V-G3-…`) est cité dans la doc des tests qui l'emploient (`tests/etape_g3_*.rs`). Elle est séparée des résultats figés de notre propre code (`references/*.empreinte`) : **aucune valeur « juste » de cette fiche ne vient du prototype de l'écrivain**. Les seules valeurs tirées de notre code sont celles des
tests étiquetés « régression » (`l_empreinte_des_images_de_reference`, V-G3-15 ; `jusqu_au_bout_comme_g2_et_la_bibliotheque`, V-G3-06, qui compare au tableau de G2 : un résultat figé de notre code par G2), qui le disent en toutes lettres. Une valeur vient d'une publication lue (dite lue, ou dite non lue), d'un calcul exact
fait par un outil indépendant (nommé), d'un calcul fait à la main (écrit dans le test) ou d'une convention que la doc de `src/vitrine.rs` fixe (dite telle, avec sa raison).

**Outils indépendants** (Python 3.13, dans `essai-g3-outils/` des notes de la session ; le code Rust n'est jamais l'outil) : `germe_ref.py`, écrit d'après la **doc seule** de `rendu_hex` (la géométrie, le dégradé) sans lire le code Rust : il construit en fractions exactes l'image du germe seul (`(556, 1)`, vue de rayon 69, `plein` 1,5) :
douze pixels dans l'hexagone, la couleur `(58, 133, 192)`, l'image entière de SHA-256 `84149be2…a766` (V-G3-14) ; `mur_g3` (l'exemple `g3_mur.rs`, en Rust : une **mesure**, pas un outil de vérification) a joué trente tirages de réglages pour dire où commence le mur (V-G3-MES-02).

**Critères écrits avant les calculs** (ce qu'on accepte comme source et comme tolérance) :

1. Les tests « exacts » (les jeux, les codes, la conversion, le mur, les images, les textes) n'ont **aucune** tolérance : égalité d'entiers, d'octets, de chaînes. La doc fixe chaque conversion, chaque borne (incluse ou non), chaque ordre de refus et la règle du mur : une implémentation fidèle donne les mêmes entiers.
2. **Aucun nombre publié n'est un test.** L'article de Gravner et Griffeath ne donne, pour les figures 9, 10, 12 et 13, que des images et des réglages. Les réglages sont des **entrées**, relus à la source par G1 et G2 (V-G3-01) ; ce que notre code en tire est une **régression** (V-G3-06, V-G3-15) ou une égalité entre deux
   voies de **notre** code (le module et la bibliothèque : V-G3-05, V-G3-08).
3. **L'étiquette est notre modèle** (G2) : la page dit « notre classement, pas celui de l'article » (V-G3-25, V-G3-27) ; les tests la vérifient comme un numéro (V-G3-10), pas comme une vérité.
4. Un réglage d'un jeu nommé n'est **jamais écrit dans la page** (V-G3-22) : il vient du module, qui le tient de `balayage` (V-G3-02). Les curseurs doivent atteindre chaque jeu sur leur grille (V-G3-21).
5. Les tailles des tests viennent du **coût mesuré** (`MODELE.md`, « Le coût ») : de 25 à 62 de rayon et de 1 300 à 3 000 pas, un domaine de rayon 100 ; trois jeux (de 1,5 à 3,5 s chacun en natif) jusqu'au bout, les seize autres sur leurs premiers pas.
6. **Un chiffre du brief se mesure** (leçon 14) : les chiffres de coût et de mur du brief (« ~0,8 s par 1 000 pas à `R = 100` », « 73 % du rayon du domaine », « 26 ms sous Node ») ont été remesurés : voir V-G3-MES-01 à 04.

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les tests `régression :` de l'étape G3 sont `l_empreinte_des_images_de_reference` et `jusqu_au_bout_comme_g2_et_la_bibliotheque` (en partie) ; `les_cinq_etiquettes_passent_par_le_module` est « régression + juste » (les instants où la bibliothèque donne chaque étiquette
sont des mesures de notre code ; l'égalité du numéro est exacte).

---

## Les références publiées (relues à la source par G1 et G2, pas par G3)

**V-G3-00 : ce qui a été lu.** G3 **ne lit aucune publication de plus** : les jeux de réglages qu'il met sur la page sont ceux que G1 (`references/etape-g1-valeurs.md`, V-G1-01 à V-G1-03) et G2 (`references/etape-g2-valeurs.md`, V-G2-00 à V-G2-05) ont relus à la source (J. Gravner et D. Griffeath, « Modeling snow crystal growth II: A mesoscopic
lattice map with plausible dynamics », *Physica D* 237 (2008) 385-404, <https://doi.org/10.1016/j.physd.2007.09.008>, dans la préimpression de l'auteur : section 5, figures 9, 10 et 12, annexe 8, et le texte de la section 6 pour la figure 13). **Je ne les ai pas relus** : la fiche renvoie à G2 et dit, comme G2, ce qui est lu et ce qui est déduit.
**La version publiée dans *Physica D* n'a pas été lue** (G1 et G2 : l'éditeur en refuse l'accès) ; la figure 11 (le réglage `κ`) et les figures 14 à 17 ne sont pas lues non plus : la neige de G3 n'en emploie aucune.

**V-G3-01 : les dix-neuf jeux nommés** (`les_jeux_nommes_sont_ceux_de_la_source`, `les_curseurs_atteignent_chaque_jeu_nomme`, `etape_g3_commun::JEUX`). Chaque jeu est un cristal d'une figure, avec ses sept réglages `ρ β α θ κ μ γ` en millionièmes, un domaine de rayon 100 et un nombre de pas :

| jeux | figure | réglage qui varie | valeurs | autres réglages (`ρ β α θ κ μ γ` sans le balayé) | pas | source |
|---|---|---|---|---|---|---|
| 0 à 5 | 9 | `ρ` | 0,40 ; 0,41 ; 0,42 ; 0,44 ; 0,46 ; 0,50 | `β = 1,3`, `α = 0,08`, `θ = 0,025`, `κ = 0,003`, `μ = 0,07`, `γ = 0,00005` | 2000 | G2 V-G2-01 : légendes de la figure 9 ; annexe 8, ligne « 9 » (colonne `ρ` absente) ; **lu** (la ligne « 15tr » en est la copie avec `ρ = 0,5`) |
| 6 à 11 | 10 | `β` | 1,9 ; 2,2 ; 2,4 ; 2,6 ; 2,7 ; 2,8 | `ρ = 0,8`, `α = 0,004`, `θ = 0,001`, `κ = 0,05`, `μ = 0,015`, `γ = 0,0001` | 1300 | G2 V-G2-02 : légendes de la figure 10 ; ligne « 10 » (colonne `β` absente) ; **déduit** |
| 12 à 17 | 12 | `μ` | 0,04 ; 0,05 ; 0,06 ; 0,07 ; 0,08 ; 0,09 | `ρ = 0,5`, `β = 1,4`, `α = 0,1`, `θ = 0,005`, `κ = 0,001`, `γ = 0,001` | 2000 | G2 V-G2-03 : légendes de la figure 12 ; ligne « 12 » (colonne `μ` absente) ; **déduit** |
| 18 | 13 (milieu) | aucun | — | `ρ = 0,36`, `β = 1,09`, `α = 0,01`, `θ = 0,0745`, `κ = 0,0001`, `μ = 0,14`, `γ = 0,00001` | **3000** | G1 V-G1-02 : le texte de la section 6 et la ligne « 13m » de l'annexe donnent les mêmes valeurs ; **lu**. Les 3000 pas sont **un choix à nous** (voir V-G3-MES-03) |

Le champ « déduit » du module (`givre_neige_jeu(jeu, 13)`) vaut 1 pour les jeux 6 à 17 : la page le dit (« Réglages déduits de l'annexe de l'article : la colonne du réglage qui varie y manque »). Tolérance : sans objet (des entrées). Critère : chaque nombre est égal à la table **écrite à la main** ; chaque valeur est sur la grille d'un curseur de la page.

**V-G3-02 : une seule source** (`les_jeux_nommes_ont_une_seule_source`). Les dix-huit premiers jeux sont `balayage::jeu_rho()`, `jeu_beta()`, `jeu_mu()` (`reglage.fixer(base, valeurs[rang])`, `rayon`, `pas`) et le jeu 18 est `g1::FIG13_MILIEU` ; **le module les lit dans la bibliothèque, la page ne les écrit pas**. Critère : égalité exacte, après la conversion
`ppm = floor((2 000 000 · brut + 2^32) / 2^33)`, avec l'aller-retour `lire_decimal(ppm écrit en décimal)` qui redonne le `brut`.

## Les conventions de la doc de `src/vitrine.rs` (« La neige »)

**V-G3-03 : le champ, la vue et le `plein` de l'image nette de la neige** (`l_image_nette_de_la_neige`, `le_germe_seul_a_la_main`). Convention de la doc : le champ est `Neige::valeurs_du_cristal` (la masse cristalline `c` des cases rattachées, 0 ailleurs : G1), `plein = 6 442 450 944` (1,5, comme les images de G1 et de G2), la vue de rayon
`R · 65 / 100 + 4 = 69` (fixe). **Raison** : les images regardées (`essai-g3-outils/images/`) ; 1,5 donne aux cases rattachées (`c` de 1 à 1,1 pour la figure 13, jusqu'à 2,1 pour `β = 1,9`) des bleus moyens à foncés sans tout saturer ; la vue 69 contient un cristal arrêté au mur (rayon 65, plus 4 de marge) et reste la même pendant
toute l'animation (pas de « respiration » de l'image). Critère : l'image est, octet pour octet, `rendre(100, valeurs_du_cristal, { cote, rayon_vue: 69, sur, plein: 6 442 450 944 })` (écrits à la main dans le test).

**V-G3-04 : la conversion des réglages en `Fixe`** (`la_conversion_des_reglages_en_fixe`). `Fixe::rapport(ppm, 1 000 000)` : `brut = floor((2 ppm 2^32 + 10^6) / (2 · 10^6))`. **Calculé à la main** (`ppm × 2^32 / 10^6`, partie fractionnaire entre parenthèses) : `1 → 4294,967… → 4295` (et non 4294 : un `floor` se voit),
`3 → 12884,9… → 12885`, `50 → 214748,36… → 214748`, `400 000 → 1 717 986 918,4 → 1 717 986 918`, `500 000 → 2 147 483 648`, `999 999 → 4 294 963 001,04… → 4 294 963 001`, `1 000 000 → 4 294 967 296`. **Calcul indépendant** : le test construit l'état attendu avec `flocon::lire_decimal` du décimal `E.FFFFFF`
(la voie de G1, qui calcule `floor(N · 2^32 / 10^k + 1/2)` sur le texte) et exige le même état à l'octet. Tolérance : aucune.

**V-G3-05 : les codes de `givre_neige_demarrer` et leur ordre** (`les_codes_du_demarrage_et_leur_ordre`, `jamais_de_panique_pour_une_neige_absurde`). Convention de la doc : 0 démarrée, 1 jeu inconnu (`jeu >= 19`, vérifié d'abord), puis le **premier** réglage hors plage dans l'ordre `ρ β α θ κ μ γ` de `Parametres::verifier` (restreint aux quatre passés `ρ β κ μ`) :
2 pour `ρ` (plus de 1 000 000), 3 pour `β` (moins de 1 000 000 ou plus de 8 000 000), 4 pour `κ` (plus de 1 000 000), 5 pour `μ` (plus de 1 000 000), 6 (inatteignable). Les plages sont celles de G1 (`references/etape-g1-valeurs.md`, « Les réglages » : `0 <= ρ <= 1`, `1 <= β <= 8`, `0 <= κ, μ <= 1`, bornes incluses). Les plages
des **curseurs** de la page (`ρ` de 0 à 1, `β` de 1 à 4, `κ` de 0 à 0,1, `μ` de 0 à 0,2) sont plus étroites que celles du module : `β` s'arrête à 4 (le plus haut des jeux est 2,8), `κ` à 0,1 (le plus haut, 0,05), `μ` à 0,2 (le plus haut, 0,14). Tolérance : aucune.

**V-G3-06 : un jeu joué jusqu'au bout est le cristal de G2** (`jusqu_au_bout_comme_g2_et_la_bibliotheque`, `node_joue_un_jeu_jusqu_au_bout`). **Régression** : les jeux 5, 8 et 17 joués par le module jusqu'à leurs 2000, 1300 et 2000 pas ont les six mesures, l'étiquette et l'**empreinte** de leur ligne dans `apercus/etape-g2-mesures.csv` (calculé par G2 sous Linux et sous Windows : un résultat
de **notre** code, figé et revu par le juge de G2). Le jeu 18 est comparé à la bibliothèque seule (recalculée dans le test). Critère : égalité exacte. **Ce que cela établit** : le module de la page rend le cristal que G2 a publié dans son tableau, à l'octet de l'empreinte (« le même cristal partout » : le natif sous Linux et sous Windows, le module sous Node).

**V-G3-07 : le mur** (`le_mur_arrete_la_neige`). Convention de la doc : le scénario s'arrête au premier de la durée et du mur ; **avant chaque pas**, si `rayon_cristal >= R · 65 / 100` (65 pour `R = 100`) on ne joue pas ce pas. Comme le rayon du cristal croît d'au plus 1 par pas (G1), il vaut **exactement 65** à l'arrêt. La règle est écrite **en boucle simple**
dans le test (`etape_g3_commun::avancer_la_neige`, un `rayon_cristal()` avant chaque pas) : ce que le module rend (le nombre de pas, l'état, l'empreinte) est comparé à elle, pour trois réglages libres (le jeu 1 avec `ρ = 0,956862`, `β = 1,122299`, `κ = 0,048396`, `μ = 0,036667` ; le jeu 5 avec `ρ = 0,700218`, `β = 1,124914`, `κ = 0,07336`,
`μ = 0,031067` ; le jeu 0 avec `ρ = 1`, `β = 1`, `κ = 0`, `μ = 0`) : ce sont des réglages **choisis** pour atteindre le mur avant la durée, et le test exige que ce soit vrai **d'après la bibliothèque** (le nombre de pas vient de la boucle simple, pas d'un nombre écrit). **Le découpage ne change rien** (appels de 7 pas : même état).

**V-G3-08 à V-G3-13** : voir les tests (`l_image_nette_de_la_neige` V-G3-08, `le_tampon_et_l_etat_d_une_neige` V-G3-09, `les_cinq_etiquettes_passent_par_le_module` V-G3-10, `les_modeles_qui_existent_ne_changent_pas` V-G3-11, `jamais_de_panique_pour_une_neige_absurde` V-G3-12, `avant_tout_demarrage_la_neige_n_existe_pas` V-G3-13). Tous exacts.
V-G3-10 : les numéros d'étiquette (0 aucune, 1 plaque, 2 secteurs, 3 dendrite, 4 fougère : l'ordre de `balayage::Forme`) sont **écrits à la main** ; les instants où la bibliothèque donne chacune des cinq étiquettes (le jeu 6 à 200, 300 et 700 pas : plaque, secteurs, fougère ; le jeu 5 à 400 pas : dendrite ; le jeu 0 à 100 pas : aucune) sont des **mesures de notre code**
(`essai-g3-outils/g3_formes.rs`) ; le test exige que les cinq numéros se voient.

**V-G3-14 : le germe seul, à la main** (`le_germe_seul_a_la_main`). Au temps 0 la neige est une case rattachée de masse cristalline exactement 1. Avec `(556, 1)` et la vue de rayon 69 (139 cases de large, 4 pixels par case), le point échantillonné du pixel `(px, py)` est `X = (2 px − 555) / 8`, `Y = (2 py − 555) / 8` ; l'hexagone de la case centrale
(à plat en haut, côtés à la distance 1/2 : `|Y| <= 1/2` et `|√3 X / 2| + |Y / 2| <= 1/2`) contient **douze pixels** : lignes 276 et 279, colonnes 277 et 278 (`|Y| = 3/8`, `|X| <= 0,3608`) et lignes 277 et 278, colonnes 276 à 279 (`|Y| = 1/8`, `|X| <= 0,5052`). La couleur est celle du dégradé à `t = floor(1 × 1024 / 1,5) = 682` : `s = 2`, `f = 170`,
`R = (107 × 86 + 33 × 170 + 128) >> 8 = 58`, `V = (174 × 86 + 113 × 170 + 128) >> 8 = 133`, `B = (214 × 86 + 181 × 170 + 128) >> 8 = 192`. **Recoupé par `germe_ref.py`** (fractions exactes, d'après la doc de `rendu_hex` seule) : douze pixels, `(58, 133, 192)`, l'image entière de SHA-256 `84149be2448ab917af0a6b7bd6ad8923bdafb092d04d8c8244f187da59eaa766`.
Tolérance : aucune. Critère : l'image de la vitrine égale l'image construite ici, pour les jeux 0, 6 et 18.

**V-G3-15 : les empreintes de l'image nette** (`l_empreinte_des_images_de_reference`). **Régression** : le SHA-256 des octets R, V, B de l'image nette du jeu 3 à 150 pas en `(96, 3)`, du jeu 6 à 300 pas en `(128, 2)` et du jeu 18 à 200 pas en `(62, 2)`, tirés du prototype de l'écrivain (tout est en entiers : les mêmes sous Linux et sous Windows). Elles ne disent pas que l'image est juste :
V-G3-14 et V-G3-08 le disent, sans lire de référence.

## Les pages et le module (formes)

**V-G3-20 à V-G3-27** (`etape_g3_page.rs`) : formes de la page (le panneau, les curseurs en millionièmes, le menu vide dans la page, l'état qui suit le modèle démarré, la neige jouée par lots avec un budget de temps, l'empreinte non calculée pendant l'animation de la neige, les textes qui ne revendiquent pas plus que le modèle, `lecture.js` sans DOM) ;
`lecture_js_lit_la_neige_sur_un_faux_module` (V-G3-27) est **juste** : un faux module **qui change d'état** et des textes calculés à la main. **V-G3-30 à V-G3-33** (`etape_g3_module.rs`) : le module sous Node. **Aucune de ces vérifications n'a tourné sur Node 20 ni sur le module** dans le poste de l'écrivain (ni Node ni `wasm32` : un faux `node` = `bun` et un faux module) : la
VM les joue.

## Les mesures (jamais des preuves)

**V-G3-MES-01 : le coût natif** (l'exemple `g3_mesures1.rs`, profil `dev` à `opt-level = 3`, un poste de 12 fils partagé) : `givre_avancer(5)` 4,4 ms (0,88 ms par pas), un pas seul 1,7 à 1,8 ms, image nette `(384, 2)` 15,2 ms et sa lecture 1,5 ms, `(1024, 3)` 214 ms et 10 ms, huit mots d'empreinte 25 à 28 ms, huit mesures 1,1 à 2,8 ms, un démarrage 3,1 ms, quatre jeux (5, 6, 12, 18) à 1 % près (`g3_mesures2_sortie.txt`). **Le brief disait ~0,8 s par 1000 pas : mesuré 0,87 à 0,89 s au calme, jusqu'à 1,4 à 1,8 ms par pas quand le poste est chargé** (`g3_mesures1.rs` : 2000 pas des jeux 0 à 17 en 1,6 à 3,7 s ; les chiffres du calme sont ceux de la doc).

**V-G3-MES-02 : le mur** (l'exemple `g3_mur.rs`, trente tirages d'un jeu parmi les 18 et de `ρ ∈ [0,3 ; 1]`, `β ∈ [1 ; 3,5]`, `κ ∈ [0 ; 0,1]`, `μ ∈ [0 ; 0,15]`, graine fixe de l'exemple) : sur les 30 tirages, **six** atteignent le rayon 60 dans leurs pas (`ρ` de 0,69 à 0,96) ; pour chacun, au rayon 60, 65, 70, 75 et 80, les cases rattachées du domaine de rayon 100 sont **les mêmes** que dans un domaine de rayon 140 au même nombre de pas ; les vingt-quatre autres ne s'en approchent pas (`g3_mur_8_30.txt` et la sortie de `g3_mur.rs` sur 0 à 8). **Les trois jeux les plus rapides poursuivis au-delà de leurs pas** (`g3_mur2.rs`, `g3_mur3.rs`) : le jeu 5 (figure 9, `ρ = 0,50`) a les mêmes cases jusqu'au rayon 65 et en diffère dès 70 (t = 2709) ; le jeu 12 (`μ = 0,04`) jusqu'à 70, en diffère dès 75 ; le jeu 6 (`β = 1,9`) jusqu'à 80, en diffère dès 85. **Conséquence** : la limite du mur est **65 % du rayon du domaine** (la plus haute qui ne coupe aucun jeu nommé : le plus grand finit à 62, et que les trois jeux les plus rapides tiennent) ; **ce n'est pas un certificat de fidélité** (trois jeux, trente tirages). Le brief annonçait « 73 % » (G2) : **mesuré 62 % au plus** du rayon du domaine 100 (« 73 % » ne se retrouve pas ; 62/84 = 74 % serait le rapport au domaine de rayon 84 où G2 dit que deux cristaux sentent déjà le mur : supposé, non vérifié).

**V-G3-MES-03 : les 3000 pas du milieu de la figure 13** : à 2000, 3000, 4000 et 6000 pas dans un domaine de rayon 100, le cristal a 29, 35, 41 et 49 de rayon (1465, 2125, 2941 et 4201 cases) ; l'article demande plus de 100 000 pas. À 3000 pas l'image montre six bras qui finissent en plaques hexagonales (« a stellar crystal with plate ends »,
le mot de l'article : **à l'œil, pas un test**) : c'est le choix retenu, un stade précoce.

**V-G3-MES-04 : le coût de ce que la page lit** : l'empreinte (huit SHA-256 de 750 Ko) coûte 25 à 28 ms natifs : la page ne la lit pas pendant l'animation de la neige (elle écrit « calculée à l'arrêt ») ; les mesures (huit appels) 1,1 à 2,8 ms, lues toutes les 500 ms. **Pas mesuré** : Node, un navigateur, un téléphone.

## Ce que le juge a ajouté

**V-G3-28 : les fonctions de la neige de `app.js`, mot pour mot** (`app_js_ecrit_la_neige_mot_pour_mot`, **forme**). Valeur attendue : le corps de quinze fonctions de `docs/app.js` (`afficherEmpreinte`, `afficherMesures`, `afficherLaNeige`, `afficherLesCurseursNeige`, `remplirLeMenuNeige`, `poserLesCurseursNeige`, `demarrerLaNeige`,
`texteDeFin`, `montrerLesReperes`, `avancer`, `avancerLaNeige`, `image`, `texteDEtat`, `relancer`, `choisirUnJeuNeige`), aux espaces près, et l'ordre du chargement (`remplirLeMenuNeige();` avant le premier `relancer();`). Source : le prototype de la page, relu comme du code par le juge (l'état suit le modèle démarré, pas de résidu d'un modèle à l'autre) ; ce n'est pas une preuve
du comportement : une régression de texte. Raison : onze fautes de page plausibles passaient les tests de forme V-G3-20 à V-G3-27 (`essai-g3-juge-outils/mut1-P.txt`, Q01, Q03, Q04, Q08, Q10, Q22, Q23, Q28, Q29, Q32, Q33) ; la page se recopie à l'octet. Tolérance : les blancs.

**V-G3-09, complété : la mesure gardée de corrélation** (`le_tampon_et_l_etat_d_une_neige`, **juste**). Après un claquage `η = 0` entier (399 unités) mesuré par `givre_mesurer` (au moins un point), un `givre_neige_demarrer` refusé (codes 1 et 2) laisse `givre_correlation_rayon(0)` et `givre_dimension()` tels quels (bit pour bit) ; un démarrage réussi
les rend vides (0 et `NaN`) **avant tout `givre_mesurer`**. Source : la doc (« L'état est remplacé entièrement », « un refus ne change rien »). Raison : une neige qui gardait la mesure du modèle d'avant passait tous les tests (`givre_mesurer`, que `exiger_la_neige` appelle, l'efface de toute façon).

**V-G3-MES-05 : ce que le juge a mesuré** (jamais des preuves ; `essai-g3-juge-outils/`). Le dégradé (`images.txt`) : `c` au plus 1,37 (figure 9), 1,45 (figure 12), 1,10 (figure 13), mais 2,13, 2,68 et 3,07 pour la figure 10 (`β = 1,9`, `2,4`, `2,8`), où 22 %, 18 % et 52 % des cases saturent ; des cases rattachées ont `c` bien sous 1 (jusqu'à 0,03 : `t = 20` sur 1024).
Le mur (`mur.txt`, soixante tirages sur la grille des curseurs, graine 20261111) : huit arrêts au mur, les huit avec les mêmes cases qu'un domaine de rayon 140 jusqu'à l'arrêt ; les cinquante-deux autres finissent par la durée. Jamais de panique (`panique.txt`) : 1 240 771 appels, 209 neiges, 32 805 démarrages absurdes (384 réussis ; codes 1 à 5 : 19 683, 7 290, 3 888, 1 080, 480).
Le coût natif au calme (`cout-calme.txt`) : 1000 pas 0,84 à 0,87 s ; un lot de cinq pas 5,5 à 5,6 ms ; `(384, 2)` 18,5 à 20,8 ms + 1,7 ms ; `(320, 1)` 4,3 à 4,9 ms ; `(1024, 3)` 265 à 272 ms ; l'empreinte 31 à 35 ms ; les mesures 1,2 à 2,9 ms ; un démarrage 3,5 ms. Sur le poste chargé par les bancs de fautes (`cout-charge.txt`) : 1000 pas 1,2 à 1,8 s ; `(384, 2)` 22 à 31 ms ; `(1024, 3)` 350 à 440 ms ; l'empreinte 35 à 74 ms.
Le second prototype (`dump-p2.txt`, `dump-pw.txt`) : 4 495 lignes identiques, SHA-256 `3441075900f238db447cc175664a5fff944ee96c9f40bf3c0762134b7323984f`.

**La source, relue par le juge** (la préimpression : les images des figures 9, 10 et 12, le texte des sections 5 et 6, l'annexe 8) : les légendes des trois figures donnent bien `ρ = .40, .41, .42, .44, .46, .50`, `β = 1.9, 2.2, 2.4, 2.6, 2.7, 2.8` et `μ = .04` à `.09` ; les lignes « 9 », « 10 » et « 12 » de l'annexe ont sept nombres au lieu de huit
(la colonne du réglage qui varie manque **aux trois**) ; la ligne « 15tr » redonne la ligne « 9 » avec `ρ = .5` (d'où « lu » pour la figure 9) ; rien ne recoupe en entier les lignes « 10 » et « 12 » (d'où « déduit ») ; la ligne « 13m » et le texte disent les mêmes sept nombres. **La ligne « 13l » de l'annexe, elle, contredit le texte** (`α = .2`, `θ = .026`, `γ = .0001` dans
l'annexe ; `α = .026`, `θ = .2`, `γ = .00001` dans le texte) : le cristal de gauche de la figure 13 n'est pas employé, et ne doit pas l'être sans trancher. Aucun résultat publié n'est testé : l'article ne donne que des images et des réglages.
