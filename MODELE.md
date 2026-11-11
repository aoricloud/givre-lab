# Givre Lab : le modèle

Ce document nomme, pour chaque mécanisme, sa référence publiée, ses choix et ses limites. Une section par étape ; chaque
section finit par ce que ses tests n'établissent pas.

## Étape 0 : le socle (générateur, grilles, arithmétique, empreinte, image)

Rien ne simule encore de cristal : l'étape pose ce dont toutes les suivantes se servent et qui doit être identique, au
bit près, sous Linux et sous Windows. Les valeurs attendues de ses tests, chacune avec sa source, sont dans
`references/etape-0-valeurs.md`.

### Le générateur pseudo-aléatoire : `alea`

PCG32 (variante XSH-RR 64/32) de Melissa O'Neill, *PCG: A Family of Simple Fast Space-Efficient Statistically Good
Algorithms for Random Number Generation*, rapport HMC-CS-2014-0905 (2014), et la page de référence pcg-random.org.
`Alea::avec_flux(graine, flux)` est exactement `pcg32_srandom_r` de la référence ; `Alea::new(graine)` prend le flux 54.
Les sorties de la graine 42 et du flux 54 sont celles de la démonstration officielle (`pcg32-demo`). `borne(n)` suit
`pcg32_boundedrand_r` (tirage sans biais par rejet).

### Les grilles : `reseau`

La grille carrée (indice par lignes, voisinages à 4 et à 8 dans un ordre fixe, disque en entiers) et l'hexagone en
coordonnées axiales (q, r), s = −q−r, avec ses six voisins dans un ordre fixe et la rotation de 60° exacte en entiers
(q, r, s) → (−r, −s, −q). La rotation exacte en entiers est ce qui rendra la symétrie d'ordre 6 des flocons (étape 3)
exacte au bit près. Référence pour les coordonnées hexagonales : Red Blob Games, *Hexagonal Grids* (Amit Patel).

### L'arithmétique écrite dans le dépôt : `math`

Aucune fonction de `f64` du système (`sin`, `exp`, `ln`, `powf`…) : elles passent par la bibliothèque mathématique du
système et peuvent différer d'un système à l'autre, donc d'une empreinte à l'autre. À la place : une racine carrée
entière exacte ; un logarithme népérien écrit avec `+ - * /` seulement (réduction par l'exposant, série de l'argument
hyperbolique), exact à 5e-16 en relatif, mesuré à 2,95e-16 contre la valeur exacte ; la table `EXP_NEG` de
`round(2^32 · e^(−k/16))` (probabilités d'Arrhenius de l'étape 4, lues dans une table, sans exponentielle), recalculée
en entiers de précision arbitraire ; et la virgule fixe `Fixe` (Q31.32, arrondi à l'unité du dernier chiffre, documenté),
pour que les sommes de l'étape 3 soient des sommes d'entiers.

### Le scénario, l'empreinte, l'image, le lancement

- Le scénario : un fichier d'une directive par ligne. L'étape 0 n'interprète que la forme (`boite` ou `hexagone`) ; les
  autres directives sont gardées dans l'ordre, pour les étapes suivantes, et `lancer` refuse celles qu'il ne comprend
  pas.
- L'empreinte : SHA-256 de NIST FIPS 180-4, sur l'écriture canonique de l'état, décrite octet par octet dans la doc de
  `lancer`. Les vecteurs de test sont ceux du NIST (exemples de FIPS 180-4 et fichiers du CAVP).
- L'image : un écrivain PNG (spécification W3C PNG, ISO/IEC 15948 ; zlib, RFC 1950 ; deflate, RFC 1951), RVB 8 bits, blocs
  deflate stockés, CRC-32 et Adler-32 écrits dans le dépôt ; sa sortie est relue avec `zlib` et un décodeur indépendant.
- Le lancement : `givre lancer <scénario> --graine <n> --sortie <dossier>` écrit l'empreinte, les mesures en CSV et
  l'aperçu (la graine seule, pour l'œil).

### Ce que cette étape n'établit pas

- L'égalité entre Linux et Windows n'est établie que pour le logarithme (au bit, sur 38 402 points mesurés sous les deux
  systèmes) et par la suite de tests, verte des deux côtés. Que l'empreinte de référence
  (`references/vide-graine-1.empreinte`) soit la même sous les deux systèmes n'est prouvé que par la CI, qui la rejoue :
  une empreinte figée prouve que rien n'a bougé, pas que c'est juste.
- La garde des sources (`tests/etape_0_regles.rs`) lit le texte de `src/` : elle ne voit ni une macro, ni un `include!`,
  ni une fonction du système atteinte sans que son nom soit écrit dans `src/`. Elle refuse le type `HashMap` en entier,
  ce qui est plus strict que la règle du dépôt.
- Le test du logarithme compare à la fonction `ln` du système où il tourne : une bibliothèque mathématique future moins
  exacte que 2e-16 pourrait le faire échouer sans faute du code (marge mesurée : 1,56).
- Le générateur n'est éprouvé que par ses vecteurs officiels et par `borne` ; son uniformité statistique n'est pas
  mesurée ici. Les vecteurs « Monte Carlo » du CAVP ne sont pas joués.
- Aucun lecteur d'images réel n'a ouvert un aperçu : le PNG est relu par `zlib` et par le décodeur du test.

## Étape 2s : la croissance de surface en bande (dépôt aléatoire, relaxation, balistique, Eden)

Quatre modèles font croître un front sur une bande de L colonnes périodique en x, et on mesure l'exposant de croissance β
de sa rugosité, w(t) ~ t^β, avec w² = (1/L) Σ (h − moyenne)², calculée exactement en entiers. Les règles, le hasard consommé
appel par appel et l'écriture canonique de l'état sont dans la doc de `src/surface.rs` ; les valeurs attendues des tests,
chacune avec sa source, dans `references/etape-2s-valeurs.md`.

### Références

- Dépôt aléatoire : aucune corrélation entre colonnes, β = 1/2. Solution exacte démontrée dans ce dépôt (loi multinomiale
  de N = tL particules sur L colonnes) : E[w²] = t (1 − 1/L) et Var(w²) = 2n(n − 1)(L − 1)/L⁴ ; la source publiée ne donne
  que W² = t (Chou et Pleimling ; Kardar, cond-mat/9507019, éq. 2), le facteur 1 − 1/L est un calcul.
- Relaxation de surface : Family, J. Phys. A 19 (1986) L441 ; échelle de Family et Vicsek, J. Phys. A 18 (1985) L75, w(L, t) ~
  L^α f(t / L^z) avec z = α / β ; classe d'Edwards-Wilkinson (1982), β = 1/4. Le texte de 1986 n'a pas pu être lu : la règle
  (la particule ne quitte sa colonne que pour un voisin strictement plus bas) est la lecture majoritaire de quatre sources
  ouvertes (Mal et Ray, arXiv:1005.3513 ; Mal, Ray et Shamanna, arXiv:0805.0647 ; Chou et Pleimling, arXiv:1007.2380 ;
  Raychaudhuri et al., arXiv:cond-mat/0105483).
- Dépôt balistique : h_i(n + 1) = max(h_{i−1}(n), h_i(n) + 1, h_{i+1}(n)), Farnudi et Vvedensky, Phys. Rev. E 83 (2011)
  020103(R), éq. 2 ; classe KPZ (Kardar, Parisi et Zhang, Phys. Rev. Lett. 56 (1986) 889), β = 1/3.
- Eden : une particule s'ajoute à un site du périmètre tiré uniformément (Eden 1961, texte non lu) ; classe KPZ. L'ordre du
  périmètre et celui des voisins sont des conventions de ce dépôt, pas de la littérature : elles fixent le hasard.

### Ce qui est mesuré, et l'écart à la valeur publiée

À L = 1024, sur les instants de 16 à 512 couches, une pente par groupe de graines (60 groupes) : dépôt aléatoire
0,5004 ± 0,0018 ; relaxation 0,2463 ± 0,0036 ; balistique 0,2825 ± 0,0028 ; Eden 0,3030 ± 0,0041. Un second simulateur
indépendant (autre générateur, autre code) donne 0,4998, 0,2461, 0,2830 et 0,3040.

**Valeur publiée non atteinte** : le balistique et Eden sont sous 1/3 (de 0,05 et de 0,03). Ce n'est pas une faute de règle ni de
mesure : Farnudi et Vvedensky publient β = 0,280 ± 0,002 pour le balistique à L = 1024 (0,302 à 4096, 0,308 à 8192, 0,334 à
524 288), contre 0,283 ici. L'exposant effectif ne monte vers 1/3 que lentement avec la taille. Les tests figent ce que le
modèle fait à cette taille et sur cette fenêtre, avec des tolérances de quatre à six écarts-types mesurés ; ils n'affirment
pas que β tende vers 1/3.

### Ce que cette étape n'établit pas

- Que β tende vers 1/3 : on l'a vu monter pour le balistique (0,283 à L = 1024, 0,307 à L = 8192 sur une fenêtre tardive),
  mais aucun test ne le vérifie ; pour Eden on ne l'a pas vu.
- Que la fenêtre soit dans le régime de croissance pur : elle déborde sur l'approche de la saturation (w² est déjà 10 % sous la
  valeur d'une bande huit fois plus large à t = 512).
- Que la convention de la relaxation soit exactement celle de Family (1986) : les trois conventions plausibles donnent 0,246,
  0,239 et 0,2425, toutes dans la tolérance ; seuls les tests exacts les distinguent.
- Le temps d'une implémentation honnête mais lente : la doc dit qu'un pas d'Eden doit coûter un temps constant ; aucun test
  ne borne le temps.

## Étape 1 : l'agrégat de Witten et Sander (DLA sur grille carrée)

Un germe occupe le site (0, 0) ; des marcheurs sont lancés un à un sur un anneau autour de l'amas, marchent au hasard sur la
grille, sont abandonnés s'ils s'éloignent trop, et collent au premier contact : dès qu'un des quatre voisins du marcheur est
dans l'amas, il s'arrête et son site s'ajoute. Les règles, le hasard consommé appel par appel (un tirage par lancement, un par
pas de marche) et l'écriture canonique de l'état sont dans la doc de `src/dla.rs` ; les mesures dans celle de `src/mesures.rs` ;
les valeurs attendues des tests, chacune avec sa source, dans `references/etape-1-valeurs.md`.

### Références

- T. A. Witten et L. M. Sander, *Diffusion-limited aggregation, a kinetic critical phenomenon*, Phys. Rev. Lett. 47, 1400
  (1981), et *Diffusion-limited aggregation*, Phys. Rev. B 27, 5686 (1983). Le texte intégral de l'article de 1981 n'a pas pu
  être lu ici : ce que le plan de ce dépôt disait de sa valeur de D (« un peu plus basse que 1,71 ») n'est pas vérifié, et
  aucun test ne s'y appuie.
- La convention de lancement à « rayon de l'amas + 5 » et d'abandon à deux fois le rayon est celle des premières simulations
  sur grille (Y. L. Loh, arXiv:1407.2586, 2014, qui cite Witten et Sander). Un abandon suivi d'un nouveau tirage uniforme sur
  l'anneau n'est pas le retour exact du marcheur : c'est une approximation connue, mesurée sans effet sur D ni sur β à
  l'erreur-type près (0,007 environ) à 4000, 10 000 et 30 000 sites.
- La valeur admise de la dimension fractale en 2D, environ 1,71, vient de grands amas : Meakin (1983), Tolman et Meakin
  (1989), Ball et Witten (1984), Sander (2000) ; Loh (2014) montre qu'au-delà d'environ 10⁵ sites l'anisotropie de la grille
  l'emporte. Meakin et Tolman et Meakin n'ont été lus que par leur résumé.
- Le rayon de giration croît comme N^(1/D) (Witten et Sander). Les dimensions de masse-rayon et de boîtes se mesurent sur des
  images synthétiques dont la réponse est connue (segment → 1, carré plein → 2, fractale de Vicsek → ln 5 / ln 3, tapis de
  Sierpinski → ln 8 / ln 3), d'où l'exactitude des mesures : les images se fabriquent dans les tests.

### Ce qui est mesuré, et l'écart à la valeur publiée

Sur 24 à 40 graines par taille, avec un simulateur écrit à part (autres structures de données, mêmes sites que la
bibliothèque aux réglages de la doc) ; chaque valeur est une moyenne ± son erreur-type :

| Taille de l'amas | masse-rayon, de 6 à R/2 | 1/β, exposant du rayon de giration |
|---|---|---|
| 4000 sites | 1,663 ± 0,007 | 1,697 |
| 10 000 sites | 1,682 ± 0,006 | 1,714 |
| 30 000 sites | 1,689 ± 0,006 | 1,723 |

**Valeur publiée non atteinte (masse-rayon)** : à 4000 sites la dimension de masse-rayon vaut 1,66, soit 0,04 sous 1,71. Ce n'est
pas une faute de règle : elle est la même, à l'erreur-type près, avec la règle de ce dépôt, un abandon cinq fois plus loin, une
marge quatre fois plus grande ou le retour exact sans abandon ; elle monte avec la taille de l'amas (effet de taille finie de
l'estimateur). L'exposant du rayon de giration, lui, donne 1/β entre 1,70 et 1,72 à toutes ces tailles, comme la valeur
publiée ; c'est le test « juste » de l'étape, le test de masse-rayon étant une « régression » de ce que fait ce modèle à 4000
sites. La dimension de boîtes de l'amas (environ 1,49) n'est que rapportée : aucune valeur publiée ne la fixe.

### Ce que cette étape n'établit pas

- Que la DLA sur grille carrée a la dimension 1,71 : l'exposant de giration la donne (1,70 à 1,72), la masse-rayon depuis le
  germe donne 1,66 à 1,69 et monte avec la taille. Au-delà d'environ 10⁵ sites l'anisotropie de la grille l'emporterait : rien
  n'est testé à ces tailles.
- Que l'abandon suivi d'une relance uniforme est sans effet : mesuré sans effet à l'erreur-type près ; un biais plus petit ne
  se verrait pas.
- Que le modèle est exactement celui de l'article de 1981 : son texte n'a pas été lu.
- Que la dimension de boîtes de l'amas a un sens : elle n'est que rapportée.
- Qu'une implémentation qui boucle sans fin échoue vite : elle n'échoue qu'au délai de la CI.
- Le comportement de `lancer` sur de très grandes boîtes (4095 × 4095, mémoire du réseau) : aucun test à cette taille.

## Étape 3 : les flocons de Reiter (modèle cellulaire local, grille hexagonale, virgule fixe)

Le modèle de C. A. Reiter, *A local cellular model for snow crystal growth*, Chaos, Solitons & Fractals 23 (2005) 1111-1119, sur
l'hexagone de `reseau` (coordonnées axiales), sans aucun hasard, en virgule fixe `Fixe` (Q31.32) : la symétrie d'ordre 6 (rotations
de 60° et réflexion) de tout l'état est donc exacte au bit près. Chaque cellule porte une masse d'eau ; elle est gelée si sa masse
vaut au moins 1, de bord si elle touche une cellule gelée, réceptive dans ces deux cas. Un pas : la masse des cellules non
réceptives diffuse (poids 1 − α/2 au centre, α/12 aux six voisines), celle des cellules réceptives ne diffuse pas vers ses voisines et reçoit γ (elle reçoit, comme toute cellule, la diffusion
venue de ses voisines non réceptives) ; le
bord est remis à β à chaque pas (la couronne extérieure de l'hexagone par défaut, ou le cercle de l'auteur avec `Bord::Circulaire`, voir « Le bord
circulaire » plus bas) ; l'état initial vaut β partout, sauf le germe à 1. Les règles, l'ordre et
l'arrondi de chaque opération en virgule fixe, l'écriture canonique et le contrat de `lancer` sont dans la doc de `src/flocon.rs` ;
les valeurs attendues des tests, chacune avec sa source, dans `references/etape-3-valeurs.md`.

### Références

- Reiter 2005 : préimpression (texte lu) et page auxiliaire de l'auteur, dont le script J `sfn.ijs` (page `sfn.html`) qui donne
  la table du temps de croissance en fonction de β et γ pour une région de rayon 100. Les règles de la doc sont celles du script (la table de
  240 cases est retrouvée exactement par un modèle écrit à part avec ces règles et la géométrie de l'auteur), sauf la géométrie du
  bord (voir ci-dessous) et les flottants (Q31.32 ici).
- « β = 0, γ = 0 : no growth » et « β = 0, γ > 0 : plates » (préimpression) : la plaque est un test « juste » (forme close).
- Gravner et Griffeath, Physica D 237 (2008) 385 (lu), reprennent le modèle ; Libbrecht 2005 et Gravner et Griffeath 2009 n'ont pas
  été relus : aucun test n'en dépend. Les figures de la préimpression n'ont pas été lues (des images).
- Le jeu « dendrite » (β = 0,4, γ = 0,001) est celui de la vignette `h0o4_x0o001` de la table de l'auteur et des exemples de sa page
  et de la figure 5 ; le jeu « étoile » (β = 0,45, γ = 0,1) est celui de la vignette `h0o45_x0o1` de la même table (une étoile
  pleine à six pointes), la préimpression ne disant que « stellar or more abstract forms » pour un γ élevé : le nom de classe
  « étoile » est le nôtre, pas celui de l'auteur. Les tests de classe de forme (compacité, pointes) et la vitesse radiale sont des
  « régressions », pas des preuves.

### Le bord circulaire (étape 3c)

Un flocon a un bord au choix (`Bord`) : `Hexagonal` (la couronne de l'hexagone, le défaut de `Flocon::new`, inchangé) ou `Circulaire`
(`Flocon::avec_bord`) : les cases de l'hexagone dont `4 · (q² + q r + r²) >= 3 R²`, c'est-à-dire hors du cercle de rayon `R sin 60°`, le bord du
script de l'auteur (`ocir` de `sfn.ijs` ; pour `R = 100` : `d² >= 7500`). L'entier `q² + q r + r²` est le carré exact de la distance euclidienne au
centre (deux cases voisines à 1) : tout reste en entiers, sans flottant ni racine, et la symétrie d'ordre 6 reste exacte au bit près. La couronne est toujours dans le bord circulaire
(une preuve de quelques lignes dans la doc de `src/flocon.rs`) ; le bord circulaire est la couronne pour `R <= 7` et en a davantage dès `R = 8` (54 cases contre 48).
`Flocon::distance2_cristal` est la mesure d'arrêt de l'auteur (`d² >= 7203` pour une région de rayon 100). L'écriture canonique garde l'étiquette `GIVRE003`
et change l'octet de forme : 1 (hexagonal, comme avant) ou 2 (circulaire). Seule la bibliothèque change : ni `lancer`, ni la vitrine, ni la page n'emploient encore le bord circulaire.
L'en-tête de `tests/etape_3_publie.rs` (gelé à l'étape 3, laissé tel quel) parle du seul bord hexagonal : les 59 cases qu'il écarte sont désormais testées
avec `Bord::Circulaire` par `tests/etape_3c_table.rs`.

### Ce qui est mesuré, et l'écart à la valeur publiée

**La table de Reiter (région de rayon 100, α = 1) est retrouvée en entier avec le bord circulaire : 240 cases sur 240** (`tests/etape_3c_table.rs`) : les 219 numéros
non plafonnés sont égaux, et pour les 21 cases plafonnées à 9999 aucun arrêt n'a lieu avant le 10000e pas (la table ne distingue pas un arrêt exactement au 10000e pas).
**Avant** (le bord hexagonal, l'arrêt à `d² >= 7203` ou au rayon hexagonal 85, c'est la même chose ici) : **181 sur 240** (160 non plafonnées et les 21 plafonnées) ; les 59
autres, les « dendrites lentes » (`β` de 0,05 à 0,5 et `γ <= 0,05`), ont 0,1 à 4,9 % de pas en plus (au pire `β = 0,2`, `γ = 0` : numéro 9435 contre 8994 publié). Le script J de l'auteur
(`sfn.ijs`) fixe à `β` les cases à distance euclidienne `d >= 100 sin 60° ≈ 86,6` du germe, un cercle à 1,6 case des pointes quand elles atteignent 85, alors que la couronne
de l'hexagone de rayon 100 en est à 15 cases : la vapeur maintenue à `β` arrive plus vite chez l'auteur. Ni une règle, ni un arrondi, ni la lecture du numéro n'y sont pour quelque chose.
Le bord hexagonal reste le défaut (rien de ce qui existait ne change, à l'octet) ; `Bord::Circulaire` est l'option qui retrouve la table.

### Ce que cette étape n'établit pas

- La table de Reiter de rayon 100 est retrouvée en entier avec `Bord::Circulaire` (240 cases sur 240, α = 1) ; le bord hexagonal, lui, n'en retrouve que 181. Mais :
  les 21 cases plafonnées à 9999 ne disent que « pas d'arrêt avant le 10000e pas » ; les tables de rayon 200 et 400 de l'auteur ne sont pas testées ; seule la
  bibliothèque a le bord circulaire (pas `lancer`, ni la vitrine, ni la page) ; la table complète (467 245 pas de simulation) est un test ignoré par défaut.
- Le script qui a produit la table (seul celui de l'animation est publié).
- Que les classes étoile, secteur et dendrite soient les « vraies » : aucune valeur publiée ne les chiffre.
- Les tables de rayon 200 et 400, et tout ce que montrent les figures de la préimpression, non lues.
- Le temps d'exécution sur la CI de GitHub (estimé, non mesuré).

## Étape 4 : attachement et détachement thermiques (le modèle « solide sur solide », Metropolis)

Le modèle SOS (« solid on solid », de Kossel) d'une face cristalline : une face carrée L × L périodique de hauteurs entières, de
hamiltonien H = J Σ |h_i − h_j| − Δμ Σ h_i (la somme des écarts de hauteur porte sur chaque paire de plus proches voisins une
fois ; Δμ est la différence de potentiel chimique qui pousse la croissance). Un essai : une colonne au hasard, « ajouter » ou
« ôter » une particule avec une chance sur deux, acceptation avec la probabilité min(1, exp(−ΔH/kT)) lue dans la table
`math::EXP_NEG` (J/kT et Δμ/kT en seizièmes entiers : aucune exponentielle dans le code). Le temps se compte en balayages (L²
essais). Les règles, le hasard consommé appel par appel, l'écriture canonique et le contrat de `lancer` sont dans la doc de
`src/cinetique.rs` ; les valeurs attendues des tests, chacune avec sa source, dans `references/etape-4-valeurs.md`.

**Ce n'est pas la cinétique de Gilmer et Bennema** : c'est un Metropolis à une particule, qui a le même état d'équilibre que leur
modèle mais pas leur loi de taux ; les vitesses de croissance de cette étape ne se comparent donc à aucune valeur publiée. Le nom
« Monte-Carlo cinétique » de la feuille de route doit se lire ainsi.

### Références

- M. Hasenbusch et K. Pinn, J. Phys. A 30, 63 (1997), arXiv cond-mat/9605019 (lu) : le modèle SOS absolu (ASOS), H = K Σ |h_x − h_y|
  sur les paires de plus proches voisins, la température de transition rugueuse K_R = 0,80608(2), la largeur qui diverge en ln L du
  côté rugueux et reste finie du côté lisse ; arXiv hep-lat/9710079 (lu) recoupe la convention (J par liaison, chaque liaison une fois).
- La pente 1/π² de la largeur contre ln L à la transition : Hasenbusch, Meyer et Pütz, arXiv hep-lat/9601011 (lu, pour l'interface
  d'Ising 3D ; appliquée au SOS par universalité, ce que la fiche dit).
- Libbrecht, Rep. Prog. Phys. 68, 855 (2005) (texte lu) : la vitesse v = α v_cin σ ; une face rugueuse a α ≈ 1, une face lisse croît
  par nucléation bidimensionnelle, α ≈ A exp(−σ_crit/σ). Gilmer et Bennema, J. Appl. Phys. 43, 1347 (1972) : résumé seul. Weeks et
  Gilmer (1979), Burton, Cabrera et Frank (1951) et van Beijeren (1977) n'ont pas été lus.
- K_R = 0,806 vaut environ 12,9 seizièmes : les faces dites lisses des tests (J/kT = 1, 1,25, 1,5, 2) sont du côté lisse, les rugueuses
  (J/kT = 0,375, 0,5, 0,75) du côté rugueux.

### Ce que les tests établissent, et ce qu'ils n'établissent pas

Sont « justes » : le simulateur naïf écrit dans les tests (mêmes hauteurs, au bit) ; le cas J = 0, où la vitesse moyenne par balayage
vaut exactement (1 − p)/2 avec p = EXP_NEG[Δμ en seizièmes] / 2³² ; la distribution de Boltzmann exacte d'une face 3 × 3 (par
énumération) ; la pente de w² contre ln L, qui passe par 1/π² entre J/kT = 0,75 et 0,875 : le modèle encadre K_R = 0,806 au seizième
près (avec 12 à 29 % d'excès de pente à J/kT = 0,8125, des corrections en 1/ln L que la source annonce). Sont des « régressions »
(la forme décrite par les sources, à des tailles et des durées écrites) : la nucléation à bas T, la linéarité à haut T, la
rugosité qui croît avec T.

- Aucune vitesse de croissance publiée n'est reproduite, et aucune ne peut l'être avec cette dynamique : seules les propriétés
  d'équilibre (Boltzmann sur 3 × 3, K_R, la pente 1/π²) se comparent à la littérature.
- K_R n'est encadré qu'au seizième (0,75 < K_R < 0,875) ; le mesurer demanderait une autre grille de J ou de bien plus grandes faces.
- Il n'y a pas de seuil de nucléation au sens strict : « quasi nulle » dépend de la taille et de la durée ; ni la loi exp(−σ_crit/Δμ)
  ni σ_crit ne sont testées.
- La linéarité à haut T est mesurée à un seul J, une seule taille, sur une plage de Δμ ; la mobilité n'est comparée à rien de publié.
- L'égalité entre Linux et Windows est établie sur 40 faces et sur la suite entière ; la CI la rejoue par la référence d'empreinte.

## Étape 2d : les variantes de la DLA (probabilité de collage et réduction du bruit)

Deux variantes de la DLA de l'étape 1, chacune d'un paramètre, dans `src/dla_variantes.rs` : le **collage partiel** (un marcheur
qui touche l'amas ne colle qu'avec la probabilité p) et la **réduction du bruit** (un site au contact de l'amas ne se remplit
qu'après avoir été touché m fois ; m = 1 est la DLA de l'étape 1, et p = 1 aussi : `AmasVariante` redonne alors `Amas` site pour
site). La dynamique, le hasard consommé appel par appel, l'écriture canonique et le contrat de `lancer` sont dans la doc de
`src/dla_variantes.rs` ; les valeurs attendues des tests, chacune avec sa source, dans `references/etape-2d-valeurs.md`. Deux
mesures propres aux variantes : la densité N / Rg² et l'anisotropie S4 = Σ (x⁴ − 6x²y² + y⁴) / Σ (x² + y²)² autour du germe (+1
pour des bras sur les axes, −1 sur les diagonales, 0 pour un disque).

### Références

- **Collage partiel** : Witten et Sander, Phys. Rev. B 27, 5686 (1983), p. 5691 (texte lu) : le marcheur colle « only during a
  fraction s of its encounters », 1/s joue le rôle d'une longueur capillaire, et la figure 5 montre l'épaississement des branches
  pour s = 0,5. **La règle exacte sur réseau carré n'est écrite nulle part** : celle de ce dépôt (le pas vers l'amas refusé, la
  position inchangée, un nouveau tirage au contact suivant) est une convention, différente dans le détail du modèle à une
  dimension de leur équation 11.
- **Réduction du bruit** : Tang, Phys. Rev. A 31, 1977 (1985), Kertész et Vicsek, J. Phys. A 19, L257 (1986), Nittmann et Stanley,
  Nature 321, 663 (1986) n'ont pas été lus ; la règle de ce dépôt (un compteur par site de périmètre, qui persiste ; le marcheur
  qui touche est retiré et un nouveau est lancé ; le site se remplit quand son compteur atteint m) est celle que décrivent Meakin
  (dans Pietronero, *Fractals' Physical Origin and Properties*, extrait lu) et Stanley. Une autre variante, où tous les compteurs
  repartent à zéro après chaque croissance, est attribuée à Kertész et Vicsek par une seule source secondaire (Lam) : c'est un
  autre modèle, que la doc nomme et exclut.
- Alves et Ferreira (2006, texte lu) : l'anisotropie axiale augmente avec le paramètre de réduction du bruit ; leur figure 1 montre
  aussi que, dans une même boîte, moins de particules s'agrègent quand le bruit est plus réduit (la densité baisse).

### Ce qui est mesuré

Densité N / Rg² (N = 1000) : 1,31, 1,61, 2,16 et 3,42 pour p = 1, 0,5, 0,2 et 0,05 : l'amas est plus dense quand le collage baisse.
Anisotropie S4 (N = 400) : 0,15, 0,47, 0,74 et 0,89 pour m = 1, 2, 4 et 8 : elle croît avec la réduction du bruit, jusqu'à
des bras presque tous sur les axes. Ces deux tendances sont des « régressions » : la littérature en publie le sens, aucun nombre à
ces tailles. Constats non testés : le collage partiel épaissit l'amas sans accroître S4 (0,16, 0,21, 0,16 puis 0,09 pour p = 1,
0,5, 0,2 et 0,05, à N = 1000 sur 200 graines) ; la densité ne croît pas avec m (1,56, 1,67, 1,61 et 1,44 pour m = 1, 2, 4 et 8).

### Ce que cette étape n'établit pas

- Qu'une valeur publiée est reproduite : aucune ; les deux tendances ne sont que des directions publiées.
- Que la règle du bruit est mot pour mot celle de Tang ou de Nittmann et Stanley : elle est celle que décrivent Meakin et Stanley.
- Que la variante de Kertész et Vicsek (remise à zéro globale, si c'est bien la leur) est couverte : c'est un autre modèle.
- Que le collage partiel suit Witten et Sander dans le détail : le pas refusé est une convention.
- Que le collage partiel mène aux quatre bras : à N = 1000 il n'accroît pas S4 ; à très grand N, non mesuré.
- Le contrat de `lancer` pour des scénarios qui mêlent `surface` et les directives de la DLA : seuls trois cas de refus le fixent.
- Qu'une implémentation qui boucle échoue vite : elle échoue au bout de 120 s (le délai des simulations des tests).

## Étape 5 : la vitrine (la DLA et les flocons de Reiter dans une page web, en WebAssembly)

La vitrine fait tourner dans un navigateur le code des étapes 1 et 3, sans le réécrire : `src/vitrine.rs` est compilé deux fois,
en natif (les tests l'appellent comme n'importe quelle fonction Rust) et en WebAssembly (`docs/givre.wasm`, cible
`wasm32-unknown-unknown`, module sans importation). Onze fonctions exportées `givre_*`, qui ne prennent et ne rendent que des
nombres : démarrer l'un de quatre modèles (0 la DLA, 1 la plaque, 2 l'étoile, 3 la dendrite, avec les réglages des scénarios
d'exemple), avancer, lire le temps, la durée, la taille de l'image et ses pixels (`0x00RRGGBB`, `u32::MAX` hors de l'image), le
nombre de sites, le rayon, la compacité et les huit mots de l'empreinte. La page (`docs/index.html`, `docs/app.js`,
`docs/lecture.js`) les appelle, dessine l'image sur un canvas et affiche l'empreinte. Les règles, le temps, l'état avant tout
démarrage et les valeurs « hors de portée » sont dans la doc de `src/vitrine.rs` ; les valeurs attendues des tests, chacune avec
sa source, dans `references/etape-5-valeurs.md`.

La preuve de l'étape est la phrase de la feuille de route : **l'empreinte d'une graine, lue dans la page, est celle du code
natif**. Pour la DLA de graine 1 jouée jusqu'au bout, c'est le contenu de `references/dla-graine-1.empreinte`.

### Références

- MDN, `WebAssembly.instantiate()` (lu) : `instantiate(bufferSource, importObject)` rend `{ module, instance }` ; l'objet
  d'importations est facultatif ; une importation non fournie donne `LinkError`. MDN, `ImageData()` (lu) : `new ImageData(dataArray,
  width, height)` attend un `Uint8ClampedArray` de `4 × largeur × hauteur` octets.
- La spécification de l'interface JavaScript de WebAssembly (lue) : un `i32` rendu est lu comme un nombre signé (d'où le
  `>>> 0` qui le relit non signé) ; un `i64` serait un `BigInt` ; un `f64` NaN devient `NaN`. Le format binaire du module (lu) :
  `\0asm`, version 1, sections, exportations ; c'est ce que lit le petit lecteur de `tests/etape_5_commun/mod.rs`.
- La référence de Rust (lue) : `#[unsafe(no_mangle)]` est obligatoire dès l'édition 2024 ; une panique qui atteint une fonction
  `extern "C"` devient un abandon du processus : aucune fonction de la vitrine ne panique, en natif comme en WebAssembly. Node,
  « process I/O » (lu) : les écritures sur un tube sont asynchrones sous POSIX et synchrones sous Windows, d'où `process.exitCode`
  dans les scripts de test. Non lu : un exemple officiel de `cdylib` pour `wasm32-unknown-unknown` (la commande de construction
  est éprouvée par six reconstructions identiques).

### Ce que les tests établissent

Tout est comparé à l'état du code natif par une **égalité exacte** (entiers, octets, mots, bits de `f64`) ou à un calcul fermé
(couleurs d'un flocon au temps 0, tailles d'image, indépendants du code) : aucune tolérance statistique, aucun hasard à mesurer.
Le module commité, lu sous Node, redonne l'empreinte, les accesseurs et l'image du code natif (huit cas dont des morceaux
inégaux, la DLA de graine 1 jusqu'au bout, un morceau qui dépasse la durée) ; `docs/lecture.js` (l'empreinte affichée, l'image
dessinée, la graine du champ) est éprouvé sous Node contre le natif. Ce qu'aucun test ne joue mais que le juge a mesuré : le module se reconstruit identique à l'octet par la commande de la
doc (six constructions, trois dossiers de travail, deux de cible).

### Ce que cette étape n'établit pas

- **Que la page marche dans un navigateur.** `lecture.js` est éprouvé sous Node contre le natif ; le reste de la page (que
  l'empreinte soit bien écrite dans `#empreinte`, que l'image soit bien dessinée, que la graine du champ soit passée, l'animation,
  le curseur de vitesse, l'arrêt au bout) n'est éprouvé que par sa forme (des tests de texte, étiquetés `forme`) et par un coup
  d'œil humain, une fois, dans Chromium. Jamais Firefox, Safari ni un téléphone.
- **Que `docs/givre.wasm` soit le module de la dernière source** : les tests comparent son **comportement** au code natif, pas
  ses octets. Un changement voulu de l'état ou des couleurs de `dla`, `flocon` ou `vitrine` rend le test du module rouge tant
  qu'il n'est pas reconstruit ; un changement sans effet de comportement laisse un module différent à l'octet mais égal par le
  comportement.
- **Que la construction soit reproductible ailleurs que sur la machine virtuelle** (six constructions identiques, chaîne 1.99.1) ni
  que le module se comporte de même sur un autre moteur WebAssembly que Node 20 (V8).
- **Node sous Windows** : les scripts ont tourné sous Linux ; la CI de `windows-latest` est le premier essai.
- **Un amas de DLA qui sortirait du réseau** (`givre_avancer` s'arrête alors sans panique ; impossible pour 3000 sites), **l'état
  avant tout démarrage en natif** (l'état est global : seul Node, sur un module neuf, le lit), **un accesseur de largeur échangé avec
  celui de hauteur** (les images sont carrées), **les performances sur un téléphone**.
- **La justesse physique des images** : celle des étapes 1 et 3, reprise telle quelle.

## Étape Z1 : la corrélation densité-densité (la troisième mesure de D, et le zinc en repère)

La DLA de l'étape 1 se mesure par la masse-rayon et par le rayon de giration. Cette étape ajoute la mesure de Matsushita et coll. : la **corrélation
densité-densité** `C(r)`. Pour un amas de `n` sites, `C(r)` est le nombre de paires ordonnées de sites à distance `<= r`, divisé par `n` et par le nombre de
cases d'un disque plein de rayon `r`, centre exclu ; elle vaut exactement 1 sur un plan plein et varie comme `r^(D-2)` pour un amas fractal. La pente de `ln C`
contre `ln r`, plus 2, donne `D`. Tout le comptage se fait en entiers (distances au carré, comptes en `u64`, dénominateur en `u128`) ; les seules fonctions de
`f64` sont `+ - * /`, `sqrt` et `math::logarithme` : **la règle des nombres d'`AGENTS.md` ne change pas**. Le module est `src/correlation.rs` (fonctionnalité
`etape-z1`) ; les règles, la formule et l'ordre exact des opérations sont dans sa doc ; la page (bouton « Mesurer D », courbe log-log, repère du zinc) et les
quatre fonctions de mesure de `src/vitrine.rs` sont décrites dans la doc de ce fichier et dans `tests/etape_z1_page.rs` ; les valeurs attendues des tests,
chacune avec sa source, dans `references/etape-z1-valeurs.md`.

La **forme cumulée** (paires à distance `<= r`) est celle du module. Matsushita et coll. comptent, d'après leur résumé, « la corrélation densité-densité » des
photographies numérisées sans que la forme exacte (par couronne ou cumulée) ait été lue. Sur le prototype, la forme par couronne (paires à distance dans
`]r-1, r]`, divisées par les cases de la couronne) est **moins bonne** sur une grille carrée : sur sept formes à solution connue l'erreur absolue à la dimension
exacte est en moyenne de 0,045 pour la forme cumulée et de 0,077 pour la couronne (la diagonale donne même `NaN` à la couronne : des couronnes sans aucune
paire), et sur la DLA l'écart-type d'une graine est de 0,018 contre 0,029 à 4000 sites et de 0,013 contre 0,024 à 10 000 sites (30 graines). La pente moyenne est
la même à 0,01 près (1,645 contre 1,644 à 4000 sites ; 1,653 contre 1,659 à 10 000).

### La fenêtre d'ajustement

La pente se mesure sur **tous les rayons entiers de 4 à `étendue / 8`** (division entière ; l'étendue est le plus long côté de la boîte qui contient l'amas, en
nombre de pas), au moins 6 rayons (donc une étendue d'au moins 72 : pour la DLA de 465 à 774 sites). Elle ne dépend que de l'étendue de l'amas : ni de l'ordre
des sites, ni de leur position, ni de la valeur que la mesure donne. C'est la fenêtre que le plan suggérait (de 4 au quart du rayon de l'amas environ) ; **ce
n'est pas la meilleure sur les formes** (voir la table), seulement dans le palier des fenêtres raisonnables.

Sensibilité mesurée sur le prototype (dimension pour chaque fenêtre ; les cinq formes sont des images à solution connue, la DLA est la moyenne de 120 graines
à 4000 sites et de 90 graines à 10 000 sites) :

| fenêtre | trait de 243 sites (1) | carré plein 243² (2) | Vicsek en croix, niveau 5 (1,465) | Vicsek en X, niveau 5 (1,465) | tapis, niveau 5 (1,893) | DLA 4000 sites | DLA 10 000 sites |
|---|---|---|---|---|---|---|---|
| 4 à `étendue / 12` | 0,967 | 1,964 | 1,472 | 1,548 | 1,850 | 1,639 | 1,649 |
| 4 à `étendue / 10` | 0,966 | 1,959 | 1,451 | 1,532 | 1,848 | 1,643 | 1,654 |
| **4 à `étendue / 8` (la fenêtre de la doc)** | 0,962 | 1,951 | 1,444 | 1,509 | 1,841 | **1,649** | **1,661** |
| 4 à `étendue / 6` | 0,956 | 1,939 | 1,465 | 1,476 | 1,824 | 1,655 | 1,667 |
| 4 à `étendue / 4` | 0,943 | 1,916 | 1,456 | 1,488 | 1,799 | 1,657 | 1,670 |
| 4 à `étendue / 3` | 0,930 | 1,892 | 1,426 | 1,476 | 1,780 | 1,647 | 1,661 |
| 6 à `étendue / 2` | 0,893 | 1,827 | 1,423 | 1,423 | 1,712 | 1,596 | 1,615 |
| 2 à `étendue / 8` | 0,966 | 1,960 | 1,459 | 1,488 | 1,850 | 1,638 | 1,651 |
| 6 à `étendue / 8` | 0,956 | 1,945 | 1,441 | 1,468 | 1,832 | 1,655 | 1,667 |
| 8 à `étendue / 8` | 0,955 | 1,939 | 1,432 | 1,484 | 1,827 | 1,660 | 1,672 |

Lecture : tant que le haut de la fenêtre est entre `étendue / 12` et `étendue / 4` et le bas entre 2 et 8, la DLA donne de 1,638 à 1,660 à 4000 sites et de 1,649 à
1,672 à 10 000 sites (une dispersion de 0,02 ; l'écart-type d'une graine est 0,015 à 0,02). Une fenêtre qui monte jusqu'à `étendue / 2` mord sur le bord de l'amas
et fait chuter la valeur de 0,05 (1,596) : c'est l'effet de bord, présent aussi sur le carré plein (1,83 au lieu de 2). Sur les formes, l'écart moyen à la
dimension exacte (13 formes pour les fenêtres qui existent sur toutes) est 0,044 pour la fenêtre de la doc, 0,039 pour `4 à étendue / 6`, 0,031 pour
`2 à étendue / 8` et 0,109 pour `6 à étendue / 2`.

### Les trois mesures de la dimension de la DLA, avant et après

Avant l'étape : deux mesures (masse-rayon depuis le germe sur les rayons 6 à `R / 2`, et `1/β` du rayon de giration). Après : trois. Moyennes sur 12 à 120 graines (Rust,
amas de la bibliothèque, `Amas::new(graine, 400)` ou 800), avec l'écart-type d'UNE graine entre parenthèses :

| taille (graines) | corrélation (nouvelle) | masse-rayon | 1/β (giration) |
|---|---|---|---|
| 4000 sites (120) | **1,6486** (0,022) | 1,6737 (0,047) | 1,7101 (0,060) |
| 10 000 sites (120) | **1,6587** (0,015) | 1,6803 (0,035) | 1,7127 (0,054) |
| 30 000 sites (12) | **1,6665** | 1,6828 | 1,7079 |

Les trois s'ordonnent `corrélation < masse-rayon < 1/β` à chaque taille, dans tous les blocs de 24 graines essayés, avec des écarts moyens de 0,02 à 0,03 (corrélation
et masse-rayon) et de 0,05 à 0,06 (corrélation et `1/β`) ; la corrélation est la plus régulière d'une graine à l'autre. Elles s'accordent donc à un dixième près, pas
à la précision d'une graine ni au centième. Les mesures de l'étape 1 sont retrouvées (la masse-rayon à 4000 sites : 1,6737 sur 120 graines contre 1,670 sur 250).

### Les valeurs publiées, dites telles quelles

- **1,713 ± 0,003** (B. Davidovitch, A. Levermann et I. Procaccia, arXiv:cond-mat/0008053, 2000 ; Phys. Rev. E 62, R5919) est la dimension asymptotique de la DLA
  **calculée** par une renormalisation du premier coefficient de Laurent d'une application conforme, sur de petits amas de 1 à 20 particules (le texte donne ±0,005 pour
  sa forme révisée) : ce n'est ni une mesure de corrélation ni une mesure de masse sur un grand amas. La corrélation du banc est **0,064 en dessous** à 4000 sites
  (1,649), 0,054 à 10 000 (1,659) et 0,046 à 30 000 sites (1,667) : elle monte lentement avec la taille et **n'atteint pas** 1,713, à aucune des tailles essayées. On dit
  « compatible avec une dimension qui monte vers 1,7 », jamais « retrouvé ». Aucun test ne s'appuie sur cette valeur.
- **1,66 ± 0,03** (M. Matsushita, M. Sano, Y. Hayakawa, H. Honjo et Y. Sawada, Phys. Rev. Lett. 53, 286, 1984, feuilles de zinc électrodéposées, « dimension de
  Hausdorff moyennée sur de nombreux exemples », mesurée par la corrélation densité-densité des photographies numérisées) est un **repère**, jamais un test : à 4000 et
  à 10 000 sites la corrélation du banc (1,65 et 1,66) tombe dans la barre `1,63 à 1,69`, et la masse-rayon aussi (1,67 et 1,68), alors que `1/β` (1,71) est au-dessus. Cela ne
  prouve rien : le texte intégral de l'article n'a pas été ouvert, **la taille d'amas du zinc n'a pas été relevée** (la page dit « taille d'amas du zinc non relevée »), pas
  plus que la fenêtre de `r` du 1,66 ni la forme de `C(r)` ; et un amas de zinc n'est pas un amas de DLA sur grille carrée. Après une mesure, la page dit « compatible, ce qui ne prouve pas
  que le zinc suit la DLA » si `|D - 1,66| <= 0,03`, et « hors de la barre du zinc » sinon ; avant toute mesure, ni l'une ni l'autre.

### Le plafond de la page

La mesure est en `O(n²)`. Temps natif (profil `release`, une graine, le meilleur de trois) : 0,008 s à 2000 sites, 0,018 s à 3000, 0,025 s à 4000, 0,040 s à 5000, 0,17 s à 10 000,
0,69 s à 20 000, 1,67 s à 30 000. La DLA de la page s'arrête à 3000 sites : le plafond (`vitrine::MESURE_SITES_MAX = 10 000`) n'est atteint par aucun modèle de la page ; il garde la
page vive si un modèle plus grand s'y ajoute. Le module WebAssembly est plus lent d'un facteur **non mesuré** (ni Node ni la cible `wasm32-unknown-unknown` n'étaient disponibles) ; le
plafond se relève sur une mesure du module.

### Références

- M. Matsushita, M. Sano, Y. Hayakawa, H. Honjo et Y. Sawada, *Fractal Structures of Zinc Metal Leaves Grown by Electrodeposition*, Phys. Rev. Lett. 53, 286 (1984) : le
  résumé de l'éditeur seulement (lu) : 1,66 ± 0,03, corrélation densité-densité des photographies numérisées, accord avec la DLA 2D.
- B. Davidovitch, A. Levermann et I. Procaccia, *Convergent Calculation of the Asymptotic Dimension of Diffusion Limited Aggregates: Scaling and Renormalization of Small
  Clusters*, arXiv:cond-mat/0008053 (2000) : lu (résumé et texte) ; le détail des équations n'est pas relu.
- OEIS A000328, « Number of points of norm <= n^2 in square lattice » (lue) : le nombre de cases du disque plein, bord compris.
- Les dimensions de similarité de la fractale de Vicsek (`ln 5 / ln 3`) et du tapis de Sierpinski (`ln 8 / ln 3`) sont celles de l'étape 1 (Wikipedia, non relues ici).

### Ce que cette étape n'établit pas

- **Que la DLA a la dimension 1,713, ou que son amas du banc la mesure** : la corrélation donne 1,65 à 1,67 de 4000 à 30 000 sites et monte lentement ; rien n'est testé au-delà.
  Les trois mesures s'accordent à un dixième près, pas davantage : le test est un contrôle croisé (une « régression » de ce modèle), pas une preuve de la dimension.
- **Que le zinc suit la DLA** : le repère 1,66 est un résumé d'article ; sa taille d'amas, sa fenêtre et la forme exacte de sa corrélation n'ont pas été lues, et aucun test
  n'y compare le banc.
- **Que la fenêtre `4 à étendue / 8` soit la bonne** : elle est dans un palier (de 1,64 à 1,67 pour la DLA, de 0,03 à 0,045 d'écart aux formes) mais n'est pas la meilleure sur les
  formes (`2 à étendue / 8` fait mieux) ; elle ne se règle pas sur la DLA.
- **Que la forme cumulée soit celle de Matsushita** (non lu), ni que la mesure vaille au-delà de la grille carrée (la DLA sur grille carrée devient anisotrope au-delà d'environ 10⁵
  sites, Loh 2014 ; non essayé).
- **Que la mesure marche dans le module WebAssembly et dans la page** : le module n'a pas pu être reconstruit ici ; les tests de Node (`tests/etape_z1/correlation-wasm.mjs`) n'ont
  pas été exécutés ; la page (bouton, courbe, barre) n'a pas été vue dans un navigateur ; Firefox, Safari et un téléphone : jamais essayés. Les tests de forme ne disent pas que le
  DOM est bien relié.
- **Le temps du module** (non mesuré) ; **un amas au-delà du plafond dans la page** (la DLA de la vitrine s'arrête à 3000 sites : aucun test ne l'atteint).
- Que la mesure est sûre pour des coordonnées de l'ordre de `2^31` en dehors des cas testés (deux sites aux extrémités de l'`i32`).

## Étape D1 : le claquage diélectrique (le champ de Laplace et la croissance en `φ^η`)

Le modèle de **claquage diélectrique** de L. Niemeyer, L. Pietronero et H. J. Wiesmann (Phys. Rev. Lett. 52, 1033, 1984) : le potentiel `φ` autour d'un amas vérifie l'équation de
Laplace (`φ = 0` sur l'amas, `φ = 1` sur une frontière extérieure, chaque site libre est la moyenne de ses voisins) ; l'amas croît d'un site voisin tiré avec une chance
proportionnelle à `φ^η`, `η` entier. `η = 0` : toutes les candidates à égalité, un amas plein (modèle d'Eden) ; `η = 1` : la chance de coller est le potentiel (dans le continu, la
probabilité qu'un marcheur arrive là en premier ; sur la grille, ce n'est pas la DLA de l'étape 1 : voir plus bas) ; plus `η` monte, plus l'amas est fin. Le module est `src/claquage.rs` (fonctionnalité `etape-d1`) ; la dynamique, la consommation du
hasard appel par appel, l'écriture canonique et la directive `claquage <eta> <particules>` y sont écrites, et une implémentation fidèle donne EXACTEMENT les mêmes sites, les mêmes
potentiels (bit pour bit) et le même générateur. Les valeurs attendues des tests, chacune avec sa source, sont dans `references/etape-d1-valeurs.md`. **La règle des nombres
d'`AGENTS.md` ne change pas** : le potentiel est un `f64` calculé avec `+ - * /`, `abs`, `max` et `clamp` ; `φ^η` par multiplications répétées (jamais `powi`) ; les comptages et
les coordonnées en entiers ; aucune `HashMap`.

**La méthode de 1984 (grille, bord, calcul du potentiel) n'a pas été lue** (le résumé de l'éditeur est lu et n'en dit rien ; le texte intégral est derrière un accès payant). Les choix
ci-dessous sont donc **à nous**, et aucune valeur publiée n'en dépend à cette étape : la courbe `D(η)` contre Hastings (2001) est l'étape D2, qui ne se lit pas ici.

### Les choix à nous

| Choix | Valeur | Pourquoi |
|---|---|---|
| frontière | le cercle `x² + y² > d²` (`d` = `demi_cote`), `φ = 1` ; l'amas qui la touche s'arrête | une électrode extérieure ronde, sans effet de coin ; le germe seul est à égale distance de tous ses bords |
| voisinage | quatre cases (jamais les diagonales) | comme la DLA de l'étape 1 |
| calcul du champ | sur-relaxation **rouge-noir**, `ω = 2 - 6 / (d + 6)`, bornée à `[0, 1]` (`clamp`), arrêtée à un écart de passe `< 2^-20`, départ = le champ du pas précédent (`1` au départ) | voir « Pourquoi une sur-relaxation rouge-noir » |
| garde | une relaxation qui n'a pas convergé après `PASSES_MAX = 100 000` passes panique (mesuré : 354 passes à froid à `d = 60`, 1316 à `d = 255`) | un code qui ne converge pas (un voisin diagonal) échoue au lieu de tourner sans fin ; aucun bit ne change |
| `η = 0` | aucun champ n'est calculé (`potentiel` rend `None`) | les poids sont tous 1 : le résultat n'en dépend pas ; coût nul |
| candidates | les sites libres touchant l'amas par un côté, dans l'ordre `y` croissant puis `x` croissant | l'ordre fixe le hasard |
| tirage | un seul `suivant_f64` par site ajouté, `u = t W`, première somme cumulée `> u` | une règle sans cas du dernier site (`u < W` toujours) |

**Quand l'amas s'arrête** : dès qu'un de ses sites a un voisin hors du domaine (la décharge a rejoint l'autre électrode) ; ce site est ajouté, le pas suivant rend `None`. Un `None` ne
consomme aucun tirage.

### Pourquoi une sur-relaxation rouge-noir (la suggestion du brief, Jacobi, est fausse)

Le brief suggérait une relaxation de Jacobi. **Mesurée, elle est trop chère** : un balayage de Gauss-Seidel (le schéma voisin, deux fois plus rapide que Jacobi) demande **4358 passes
par site ajouté** à `d = 60` (300 sites, graine 1, seuil `2^-30` : 1,3 million de passes, 61 s) parce que le mode lent du domaine, qu'un site de plus excite à chaque pas, se
relaxe en `d²` passes ; Jacobi n'a pas été mesuré jusqu'au bout (arrêté après une minute, durée que Gauss-Seidel met à finir un amas). La sur-relaxation rouge-noir (`ω` proche de 2) le fait en `d`
passes : 272 passes par site au même seuil (avec `ω = 2 - 6 / d` à l'époque), soit **16 fois moins**, et un balayage sans dépendance entre les sites d'une même couleur (l'ordre à l'intérieur d'une couleur ne change aucun
bit : une main peut la découper à sa guise). Elle reste un schéma que le test naïf écrit en dix lignes et qui donne les mêmes bits. Les itérés peuvent sortir de `[0, 1]` quand
`ω > 1` : le `clamp` y remédie sans changer la limite (le champ rendu vérifie `0 <= φ <= 1`, ce qu'un test lit).

`ω = 2 - 6 / (d + 6)` : un facteur trop bas coûte beaucoup plus qu'un facteur trop haut. Passes par site ajouté (graine 1, 300 sites, seuil `2^-20`, `ω = 2 - a / d`) :

| `a` | 2,5 | 3,5 | 5 | 7 | 9 | 12 | 16 |
|---|---|---|---|---|---|---|---|
| `d = 60` | 187 | 135 | **104** | 124 | 155 | 199 | |
| `d = 100` | | | **164** | | 194 (`a = 8`) | 258 | 315 |

La formule retenue vaut `a ≈ 5,5` à `d = 60` et `a ≈ 5,7` à `d = 100`, un peu au-dessus de l'optimum (`ω` optimal `≈ 2 - 3,4 / R` sur un disque de rayon `R`, et plus bas encore quand l'amas
réduit la couronne). Elle ne dépend de rien d'autre que de `d` et vaut 1 pour `d = 0`.

### Le coût (mesuré)

Temps natif, un fil, un processus, profil `dev` à `opt-level = 3` (Ryzen 5 5500U), une graine (la 1), `η = 1` sauf mention, `d` à peu près 2,5 fois le rayon final :

| `η` | sites | `d` | rayon final | temps total | par site ajouté |
|---|---|---|---|---|---|
| 1 | 200 | 50 | 19 | 0,47 s | 2,3 ms |
| 1 | 500 | 85 | 30 | 4,6 s | 9,1 ms |
| 1 | 1000 | 110 | 45 | 17 s | 17 ms |
| 1 | 2000 | 150 | 74 | 87 s | 44 ms |
| 1 | 3000 | 185 | 83 | 211 s | 70 ms |
| 0 | 1000 | 60 | 22 | 0,03 s | 0,03 ms |
| 2 | 1000 | 200 | 74 | 98 s | 98 ms |
| 3 | 1000 | 255 | 108 | 210 s | 210 ms |

Le coût suit `d³ × sites` (chaque pas relaxe les `π d²` sites en un nombre de passes d'ordre `d`, une centaine de passes à `d = 45`) : **il domine tout**, et il est **de deux à trois ordres de grandeur
au-dessus d'une marche de DLA** (la DLA de 300 sites : 0,005 s, de 1000 sites : 0,05 s). Il fixe les tailles des tests : de 100 à 300 sites pour `η` = 1, 2, 3 (`d = 40` à 45 : 0,25 à 0,4 s par amas) et 2000 sites à `η = 0`.
**À ces tailles la dimension est bien en dessous de sa valeur limite** (la DLA mesurée par la corrélation y est à 1,60 à 300 sites, contre 1,65 à 4000) : les tests de forme ne prouvent que l'ordre
et la compacité, pas une dimension. Un amas fin (`η = 3`) de 1000 sites atteint déjà le rayon 108 (il tient dans `d = 255` ; au-delà de quelques milliers de sites, extrapolation non mesurée, il n'y tiendrait plus) ; la
courbe `D(η)` (D2) coûtera des minutes par amas dans la page et dans les tests.
**Les passes par site** à `η = 1`, 300 sites, `d = 45` : 80 (seuil `2^-20`) ; à `2^-14`, `2^-17`, `2^-24` et `2^-30` : 39, 59, 111 et 161.

### La sensibilité aux choix à nous (mesurée, `η = 1`)

**Le rayon de la frontière** (300 sites ; la DLA de même taille a un rayon moyen de 27,6 ; corrélation sur les rayons 2 à 8 ; 60 graines sauf mention) :

| `d` | 41 | 45 (120 graines) | 55 | 60 | 82 | 110 |
|---|---|---|---|---|---|---|
| `d` / rayon DLA | 1,5 | 1,6 | 2,0 | 2,2 | 3,0 | 4,0 |
| corrélation | 1,679 | 1,677 | 1,680 | 1,674 | 1,669 | 1,671 |
| masse-rayon | 1,630 | 1,614 | 1,658 | 1,646 | 1,641 | 1,618 |
| `1/β` | 1,820 | 1,800 | 1,833 | 1,812 | 1,807 | 1,804 |
| rayon moyen de l'amas | 23,9 | 23,5 | 23,4 | 23,6 | 23,2 | 23,6 |
| temps par amas (8 fils en même temps) | 0,6 s | 0,8 s | 1,4 s | 1,9 s | 4,6 s | 16 s |

Les trois estimateurs ne bougent pas avec `d` au-delà du bruit (l'écart-type de la moyenne de 60 graines est `0,005` pour la corrélation, `0,02` pour la masse-rayon et `0,02` pour `1/β`) :
de 1,5 à 4 fois le rayon DLA, la corrélation reste entre 1,669 et 1,680. On prend **`d = 45`** pour les tests (0,4 s par amas). Pour `η` plus grand, l'amas est plus étendu : `d` doit suivre (le
rayon passe de 13 à 23 de `η = 1` à `η = 3` à 100 sites).

**Le seuil d'arrêt** (24 graines, 300 sites, `d = 45` ; outil `d1_schema.rs`, qui refait le schéma de la doc et dont l'amas de la graine 1 est celui du module) :

| seuil | `2^-14` | `2^-17` | **`2^-20`** | `2^-24` | `2^-30` |
|---|---|---|---|---|---|
| amas identiques à ceux de `2^-20` | 19 / 24 | 23 / 24 | 24 / 24 | 24 / 24 | 24 / 24 |
| corrélation moyenne | 1,6753 | 1,6748 | 1,6756 | 1,6756 | 1,6756 |
| passes par site | 39 | 59 | 80 | 111 | 161 |

Les amas sont **identiques à ceux de `2^-20` au-dessous de `2^-20`** et ne changent qu'à partir de `2^-17` : le choix `2^-20` est conservateur et n'est pas ce qui décide d'un amas. **Le facteur `ω`**
ne change que le chemin vers la limite (le même amas pour `a` de 2,5 à 12 sur la graine 1, les passes de 104 à 199).

### Ce qui est mesuré (et que les tests figent, étiquetés « régression »)

**`η = 0` : un amas plein.** 2000 sites, `d = 50`, 120 graines : la dimension de masse-rayon (rayons 6 à rayon/2) vaut `2,0084` (σ `0,0031`), à `-0,0020` (σ `0,0023`) de celle du disque plein de mêmes rayons ;
le disque de rayon 10 est rempli à `0,9999` (au moins `0,9968`), celui de rayon 16 à `0,998` (au moins `0,986`). La corrélation sur les rayons 2 à 4 vaut `1,958` (une borne basse : le disque fini la
fait baisser) ; `1/β` sur `N/8` à `N` vaut `2,14` (les premiers sites ne sont pas encore un disque). Seule la masse-rayon et le remplissage sont testés.

**`η = 1` contre la DLA de l'étape 1.** Le claquage est **plus compact que la DLA** à ces tailles, de façon **stable** : mêmes estimateurs, mêmes fenêtres pour les deux modèles :

| sites (graines claquage / DLA) | corrélation, claquage | DLA | écart | `1/β`, claquage | DLA | rayon, claquage | DLA |
|---|---|---|---|---|---|---|---|
| 300 (120 / 120), `d = 45`, rayons 2 à 8 | 1,677 | 1,603 | **+0,074** | 1,800 | 1,687 | 23,5 | 27,6 |
| 600 (24 / 60), `d = 90`, rayons 3 à 12 | 1,665 | 1,630 | **+0,035** | 1,785 | 1,697 | 34,4 | 40,6 |
| 1000 (12 / 40), `d = 130`, rayons 4 à 12 | 1,680 | 1,631 | **+0,049** | 1,778 | 1,691 | 42,8 | 56,3 |

Le claquage `η = 1` n'est **pas la DLA** sur ces tailles : l'écart de corrélation est de `+0,04` à `+0,07` (de 4 à 20 fois l'écart-type de la différence des moyennes), celui du rayon de `-4` à `-13` sites (le rayon du claquage
croît moins vite : de 23,5 à 42,8 pour 300 à 1000 sites contre 27,6 à 56,3). **Ce n'est pas un effet de la frontière** (voir la table de `d` ci-dessus : aucun effet au-delà du bruit jusqu'à 4 fois le rayon).
**La cause est mesurée (le juge, 09/11)** : le marcheur de l'étape 1 colle dès qu'un de ses voisins est dans l'amas, donc son ensemble absorbant est l'amas `C` **et ses sites
périmètre** `P` (les candidates), et sa chance d'arriver en `p` est la mesure harmonique de `p` dans `C ∪ P` ; le claquage met `φ = 0` sur `C` seul et lit `φ` au site candidat. Dans le
continu les deux coïncident, pas sur la grille : `P` absorbant écrante davantage les fonds de fjords. Le juge a écrit deux variantes (outil `juge_mesures.rs`, hors du module, même
frontière `d`, même relaxation, graines 5001 et suivantes) :

| modèle (300 sites, `d = 45` ; 600 sites, `d = 90` ; corrélation rayons 2 à 8) | corrélation 300 | rayon 300 | corrélation 600 | rayon 600 |
|---|---|---|---|---|
| DLA de l'étape 1 (100 / 50 graines) | 1,6085 (σ 0,039) | 27,4 | 1,6194 (σ 0,032) | 40,3 |
| claquage `η = 1` de la doc (100 / 50) | 1,6797 (σ 0,044) | 23,5 | 1,6954 (σ 0,030) | 33,7 |
| `C ∪ P` absorbant, poids = flux `Σ φ(voisins)` (60 / 50) | **1,6129** (σ 0,040) | **27,6** | **1,6179** (σ 0,030) | **40,3** |
| `C` absorbant, poids = `φ(p) ×` nombre de voisins dans l'amas (60) | 1,6863 (σ 0,032) | 22,2 | | |

Avec `C ∪ P` absorbant la croissance **redonne la DLA** (écart `+0,004 ± 0,006` à 300 sites, `-0,002 ± 0,006` à 600, rayons égaux) ; compter les liens vers l'amas ne change rien. À 1000 sites
(`d = 110`, 50 graines) : claquage `1,6993`, DLA `1,6169` (`+0,082`), rayons 45,8 et 54,0. **Le modèle de la doc n'est pas changé** (`φ` au site candidat est le choix de la publication) :
le claquage `η = 1` sur la grille est un modèle voisin de la DLA, plus compact, et l'écart ne diminue pas de 300 à 1000 sites. Le plan parlait de « statistiques de la DLA retrouvées dans une tolérance » :
**la tolérance mesurée est de 0,07 sur la corrélation**, pas de quelques centièmes. (L'équivalence `η = 1 ↔ DLA` est celle du plan et du continu ; elle n'a pas été relue dans l'article de 1984, dont le texte intégral n'est pas lu.)

**Plus `η` monte, plus l'amas est fin** (100 sites, `d = 40`, 120 graines, corrélation sur les rayons 2 à 6) :

| `η` | corrélation | `1/β` | rayon moyen |
|---|---|---|---|
| 1 | 1,606 (σ 0,056) | 1,767 | 13,1 |
| 2 | 1,455 (σ 0,068) | 1,503 | 17,1 |
| 3 | 1,337 (σ 0,062) | 1,314 | 23,0 |

Un seul amas de 1000 sites par `η` (graine 1) : `η = 2` : `1/β = 1,40` et rayon 74 ; `η = 3` : `1/β = 1,21` et rayon 108 ; un seul amas de 1000 sites à `η = 1` : `1/β = 1,63` (une graine :
l'écart-type d'une graine est de l'ordre de 0,13 : ces trois nombres sont de l'ordre des valeurs de Hastings, ils ne les confirment pas).

### Les valeurs publiées, dites telles quelles

- **Hastings (Phys. Rev. Lett. 87, 175502, 2001), tableau II** (lu, par ar5iv) : `η = 2` : `D = 1,433` ; `η = 3` : `D = 1,263` (ajusté sur le jeu entier ; sur la dernière moitié, le dernier quart et le dernier huitième : `1,4256`, `1,435`,
  `1,4522` et `1,264`, `1,262`, `1,243`). Méthode : **applications conformes itérées, pas un réseau**, sans barres d'erreur dans le tableau. **Le prototype ne les atteint pas et ne les teste pas** : à 100 sites la corrélation
  sur les rayons 2 à 6 donne `1,455` et `1,337` (et `1/β` `1,50` et `1,31`), à 1000 sites une graine donne `1/β = 1,40` et `1,21` ; ces estimateurs, ces tailles et ce réseau ne sont pas ceux de la publication. L'accord avec la
  courbe publiée est l'objet de D2.
- **La DLA à 1,71** (Mathiesen, Jensen, Bakke, arXiv:0712.1739, section IV, lu ; Davidovitch et coll. 2000 : 1,713 ± 0,003, un résultat de calcul sur de petits amas) : repère non testé ; le banc donne `1,60` (corrélation,
  300 sites) à `1,65` (4000 sites, étape Z1).
- **Eden** : « de 2 (`η = 0`, le modèle d'Eden) à environ 1,7 (`η = 1`, DLA) à 1 » (Hastings 2001, introduction, lu). La compacité de l'amas d'Eden et la rugosité de sa frontière : **à vérifier dans l'article d'Eden (1961, non lu)**.

### Références

- L. Niemeyer, L. Pietronero, H. J. Wiesmann, *Fractal dimension of dielectric breakdown*, Phys. Rev. Lett. 52, 1033 (1984), https://doi.org/10.1103/PhysRevLett.52.1033 (résumé lu ; texte intégral non lu).
- M. B. Hastings, *Fractal to nonfractal phase transition in the dielectric breakdown model*, Phys. Rev. Lett. 87, 175502 (2001), arXiv:cond-mat/0103312 (lu : introduction et tableau II).
- J. Mathiesen, M. H. Jensen, J. Ø. Bakke, *Dimensions, maximal growth sites and optimization in the dielectric breakdown model*, arXiv:0712.1739 (section IV et figure 4 lues ; la revue et l'année ne sont pas lues).
- M. Eden, *A two-dimensional growth process*, Proc. Fourth Berkeley Symp. Math. Stat. Prob. 4, 223 (1961) (non lu).

### Ce que cette étape n'établit pas

- **Que le claquage `η = 1` redonne la DLA** : à 300, 600 et 1000 sites il est plus compact de `0,04` à `0,08` en corrélation ; la cause (l'ensemble absorbant `C` contre `C ∪ P`) est mesurée
  par une variante hors du module, que les tests ne contiennent pas ; que l'écart tende vers 0 aux grandes tailles n'est pas établi (il ne baisse pas jusqu'à 1000 sites).
- **Aucune dimension publiée** : ni `D(η = 2) = 1,433`, ni `D(η = 3) = 1,263`, ni la DLA à 1,71 ne sont atteintes ou testées ; à 100 sites la dimension mesurée est bien en dessous de sa valeur limite. C'est l'étape D2.
- **Que la méthode de 1984 soit celle-ci** (grille, bord, calcul du potentiel : non lus), ni que le résultat ne dépende pas de la frontière pour `η >= 2` (la sensibilité à `d` n'est mesurée qu'à `η = 1`).
- **La valeur du champ au-delà de la tolérance de la doc** : le champ est à `2 SEUIL (d + 1)²` de la solution exacte (mesuré : cinquante à cent fois moins) ; un autre schéma de relaxation donnerait des champs différents de l'ordre du seuil, donc, parfois,
  un autre amas : c'est pourquoi la doc fixe le schéma bit pour bit.
- **Un `W = 0`** (tous les poids nuls : le seul `None` qu'aucune graine n'atteint) et un arrêt faute de candidates : fixés par la doc, non atteints par un test.
- **Le coût dans la page** (D3) : jamais mesuré dans le navigateur ; à 100 sites et `d = 40` le temps natif est de 0,25 s par amas, et `η = 3` à 1000 sites coûte 210 s natifs.
- **Les amas au-delà de 3000 sites**, `η` non entier, la 3D, la transition `η = 4` (hors de D1).
- **Que `etape-d1` laisse la page et le module web inchangés** n'est pas testé ici (ni Node ni la cible `wasm32` sur ce PC) : `claquage` n'est lu que par `lancer` ; ajouter `etape-d1` à `default` change pourtant les octets du module compilé (règle 12 d'`AGENTS.md`).

## Étape D2 : la courbe D(η) du claquage, à côté des points publiés

La dimension fractale `D` des amas du claquage de l'étape D1 (`src/claquage.rs`), mesurée par la corrélation densité-densité de l'étape Z1 (`src/correlation.rs`) sur une **fenêtre faite pour les petits amas**, moyennée
sur des graines, avec son erreur-type, pour `η = 0, 1, 2, 3` (`η = 0` : l'amas plein, en repère) à une taille d'amas fixée ; puis mise **à côté** des points publiés (Hastings 2001, le rappel de la DLA), sans qu'aucun test
les leur compare jamais. Le module est `src/courbe.rs` (fonctionnalité `etape-d2`) : la fenêtre, la dimension d'un amas, la moyenne et l'erreur-type (leur formule et leur ordre d'accumulation), le demi-côté, les repères,
le CSV et l'image y sont écrits à la lettre ; l'exemple `examples/courbe_d2.rs` écrit `apercus/etape-d2-courbe.png` et `apercus/etape-d2-courbe.csv`. Les valeurs attendues des tests, chacune avec sa source, sont dans
`references/etape-d2-valeurs.md`. **La règle des nombres d'`AGENTS.md` ne change pas** : `+ - * /`, `sqrt`, `round`, et `math::logarithme` par `correlation` ; les comptages en entiers ; aucun fil d'exécution dans le module
(les tests en emploient pour aller plus vite, sans changer un bit).

**La limite acceptée (André, 09/11).** À nos tailles (100 à 300 sites ; le coût du champ de Laplace interdit mieux, voir D1), la dimension mesurée est **loin de sa valeur limite** : D2 ne dit que **l'ordre et la baisse** de
`D(η)`, jamais un accord avec les valeurs de Hastings. Les points publiés sont des repères affichés avec l'écart mesuré.

### La mesure

- **La fenêtre des petits amas** (`fenetre_petit_amas`) : les rayons entiers de **2 à `max(4, e / 6)`**, `e` l'étendue de l'amas (division entière), vide si ce dernier rayon dépasse `e / 2` ou 4096 (`RAYON_MAX`). Elle ne dépend
  que de l'étendue. La fenêtre de Z1 (4 à `e / 8`, au moins 72 d'étendue) ne mesure rien ici : un amas de 100 sites a une étendue de 11 (`η = 0`) à 49 (`η = 3`), un amas de 300 sites de 22 à 114 : seuls des amas de `η = 3` à 300 sites atteignent 72.
- **La dimension d'un amas** (`dimension_d_un_amas`) : `2 + pente(ln r, ln C(r))` sur cette fenêtre, par `correlation::dimension_correlation` ; `None` si la fenêtre est vide ou si une `C(r)` est nulle.
- **Le point** (`point_de_la_courbe`) : pour chaque graine, dans l'ordre donné, un `Claquage` jusqu'à `taille` sites et sa dimension ; la moyenne (somme de gauche à droite, division par `n`), la variance **corrigée** (`n - 1`)
  et l'**erreur-type** `sqrt(variance / n)` (celle de la moyenne, pas l'écart-type d'une graine). Un amas qui touche la frontière avant `taille`, une dimension indéfinie ou moins de deux graines : panique, dite.
- **Le demi-côté** (`demi_cote_de_la_courbe`, pris par `courbe`) : `min(255, A + B · taille / 100)` en entiers, `(A, B)` = `(30, 10)`, `(32, 8)`, `(22, 18)`, `(12, 36)` pour `η = 0` à 3 : 40, 40, 40, 48 à 100 sites et 60, 56, 76, 120 à 300.
  `η > 3` est refusé.
- **Les repères** : `HASTINGS_2001_TABLE_II` (`η = 2 → 1,433`, `η = 3 → 1,263`) et `DLA_RAPPEL` (`1,71`, pour `η = 1` : **pas une mesure de Hastings**, qui écrit « roughly 1.7 »), `ecart(point, publie) = moyenne - publie`.

### La fenêtre et sa sensibilité (mesurée)

**Choisie sur des formes à solution connue, pas sur le claquage.** Écart `D mesuré - D exact` pour la même formule de fenêtre sur 14 formes (trait de 50, 100 et 300 sites ; carrés de 10, 12, 17 et 25 de côté ; disques de rayon 6, 10 et 15 ;
Vicsek de niveaux 3 et 4, `ln 5 / ln 3 = 1,465` ; tapis de niveaux 2 et 3, `ln 8 / ln 3 = 1,893`) ; outil jetable `d2_mesures formes` :

| fenêtre | trait 100 | trait 300 | carré 12 | carré 25 | disque 10 | Vicsek 4 | tapis 3 | erreur absolue moyenne (14 formes) |
|---|---|---|---|---|---|---|---|---|
| 2 à 4 (fixe) | -0,020 | -0,011 | -0,225 | -0,101 | -0,124 | +0,039 | -0,093 | **0,123** |
| **2 à `max(4, e/6)` (la règle)** | **-0,049** | **-0,037** | **-0,225** | **-0,101** | **-0,124** | **+0,020** | **-0,093** | **0,128** |
| 2 à `max(4, e/8)` | -0,041 | -0,032 | -0,225 | -0,101 | -0,124 | -0,003 | -0,093 | 0,126 |
| 2 à `max(4, e/4)` | -0,061 | -0,049 | -0,225 | -0,134 | -0,153 | +0,002 | -0,133 | 0,138 |
| 2 à `max(4, e/3)` | -0,076 | -0,061 | -0,225 | -0,159 | -0,174 | -0,030 | -0,142 | 0,154 |
| 2 à `max(5, e/6)` | -0,049 | -0,037 | -0,273 | -0,121 | -0,153 | +0,020 | -0,127 | 0,155 |
| 2 à 6 (fixe) | -0,056 | -0,044 | -0,307 | -0,134 | -0,174 | +0,033 | -0,133 | 0,172 |
| 2 à 8 (fixe) | -0,033 | -0,018 | -0,376 | -0,159 | -0,216 | +0,003 | -0,142 | 0,197 |
| 3 à `max(6, e/6)` | -0,049 | -0,038 | -0,375 | -0,162 | -0,217 | +0,026 | -0,181 | 0,209 |

Lecture : la dimension d'un **petit objet compact** (carré de 12, disque de rayon 10) est sous-estimée de 0,10 à 0,30 **quelle que soit la fenêtre** (l'effet de bord : un voisinage qui sort de l'objet est vide), d'autant plus que
la fenêtre monte ; un trait est mesuré à 0,95 au lieu de 1 ; une fractale de Vicsek de 625 sites à 1,485 au lieu de 1,465. La règle est dans le palier des fenêtres qui commencent à 2 et dont le dernier rayon reste petit
(erreur moyenne de 0,123 à 0,138) ; elle n'est pas la meilleure (2 à 4 fixe fait 0,005 de mieux) mais elle **monte avec l'étendue** (un trait de 300 sites est mesuré jusqu'au rayon 49) et ressemble à la fenêtre de Z1 (`e / 8`)
un peu élargie parce que les amas sont petits. `k = 6` ou `8` ne se distinguent pas sur les formes (0,128 et 0,126) ; **le tableau du claquage ci-dessous avait déjà été vu quand `k = 6` a été retenu** : la fenêtre est jugée sur les formes, mais son choix final entre 6 et 8 ne s'est pas fait sans avoir vu ce que le claquage donne ; le tableau dit ce que chaque fenêtre aurait donné.

**Sensibilité de `D` du claquage à la fenêtre** (moyenne et, entre parenthèses, l'écart-type d'UNE graine ; 100 sites : 48 graines, demi-côtés 40, 40, 40, 48 ; 300 sites : 24 graines, demi-côtés 60, 56, 76, 120) :

| fenêtre | 100 sites : `η = 0` | `η = 1` | `η = 2` | `η = 3` | 300 sites : `η = 0` | `η = 1` | `η = 2` | `η = 3` |
|---|---|---|---|---|---|---|---|---|
| 2 à 4 (fixe) | 1,758 | 1,657 | 1,496 | 1,378 | 1,880 | 1,703 | 1,508 | 1,394 |
| 2 à 5 (fixe) | 1,702 | 1,622 | 1,468 | 1,342 | 1,855 | 1,687 | 1,481 | 1,360 |
| 2 à 6 (fixe) | 1,658 | 1,601 | 1,460 | 1,335 | 1,836 | 1,682 | 1,474 | 1,354 |
| 2 à 8 (fixe) | 1,566 | 1,559 | 1,444 | 1,326 | 1,796 | 1,674 | 1,466 | 1,351 |
| 2 à `max(4, e/4)` | 1,758 | 1,636 | 1,458 | 1,318 | 1,841 | 1,668 | 1,434 | 1,288 |
| 2 à `max(4, e/8)` | 1,758 | 1,657 | 1,496 | 1,372 | 1,880 | 1,699 | 1,471 | 1,333 |
| 2 à `max(4, e/12)` | 1,758 | 1,657 | 1,496 | 1,378 | 1,880 | 1,703 | 1,492 | 1,352 |
| **2 à `max(4, e/6)` (la règle)** | **1,758** (0,020) | **1,657** (0,060) | **1,490** (0,076) | **1,342** (0,066) | **1,880** (0,011) | **1,684** (0,038) | **1,456** (0,046) | **1,315** (0,049) |
| 3 à `max(6, e/6)` | 1,573 | 1,561 | 1,438 | 1,309 | 1,798 | 1,671 | 1,442 | 1,299 |

**Ce que la table dit.** (1) **L'ordre `D(0) > D(1) > D(2) > D(3)` tient pour toutes les fenêtres qui mesurent les quatre `η`** : 29 sur 34 à 100 sites (les cinq autres sont vides pour tous les amas d'un `η`, d'où les `NaN` de l'outil) et 31 sur 34 à 300 sites. Mais l'écart `D(0) - D(1)` n'est de 0,10 (100 sites) qu'avec une fenêtre qui reste petite : il tombe à **0,007** pour 2 à 8
et à 0,012 pour 3 à `max(6, e/6)`, parce que le dernier rayon atteint alors la moitié de l'étendue d'un amas d'Eden de 100 sites (14,8) : l'effet de bord d'un amas plein compact se confond avec la ramification de la DLA. (2) **Le choix de la
fenêtre déplace `D` de ±0,05** : à 300 sites, `D(2)` va de 1,41 à 1,51 et `D(3)` de 1,27 à 1,39 selon la fenêtre ; **c'est plus que l'écart mesuré à Hastings** (voir plus bas) : on peut faire « tomber » le banc sur 1,433 ou sur 1,263 (2 à `e / 4`
donne `D(2) = 1,434` à 300 sites) en choisissant la fenêtre. C'est la leçon du zinc, et la raison pour laquelle aucun test ne compare ces valeurs.

**Recoupement du juge, sur 100 graines neuves (7001 à 7050 et 9001 à 9050), 14 fenêtres, un second code écrit d'après la seule doc** (mêmes bits que le prototype).
À 100 sites, l'ordre strict `D(0) > D(1) > D(2) > D(3)` tient pour les 14 fenêtres, mais `D(0) - D(1)` vaut 0,099 (z = 17,5) pour la règle, 0,077 pour 2 à 5, 0,054 pour 2 à 6, **0,006 (z = 1,1) pour
2 à 8** et **0,011 (z = 1,6) pour 3 à `max(6, e/6)`** ; à **80 sites**, ces deux fenêtres **inversent** `D(0)` et `D(1)` (`-0,014` et `-0,009`), alors que toutes celles qui commencent à 2 et
dont le dernier rayon reste au plus `e / 4` gardent l'ordre avec z ≥ 14. L'ordre établi par D2 est donc celui des fenêtres **courtes qui commencent à 2** (le dernier rayon loin de la moitié de
l'étendue d'un amas d'Eden, 13 à 15 ici) : c'est une propriété du modèle ET de cette famille de fenêtres, pas de toute fenêtre.
**Le choix entre `e / 6` et `e / 8` n'est pas neutre pour l'écart** : à 100 sites il ne change que les amas d'au moins 30 d'étendue, donc `D(2)` de 0,005 et surtout `D(3)` : `1,359` avec `e / 6`
contre `1,390` avec `e / 8` (juge, 100 graines neuves) ; l'écart à Hastings pour `η = 3` serait `+0,127` au lieu de `+0,096`. Les formes ne départagent pas (0,128 et 0,126, `e / 8` un peu meilleur) et `k = 6` a été retenu après avoir
vu le tableau du claquage : **ce choix rapproche le banc du publié pour `η = 3` sans que les formes le justifient**. Aucun test ne compare `D` au publié, mais les bandes et le CSV figé de
`etape_d2_ordre.rs` figent ce choix (une fenêtre en `e / 8` en sort : le CSV figé la voit). Les 14 fenêtres et leurs valeurs : `essai-d2-juge-outils`
des notes de la session.

### Le demi-côté (mesuré)

Rayon moyen des amas (maximum), graines 1 à 48 à 100 sites et 1 à 24 à 300 sites, avec un demi-côté assez grand pour ne jamais toucher la frontière :

| sites | `η = 0` | `η = 1` | `η = 2` | `η = 3` |
|---|---|---|---|---|
| 100 | 9,6 (13) | 13,0 (17) | 16,9 (27) | 21,7 (33) |
| 300 | 15,5 (18) | 22,5 (25) | 36,0 (51) | 51,3 (69) |

(D1 : sur 120 graines à 100 sites, le maximum de `η = 3` est 35.) Pour `η >= 1` la règle donne un demi-côté de 2,1 à 3,1 fois le rayon moyen (`η = 0` : 3,9 à 4,2, sans conséquence, il n'y a pas de champ) ; sur les 170 graines à 100 sites **aucun amas n'a touché la frontière** (ni 24 graines à 300 sites).
**`D` ne dépend du demi-côté qu'à la marge** (100 sites, 48 graines, fenêtre de la règle ; erreur-type de la moyenne `0,009` à `0,012`) :

| `η` | demi-côté → `D` moyen |
|---|---|
| 1 | 25 → 1,670 ; 30 → 1,664 ; 40 → 1,657 ; 60 → 1,658 |
| 2 | 30 → 1,500 (1 amas sur 48 touche la frontière) ; 35 → 1,483 ; 40 → 1,490 ; 50 → 1,496 ; 70 → 1,483 |
| 3 | 35 → 1,330 (1 sur 48) ; 40 → 1,333 ; 45 → 1,341 ; **48 → 1,342** ; 60 → 1,351 ; 80 → 1,352 |

Pour `η = 1` et `2` rien au-delà du bruit. Pour `η = 3`, `D` monte de 0,02 entre 40 et 60 puis se stabilise : **le demi-côté de la règle (48) laisse `D(3)` environ 0,010 sous sa valeur à demi-côté 80** (une erreur-type). Le coût de 60 serait de 2
fois celui de 48 ; la règle ne le prend pas. `η = 0` : aucun champ n'est calculé, le demi-côté ne sert qu'à arrêter l'amas (coût nul, un demi-côté de 40 convient à 100 sites).

### Ce qui est mesuré

Dimension moyenne `D` de la courbe (fenêtre de la règle, demi-côtés de la règle), erreur-type de la moyenne entre parenthèses :

| sites (graines) | `η = 0` | `η = 1` | `η = 2` | `η = 3` |
|---|---|---|---|---|
| 100 (170 graines : 1 à 120 et 1001 à 1050) | 1,759 (0,002) | 1,662 (0,004) | 1,495 (0,005) | 1,349 (0,005) |
| 100 (graines 1 à 24, celles des tests) | 1,758 (0,0036) | 1,657 (0,0117) | 1,498 (0,0155) | 1,358 (0,0124) |
| 100 (graines 1 à 50, l'aperçu `etape-d2-courbe.png`) | 1,758 (0,0028) | 1,655 (0,0084) | 1,492 (0,0106) | 1,346 (0,0095) |
| 300 (graines 1 à 24) | 1,880 (0,0022) | 1,684 (0,0077) | 1,456 (0,0095) | 1,315 (0,0100) |

L'**ordre** : à 100 sites, `D(0) - D(1)`, `D(1) - D(2)` et `D(2) - D(3)` valent sur les 170 graines 0,097, 0,167 et 0,146, de 20 à 24 fois l'erreur-type de leur différence ; dans chacun des sept blocs de 24 ou 25 graines, de 6,1 à 11,7 fois (le juge, sur huit blocs de 24 graines neuves : de 4,2 à 10,3 fois, le minimum pour `D(2) - D(3)`) ; à 300 sites (24 graines)
0,196, 0,228 et 0,140 (de 10 à 24 fois). **Un amas plein de 100 sites mesure 1,76 et non 2**, un de 300 sites 1,88 : l'effet de bord d'un disque (rayon moyen 9,6 et 15,5). C'est le repère `η = 0`, qui n'a pas de point publié ici.

### L'écart aux points publiés, tel qu'il est (et pourquoi ce n'est pas un accord)

`écart = D du banc - repère` (`ecart`) :

| `η` | repère | 100 sites | 300 sites |
|---|---|---|---|
| 1 | 1,71 (le rappel de la DLA) | -0,048 (± 0,004) | -0,026 (± 0,008) |
| 2 | 1,433 (Hastings) | +0,062 (± 0,005) | +0,023 (± 0,010) |
| 3 | 1,263 (Hastings) | +0,086 (± 0,005) | +0,052 (± 0,010) |

(Le juge, sur 100 graines neuves à 100 sites, mesure `D` = 1,760 ; 1,662 ; 1,506 ; 1,359 (erreurs-types 0,002 ; 0,005 ; 0,007 ; 0,007) et les écarts **-0,048**, **+0,073** et **+0,096** :
le même signe, et pour `η = 2` et `3` un écart plus grand de 0,01 que celui des 170 graines de l'écrivain, à 1,1 et 1,2 erreur-type combinée près. L'écart est une mesure de NOS graines, pas une constante.)

**Le signe de l'écart n'est pas unique** : le banc est **au-dessus** de Hastings pour `η = 2` et `3` (de +0,06 et +0,09 à 100 sites, de +0,02 et +0,05 à 300 : l'écart baisse avec la taille ; les ± sont l'erreur-type de la moyenne du banc, Hastings n'en donne pas), et **au-dessous** du rappel de la DLA pour `η = 1`
(la DLA elle-même, mesurée par la corrélation, est à 1,60 à 300 sites et 1,65 à 4000, loin de 1,71 : Z1). Il ne faut donc pas écrire « le banc est au-dessus du publié ». **Ce n'est pas un accord, ni un désaccord** : (1) les amas sont petits
(100 à 300 sites contre 10 000 à 15 000 particules chez Hastings) et la dimension en dépend ; (2) la méthode est différente (une corrélation sur un réseau de côté entier contre l'inverse de la pente de `ln F_1` en fonction de `ln n`, sur la moyenne de 8 à 14 amas par applications
conformes) ; (3) **la fenêtre déplace `D` de ±0,05** (table ci-dessus), plus que l'écart ; (4) Hastings ne donne **aucune barre d'erreur** (seulement la fluctuation rms de `F_1`, `0,039` et `0,056`) ; (5) le claquage `η = 1` n'est pas la DLA à ces
tailles (D1 : `+0,07` de corrélation). Aucun test ne compare `D` à un repère.

### Le coût

Temps natif, un fil, un processus, profil `dev` à `opt-level = 3` (Ryzen 5 5500U, 12 fils ; temps de CPU, la machine étant partagée), **par amas**, avec les demi-côtés de la règle (moyenne de 8 amas à 100 sites, de 1 ou 2 à 300 sites) :

| sites | `η = 0` | `η = 1` | `η = 2` | `η = 3` |
|---|---|---|---|---|
| 100 | 0,003 s | 0,36 s | 0,38 s | 0,63 s |
| 300 | 0,1 s pour 24 amas (mur, 12 fils) | 2,3 s | 5,7 s | 21 s |

Une courbe de 24 graines coûte donc **33 s de CPU à 100 sites** (24 × (0,36 + 0,38 + 0,63)) et **de l'ordre de 12 minutes à 300 sites** (24 × 29 s) ; l'exemple (`η = 0` à 3, 50 graines, 100 sites) a pris 67 s de CPU, un fil. C'est le coût de D1 (le champ de Laplace à chaque
pas : `d³` par site) : `η = 3` coûte 33 fois plus à 300 sites qu'à 100. **La courbe à 300 sites n'est donc ni un test ni un calcul de page** (D3 bornera la taille) ; elle a été mesurée une fois, pour la table ci-dessus.
**Les tests** (`cargo test --features etape-d2`, ce PC, profil `dev`) : `etape_d2_exact` 2,3 s (4,3 s de CPU), `etape_d2_ordre` **16,8 s** (36,8 s de CPU : 24 graines × `η = 1` à 3 à 100 sites, un fil par `η`, le plus long est `η = 3` : 24 × 0,63 s), `etape_d2_exemple` 2,0 s (3,0 s de CPU),
`etape_d2_fautes` 0,1 s : **21 s au mur, 44 s de CPU**. Le plan disait « quelques secondes par fichier » : c'est faux pour `etape_d2_ordre`, que le coût du champ (24 graines à 100 sites) fixe à une quinzaine de secondes. Sur une CI de deux cœurs : de l'ordre de 30 s (non mesuré).

### Références

- M. B. Hastings, *Fractal to Nonfractal Phase Transition in the Dielectric Breakdown Model*, Phys. Rev. Lett. 87, 175502 (2001), arXiv:cond-mat/0103312v2 : **lu en entier** (PDF de l'arXiv v2, texte extrait le 09/11) : tableau II (colonne « full set » :
  `η = 2 → 1,433`, `η = 3 → 1,263`, sans barre d'erreur), méthode (applications conformes itérées, régression de `ln F_1` contre `ln n` sur la moyenne de 8 à 14 réalisations de 10 000 à 15 000 particules), introduction (« roughly 1.7 (η = 1, DLA) »).
  La référence de la revue (PRL 87, 175502) est celle que la page de l'arXiv donne (« Journal reference ») ; la version imprimée n'a pas été ouverte.
- J. Mathiesen, M. H. Jensen, J. Ø. Bakke, *Dimensions, maximal growth sites and optimization in the dielectric breakdown model*, arXiv:0712.1739, section IV (lue par ar5iv) : « the DLA value D=1.71 for η=1 » ; rien pour `η = 2` ou `3`.
- B. Davidovitch, A. Levermann, I. Procaccia, arXiv:cond-mat/0008053 (`1,713 ± 0,003`, un calcul sur de petits amas : lu à Z1, non relu).
- Les dimensions de similarité de Vicsek (`ln 5 / ln 3`) et du tapis de Sierpinski (`ln 8 / ln 3`) : celles de l'étape 1.

### Ce que cette étape n'établit pas

- **Aucun accord avec Hastings, ni avec la DLA à 1,71** : l'écart est écrit (+0,06 et +0,09 à 100 sites pour `η = 2` et `3`, -0,05 pour `η = 1`), jamais testé ; la fenêtre le déplace de ±0,05.
- **La dimension limite** : à 100 ou 300 sites on est loin de la limite (la dimension monte avec la taille pour `η = 0` et `1`, baisse pour `η = 2` et `3`) ; rien n'est dit au-delà de 300 sites.
- **Que la fenêtre soit la bonne** : elle est dans un palier (0,12 à 0,14 d'erreur moyenne sur les formes), pas la meilleure, et elle sous-estime un petit objet compact de 0,10 à 0,30 ; sur le claquage seul, l'ordre tient pour toutes les fenêtres
  essayées à 100 sites mais `D(0) - D(1)` s'effondre si le dernier rayon atteint la moitié de l'étendue de l'amas d'Eden, et **s'inverse à 80 sites** pour 2 à 8 et 3 à `max(6, e/6)` (juge) :
  l'ordre établi est celui des fenêtres courtes qui commencent à 2, pas celui de toute fenêtre.
- **La sensibilité au demi-côté au-delà de `η = 3` et de 300 sites** (la règle y est extrapolée, plafonnée à 255) ; le demi-côté de `η = 3` à 100 sites laisse `D` 0,01 sous sa valeur à demi-côté 80.
- **Que la méthode de 1984 soit celle-ci** (D1) ; `η` non entier, `η >= 4` (la transition à `η = 4` est un débat ouvert : non testée), la 3D.
- **L'exemple lui-même** n'est pas lancé par un test (le jeu gelé ne le livre pas, et un `cargo run` imbriqué serait fragile) : ce que les tests voient, ce sont `csv_de_la_courbe` et `png_de_la_courbe`, qu'il appelle.
- **L'aspect de l'image** (axes, grille, lisibilité) : seuls la taille, les marqueurs et les barres sont fixés ; la page et le curseur `η` sont D3. Que `etape-d2` laisse la page et le module web inchangés n'est pas testé ici (ni Node ni `wasm32` sur ce PC) ;
  ajouter `etape-d2` à `default` change pourtant les octets du module compilé (règle 12 d'`AGENTS.md`).
- **Les temps sur une CI** : mesurés ici seulement (voir « Le coût »).

## Étape D3 : la décharge sur la page (le claquage, le curseur η, la courbe D(η) à la demande)

D1 a le claquage diélectrique (`src/claquage.rs`), D2 sa courbe `D(η)` (`src/courbe.rs`) ; D3 les met **sur la page**, comme Z1 y a mis la corrélation. Le modèle « Claquage diélectrique » de la liste, un **curseur `η`** (0 à 3), « Mesurer D » sur l'amas
du moment avec le repère publié de ce `η`, et un bloc « Courbe D(η) » qui se calcule **un amas à la fois**. Deux promesses : **ce qui peut échouer ne plante pas la page** (une panique dans le module `wasm32` abandonne le module entier : la page se
figerait) et **la page borne la taille des amas** (le claquage coûte cher : D1). Les fichiers : `src/courbe.rs` (les entrées qui rendent une erreur, fonctionnalité `etape-d3`), `src/vitrine.rs` (les modèles 4 à 7, la mesure du claquage, huit exportations
de plus : vingt-trois en tout), `docs/index.html`, `docs/app.js`, `docs/lecture.js`, `docs/givre.wasm` (reconstruit). Les valeurs attendues des tests, chacune avec sa source, sont dans `references/etape-d3-valeurs.md`. **La règle des nombres d'`AGENTS.md`
ne change pas** (`+ - * /`, `sqrt`, `round`, `math::logarithme` par `correlation` ; aucun fil d'exécution ni horloge dans `src/`).

### Les entrées qui rendent une erreur

Les fonctions de D2 **paniquent** sur un cas absurde (c'est leur contrat, éprouvé par les tests gelés de D2, et il ne change pas). D3 **ajoute** des entrées jumelles qui rendent un `Result` : `essayer_demi_cote_de_la_courbe`,
`essayer_dimension_d_une_graine` (la dimension d'**un** amas : le pas du calcul de la page), `essayer_point_de_la_courbe` (composé des briques de D2 ; **même point, à `to_bits` près**, que `point_de_la_courbe` quand rien n'échoue) et
`mesure_petit_amas` (la fenêtre des petits amas, les `C(r)` et la dimension, pour la page). `ErreurCourbe` a six variantes (`GrainesInsuffisantes`, `EtaTropGrand`, `DemiCoteHorsBorne`, `TailleHorsBorne`, `FrontiereTouchee { graine }`, `FenetreVide { graine }`) et un
texte fixé (`Display`). **L'ordre des vérifications** : les graines, puis `eta` (au plus 8, 3 pour le demi-côté de la règle), `demi_cote` (au plus 255), `taille` (au plus 100 000 : `PARTICULES_MAX`), puis la croissance des graines dans l'ordre. Une `taille` de 0 ou 1, ou
trop petite pour une fenêtre (étendue de moins de 8), est une fenêtre vide, pas une erreur de paramètre. **Aucune borne de temps** : une taille permise peut coûter des heures à `η >= 1` (D1) ; la vitrine borne elle-même. Le débordement de `B × taille` que la
relecture de la PR 29 avait signalé dans `demi_cote_de_la_courbe` est une erreur (`TailleHorsBorne`) dans `essayer_demi_cote_de_la_courbe` ; la fonction de D2, elle, n'a pas bougé d'un octet (hors de portée de la page).

### Les modèles 4 à 7 et leurs bornes

Le modèle `4 + η` est le claquage d'exposant `η` à une **taille bornée** (germe compris) et au demi-côté de la règle de D2 :

| modèle | `η` | sites | demi-côté | durée | coût natif d'une croissance entière (`release`) |
|---|---|---|---|---|---|
| 4 | 0 | 400 | 70 | 399 | 0,02 s |
| 5 | 1 | 250 | 52 | 249 | 1,5 s en moyenne, 1,7 s au pire (24 graines) ; 3,3 s au pire sur 120 graines (machine chargée) |
| 6 | 2 | 150 | 49 | 149 | 0,7 s, 0,85 s au pire (24 graines) ; 2,1 s au pire sur 120 graines (machine chargée) |
| 7 | 3 | 100 | 48 | 99 | 0,5 s, 0,5 s au pire (24 graines) ; 1,9 s au pire sur 120 graines (machine chargée) |

**Les tailles viennent du coût** (D1 : le champ de Laplace est relaxé après chaque site, `d³` par site) : le brief suggérait « une dizaine de secondes pour `η = 3` » (le temps de la croissance entière) ; la mesure dit que `η = 3` à 150 sites coûte 3 à 5 s,
à 200 sites 7 s, à 300 sites 32 s (une graine, machine chargée) : **100 sites pour `η = 3`** garde la croissance entière autour de la demi-seconde en natif au calme (1 à 2 s quand la machine est chargée) et laisse de la marge au module WebAssembly, dont la vitesse n'est pas mesurée ici (voir « Ce que cette étape n'établit pas »).
La même idée pour les autres : 150 sites à `η = 2`, 250 à `η = 1` (de 1 à 3 s), 400 à `η = 0` (gratuit). **Le pire d'une unité** (un site ajouté, un appel de `givre_avancer(1)`) : 0,001 s à `η = 0` ; 0,026 s, 0,018 s, 0,013 s à `η = 1, 2, 3` (24 graines, machine au calme),
jusqu'à 0,09 s à 0,13 s sur 120 graines (machine chargée). C'est pourquoi la page joue ces modèles **unité par unité**, en s'arrêtant quand une trentaine de millisecondes sont écoulées, au lieu d'un appel de `vitesse` unités.

**La marge au bord** (120 graines par modèle, le demi-côté de la table, le rayon de l'amas à la fin ; `frontières` est le nombre d'amas qui ont touché le bord avant leur taille) :

| modèle | rayon moyen | rayon maximal | marge (demi-côté − rayon maximal) | frontières |
|---|---|---|---|---|
| 4 (`η = 0`) | 17,2 | 22 | 48 | 0 |
| 5 (`η = 1`) | 21,1 | 30 | 22 | 0 |
| 6 (`η = 2`) | 22,7 | 37 | **12** | 0 |
| 7 (`η = 3`) | 22,2 | 34 | 14 | 0 |

Aucun amas n'a touché la frontière sur 480 amas (graines 1 à 120) : la règle de D2 (« 2,1 à 3,1 fois le rayon moyen ») tient. Si une graine le faisait, `givre_avancer` s'arrêterait sans panique (`Claquage::essayer_ajouter` rend `None`) et la page dirait
« Scénario terminé » ; ce n'est pas une promesse que 4 milliards de graines ne le font jamais (non testé : voir plus bas).

**Mesure du juge (10/11, natif `release`, machine chargée par d'autres calculs : les temps sont des majorants)**, sur des graines plus nombreuses et sur des graines neuves (7001 à 7050) ; aucune frontière touchée, aucune panique :

| modèle | graines | rayon moyen | rayon maximal | marge | temps moyen (pire) d'une croissance | pire unité |
|---|---|---|---|---|---|---|
| 4 (`η = 0`) | 1 à 2000 | 17,2 | 24 | 46 | 0,03 s (0,12 s) | 0,04 s |
| 5 (`η = 1`) | 1 à 300 | 21,2 | 30 | 22 | 1,8 s (5,2 s) | 0,23 s |
| 6 (`η = 2`) | 1 à 500 | 22,8 | 37 | 12 | 1,6 s (2,8 s) | 0,17 s |
| 7 (`η = 3`) | 1 à 500 | 22,7 | **45** | **3** | 1,2 s (3,4 s) | 0,24 s |
| 4 à 7 | 7001 à 7050 | 16,8 ; 21,3 ; 22,4 ; 22,8 | 20 ; 30 ; 33 ; 31 | 50 ; 22 ; 16 ; 17 | 0,13 (0,25) ; 3,5 (5,2) ; 1,7 (3,3) ; 1,3 (1,8) s | 0,18 ; 0,13 ; 0,08 ; 0,09 s |

**La marge du modèle 7 n'est que de 3 sites sur 500 graines** (un amas de `η = 3` est presque un trait : son rayon maximal vaut deux fois le rayon moyen) : une graine parmi les 4 milliards qui touche le bord à `η = 3` est plausible ; la page dirait alors « Scénario terminé » avant 100 sites, sans panique (c'est le contrat). Relever le demi-côté du modèle 7 (la règle de D2 donne 48) serait un changement de la table (donc des empreintes) : à décider par le pilote, pas fait ici.

**La chaîne entière contre un autre programme** : la dimension de chacun des quatre modèles, joué jusqu'au bout avec la graine 1 et mesuré par la vitrine, est celle de `d2_ref.py amas` (Python : le claquage de `d1_ref.py`, écrit d'après la doc de D1 seule, puis la fenêtre, la corrélation en fractions exactes et la dimension en `decimal` à 60 chiffres de D2) à `1e-12`
près (1,89281132677495956025 ; 1,74493588341019327296 ; 1,59607404588882589411 ; 1,38303932787124472016) : le test `la_dimension_des_quatre_modeles_est_celle_de_python` la fige.

**L'image et l'empreinte** : l'image est `amas_colore(sites, COTE_APERCU)` de D1/Z1, la même que la DLA (un carré qui suit l'amas, 512 × 512 au temps 0) ; l'empreinte est `Claquage::empreinte()` (elle contient le demi-côté et `η` : deux modèles de claquage n'ont jamais la
même). **L'empreinte d'une graine sur la page est celle du code natif** (testé sous Node contre le natif pour les quatre modèles).

### Ce que la page mesure, avec le repère de `η`

« Mesurer D » marche pour ces modèles (`givre_mesurer` : la fenêtre des petits amas de D2, rayons de 2 à `max(4, étendue / 6)`, donc **3 à 5 points** pour ces tailles) et écrit le même texte « mesuré par le banc : D = … à N sites », la courbe `C(r)` en log-log avec la droite
ajustée, puis **le repère publié de `η`** lu dans le module (`givre_courbe_repere` = `courbe::publie`) : le rappel de la DLA pour `η = 1`, Hastings 2001 pour `η = 2` et `3`, rien pour `η = 0` ; **l'écart du banc au repère, signé** ; et la phrase « ce n'est pas un accord ».
Les phrases du zinc (Z1) ne s'affichent que pour la DLA. **La page n'écrit aucune valeur publiée ni aucune valeur de `D`** : les repères viennent du module (un test lit les fichiers de la page). `D` mesuré par la page sur les modèles à leur taille, graine 1 (moyenne
sur 120 graines, écart-type d'UNE graine) :

| modèle | sites | `D` (graine 1) | `D` moyen (120 graines) | σ | repère | écart du `D` moyen |
|---|---|---|---|---|---|---|
| 4 (`η = 0`) | 400 | 1,8928 | 1,8919 | 0,0136 | — | — |
| 5 (`η = 1`) | 250 | 1,7449 | 1,6905 | 0,0398 | 1,71 (rappel de la DLA) | -0,019 |
| 6 (`η = 2`) | 150 | 1,5961 | 1,4802 | 0,0627 | 1,433 (Hastings) | +0,047 |
| 7 (`η = 3`) | 100 | 1,3830 | 1,3496 | 0,0709 | 1,263 (Hastings) | +0,087 |

**Ces nombres ne sont pas ceux de la courbe de D2** : les tailles diffèrent (400, 250, 150 et 100 sites contre 100 pour la courbe), et `D` en dépend (la dimension monte avec la taille à `η = 0` et `1`, baisse à `η = 2` et `3`, D2). Le texte de la page dit « à N sites ». Une mesure sur un seul
amas (ce que fait « Mesurer D ») a l'écart-type d'une graine (0,014 à 0,071) : l'écart au repère d'une page est du bruit de cet ordre. **Ce n'est pas un accord** : amas petits, corrélation sur un réseau carré contre l'inverse de la pente de `ln F_1` sur des applications conformes,
fenêtre de 3 à 5 rayons (D2 : elle déplace `D` de ±0,05), Hastings sans barre d'erreur.

### La courbe `D(η)` à la demande

Huit exportations (`givre_courbe_demarrer`, `_pas`, `_fait`, `_total`, `_code`, `_moyenne`, `_erreur_type`, `_repere`) : un **travail** de quatre points (`η = 0` à 3) de `graines` amas de `taille` sites (graines 1, 2, …, dans l'ordre ; demi-côté de la règle), avancé **un amas par `givre_courbe_pas`**,
les `η` dans l'ordre 0, 1, 2, 3. Le point d'un `η` est, bit pour bit, celui de `essayer_point_de_la_courbe` ; la moyenne et l'erreur-type d'un `η` sont `NaN` tant que ses amas ne sont pas tous faits. **Les bornes** : `taille` de 1 à **100**, `graines` de 2 à **8**. Les **codes d'erreur** (table dans
la doc de `src/vitrine.rs`) : 0 aucune erreur ; 1 taille hors borne ; 2 graines hors borne ; 3 un amas a touché la frontière ; 4 un amas n'a pas de dimension (fenêtre vide) ; 5 aucun travail ; 6 une autre erreur de la bibliothèque. Un nouveau démarrage efface l'ancien travail, même refusé ;
`givre_demarrer` (le modèle) n'y touche pas. **Les codes 3 et 6 ne sont pas atteignables avec les bornes** (3 : aucun amas de 100 sites n'a touché la frontière avec la règle de D2, 170 graines à D2 ; 6 : le module ne passe jamais un `η` ou un demi-côté hors borne) ; le code 4 l'est :
à 32 sites la graine 1 de `η = 0` et la graine 5 de `η = 1` n'ont pas de fenêtre (étendue de moins de 8), à 30 sites aussi la graine 7 de `η = 0` ; **dès 34 sites les graines 1 à 8 des quatre `η` ont toutes une dimension**.
**Tout l'espace de la courbe de la page a été joué par le juge** (10/11, natif, par `essayer_dimension_d_une_graine` avec le demi-côté de la règle, ce que `givre_courbe_pas` appelle) : `η` de 0 à 3, graines 1 à 8, taille de 1 à 100, soit 3200 amas : **aucune frontière touchée, aucune autre erreur, aucune panique** ; le code 3 est donc inatteignable depuis la page (pas seulement « non observé »), et le code 4 ne l'est que jusqu'à 32 sites (toutes les 32 combinaisons sans dimension de 1 à 8 sites, 31 à 9 et 10, …, 3 de 27 à 31, 2 à 32, aucune à partir de 33). Le pire amas de cet espace a coûté 2,25 s (`η = 3`, graine 3, taille 90, machine chargée).

**Ce que la page calcule** : 100 sites, 6 graines (24 amas) ; le texte d'avancement dit « η = 2, graine 3 sur 6 » ; un `setTimeout` de 20 ms entre deux pas laisse la page se redessiner. Résultat (graines 1 à 6, mesuré) :

| `η` | `D` moyen ± erreur-type | repère | écart |
|---|---|---|---|
| 0 | 1,7601 ± 0,0060 | — | — |
| 1 | 1,6558 ± 0,0278 | 1,71 | -0,054 |
| 2 | 1,5123 ± 0,0407 | 1,433 | +0,079 |
| 3 | 1,3621 ± 0,0348 | 1,263 | +0,099 |

Même signe que D2 pour chaque `η` (au-dessous du rappel de la DLA pour `η = 1`, au-dessus de Hastings pour `η = 2` et `3`) ; les erreurs-types sont celles de 6 graines (de 0,006 à 0,041, de 2 à 4 fois celles de D2 sur 50 graines). **L'ordre `D(0) > D(1) > D(2) > D(3)` se voit ; l'écart au publié n'est pas un accord.** La limite est dite sur la page en clair : amas de 100 sites,
peu de graines, corrélation sur un réseau contre applications conformes, fenêtre.

### Le coût (mesuré)

Temps natif, `release`, un fil (Ryzen 5 5500U, 12 fils, **machine partagée : les temps varient du simple au double d'une série à l'autre, et les séries sont toutes dites**). Le temps d'**un amas de 100 sites** (le pas de la courbe), moyenne et, entre parenthèses, le pire :

| série | `η = 0` | `η = 1` | `η = 2` | `η = 3` |
|---|---|---|---|---|
| A : la croissance du modèle 7, 24 graines, machine au calme | | | | 0,47 s (0,52 s) |
| B : la courbe de la page, 6 graines, machine chargée | 0,002 s (0,003 s) | 0,41 s (0,64 s) | 0,35 s (0,41 s) | 0,92 s (**1,30 s**) |
| C : 24 graines, machine plus chargée encore | 0,004 s (0,017 s) | 0,98 s (1,44 s) | 0,73 s (0,94 s) | 1,09 s (**1,46 s**) |

(La série C redonne, aux quatre décimales, la ligne « graines 1 à 24 » de D2 : `D` = 1,7579 ; 1,6574 ; 1,4980 ; 1,3582, erreurs-types 0,0036 ; 0,0117 ; 0,0155 ; 0,0124 : la chaîne de D3 calcule les mêmes amas que celle de D2.)
La courbe de la page (24 amas à 100 sites) coûte 10 s en natif dans la série B (de l'ordre de 6 s au calme, par extrapolation de A : non mesuré). **« Aucun appel au-delà d'une seconde » n'est donc tenu qu'au calme en natif** : le pire pas (`η = 3`) vaut 0,5 s au calme et 1,3 à 1,5 s quand la machine est
chargée. Série D (le juge, 10/11, par les exportations `givre_courbe_*`, machine **très** chargée par deux autres calculs) : la courbe de la page (100 sites, 6 graines) 22,9 s en tout, pire pas par `η` 0,006 ; 0,72 ; 2,05 ; **2,98 s** ; à 8 graines 26,8 s, pire pas 0,006 ; 1,52 ; 0,95 ; 1,90 s (les mêmes moyennes aux quatre décimales). Sous Node sur la VM (pilote, 10/11) : le pas de `η = 3` à 100 sites coûte 0,75 s, celui de `η = 1` et `η = 2` 0,43 s ; le temps d'un pas dépend donc d'abord de la machine du visiteur : **la page gèle le temps d'un pas**, de moins d'une seconde au calme à trois secondes sur une machine occupée. Le module WebAssembly est plus lent d'un facteur non mesuré ici : si le pas de `η = 3` dépasse la seconde sous Node, `COURBE_TAILLE_MAX` (100) se baisse (à 80 sites un amas coûte à peu près 0,6 fois autant, D2) ou le pas se fait par tranches de sites. **Les tests** (`cargo test --features etape-d3`,
ce PC, profil `dev`) : voir le rapport de l'écrivain.

### Références

- M. B. Hastings, *Fractal to Nonfractal Phase Transition in the Dielectric Breakdown Model*, Phys. Rev. Lett. 87, 175502 (2001), arXiv:cond-mat/0103312v2 : tableau II, colonne « full set » (`η = 2 → 1,433`, `η = 3 → 1,263`) ; **lue à D2** (voir « Références » de l'étape D2) ; D3 ne la relit pas, la page la **lit dans la bibliothèque**.
- J. Mathiesen, M. H. Jensen, J. Ø. Bakke, arXiv:0712.1739, section IV (« the DLA value D=1.71 for η=1 ») : lue à D2.
- Aucune référence nouvelle : D3 est de la plomberie (la page, les erreurs, la borne) autour de D1 et D2.

### Ce que cette étape n'établit pas

- **Aucun accord avec Hastings, ni avec la DLA à 1,71** : la page écrit l'écart, signé, et « ce n'est pas un accord » ; aucun test ne compare un `D` à un repère. La courbe de la page est à 100 sites et 6 graines : l'ordre se voit, rien de plus.
- **Que la page marche dans un navigateur** : la page n'est éprouvée que par sa **forme** (des tests lisent les fichiers comme du texte) et par **Node** (`lecture.js`, sans DOM, contre le natif) ; le DOM, le canvas, les `setTimeout`, le curseur, l'animation et ce que la page montre ne sont éprouvés ni par un test ni par un navigateur
  ici. Un coup d'œil humain dans un navigateur reste nécessaire (Firefox, Safari, téléphone : jamais).
- **Le temps du module WebAssembly** : ni Node ni `wasm32` sur le PC de l'écrivain. Les bornes (100 sites, 8 graines, les quatre tailles des modèles, 30 ms par image) viennent du temps natif et sont **à relever sur la mesure du module**.
- **Que le module ne panique jamais** : tout appel du module est écrit sans `unwrap` ni indice hors de portée et les entrées `essayer_*` ne paniquent pas, mais deux paniques de la bibliothèque restent hors d'atteinte du module (`wasm32` n'attrape pas les paniques) : la garde de convergence de `Claquage::relaxer`
  (« la relaxation ne converge pas », jamais atteinte, D1 : le schéma converge) et `Claquage::essayer_ajouter` (`expect` d'une candidate qui existe toujours). Aucun test ne peut les provoquer. Le juge les a cherchées en natif sur 3300 amas des modèles 4 à 7 (graines 1 à 2000, 300, 500, 500 et 7001 à 7050) et sur les 3200 amas de
  tout l'espace de la courbe de la page : aucune ; et les vingt-trois exportations appelées avec des paramètres extrêmes dans tous les ordres n'ont pas paniqué. Ce n'est pas une preuve pour 4 milliards de graines.
- **Que 4 milliards de graines n'atteignent jamais le bord** : 480 amas de l'écrivain (graines 1 à 120 × quatre modèles) et 3300 du juge ne l'ont pas touché, mais la marge du modèle 7 tombe à **3 sites** sur 500 graines (« Les modèles 4 à 7 et leurs bornes ») : un `givre_avancer` qui s'arrête avant la durée (« Scénario terminé » avant 100 sites) est
  plausible pour quelque graine, sans panique ; le code 3 de la courbe, lui, est inatteignable depuis la page (tout l'espace joué).
- **Que la page ne gèle jamais plus d'une seconde** : un pas de la courbe est un amas entier, synchrone ; il coûte 0,75 s sous Node sur la VM et jusqu'à 3 s en natif sur une machine occupée (« Le coût ») ; borner ce gel demanderait un pas par tranches de sites (un autre contrat du module) ou une taille de 80 (une courbe qui n'est plus celle de D2).
- **Le comportement de la page quand les commandes et le modèle divergent** : la page garde l'état du dernier démarrage réussi (`claquageEnCours`, `etaDuModele`) pour le repère de `η`, le texte d'état et le jeu unité par unité ; une graine invalide laisse l'ancien modèle tourner. Ce n'est éprouvé par aucun test (le DOM n'est pas éprouvé).
- **Les modèles 5 et 6 jusqu'au bout** dans les tests (2 à 5 s chacun en natif) : leur durée est lue à la table au départ, leur état par morceaux jusqu'à 45 unités ; `η = 0` et `η = 3` sont joués jusqu'au bout.
- **`η` non entier, `η >= 4`** (la transition à `η = 4` est un débat ouvert), la 3D, un amas de plus de quelques centaines de sites dans la page (D1), une courbe à 300 sites (12 minutes).
- **Que l'activation dans `default` soit sans effet sur l'état** : elle ne change aucune empreinte (testé par les tests de l'étape 5 et le nouveau test « un claquage ne laisse rien derrière lui ») mais change les octets du module (règle 12 : reconstruire **après** l'activation).
- **L'image** (axes, grille, couleurs) de la courbe de la page : libre ; seule sa forme est lue.

## Étape G1 : la neige de Gravner et Griffeath (le cœur)

Le cousin riche de Reiter, sur la même grille hexagonale : le modèle mésoscopique de J. Gravner et D. Griffeath, **sans bruit** (`σ = 0`, donc sans aucun hasard). Trois masses par case (la vapeur `d` qui diffuse, une couche
« quasi-liquide » `b` au bord de la glace, la glace `c`) et un rattachement `a` ; un pas fait quatre sous-pas dans l'ordre de l'article (diffusion, gel, rattachement, fonte) ; sept réglages (`ρ`, `β`, `α`, `θ`, `κ`, `μ`, `γ`). En virgule fixe (Q31.32) et en entiers :
la symétrie des douze images de l'hexagone est **exacte au bit près**, et la masse est conservée à l'arrondi de la division par 7 près. Le module est `src/neige.rs` (la doc y est la spécification, bit pour bit), la directive de scénario est `neige`, et le **rendu net**
(des hexagones lisses sur fond blanc, calculés à haute résolution) est un module à part, `src/rendu_hex.rs`. **Rien de ce qui existait ne change** (aucune empreinte, ni la page, ni le module web : `neige` n'est appelé que par `lancer`).

### Le modèle, tel qu'on l'a lu

Source (`references/etape-g1-valeurs.md`, V-G1-00 à 04) : la préimpression de l'article (« Revised version, September 2007 »), sections 5 et 6 et annexe 8, avec les figures 9 et 13 vues. L'état au départ est « une masse cristalline 1 à l'origine, une masse diffusive `ρ` partout ailleurs ».
Les quatre sous-pas : (1) la diffusion, moyenne sur sept cases (la case et ses six voisines : un voisin du cristal est remplacé par la vapeur de la case, ce qui « réfléchit » au bord du cristal) ; (2) le gel, une part `κ` de la vapeur d'une case de bord devient glace, le reste masse de bord ;
(3) le rattachement, par le nombre `n` de voisines rattachées : `n = 1` ou 2 et `b >= β`, ou `n = 3` et (`b >= 1` ou (la vapeur du voisinage `< θ` et `b >= α`)), ou `n >= 4` sans condition ; (4) la fonte, une part `μ` de `b` et `γ` de `c` redevient vapeur.

### Les choix à nous (dits, et ce qu'ils changent)

- **Le mur est réfléchissant et fermé** (l'article : un hexagone avec raccord périodique, ou un carré périodique). Une case de la couronne traite un voisin absent comme l'article traite un voisin du cristal. La symétrie reste exacte (le mur est stable par les douze symétries) et la masse est conservée **comme dans l'article** (un domaine fermé).
  Le prototype a mesuré ce que cela change : **le domaine s'épuise**. À `R = 100`, `ρ = 0,5` (les réglages de la figure 9), le cristal s'arrête vers le rayon 88 (la vapeur du domaine est consommée) ; il faut un rayon de domaine d'au moins 1,5 fois celui du cristal (la règle de l'article : 1,5 fois le diamètre final pour une dendrite, 2 fois pour une plaque lente).
  `Neige::rayon_cristal() == R` dit que le cristal a touché le mur. Un bord à vapeur fixée (comme Reiter) aurait une réserve infinie : ce n'est pas le modèle de l'article, et il n'est pas offert.
- **L'arrondi** : la moyenne sur sept cases est la somme exacte de sept entiers divisée par 7 **au plus proche** (`Fixe::div_entier(7)` ; jamais de demi-cas, 7 étant impair). Les produits par `κ`, `μ`, `γ` sont les produits `Fixe` (au plus proche), et chaque « reste » est repris exactement (`b = d1 - g`, la fonte retire ce qu'elle rend) : seule la division par 7 perd ou crée de la masse.
- **La fonte ne touche que les cases non rattachées** (« plus de dynamique aux cases rattachées » : l'article) : une case qui vient de se rattacher ne fond pas. **La vapeur lue pour `θ`** est celle d'après le gel (un `d°` est la valeur au début du sous-pas) : sur une case de bord, elle est nulle.
- **Les plages des réglages** (l'article n'en dit pas) : `ρ, α, κ, μ, γ` dans `[0, 1]`, `β` dans `[1, 8]`, `θ` dans `[0, 8]`, bornes incluses ; chacune a sa raison dans la doc (une proportion, `β > 1` de l'article, rien ne déborde). Les réglages de l'article y sont tous.
- **Les lectures de la source** : la préimpression écrit `n >= 3` en (3b) et `n >= 4` en (3c) (même résultat) ; sa prose de l'instabilité échange `θ` et `α` par rapport à l'équation (3b), que ce module suit (avec l'ordre des colonnes de l'annexe : les valeurs de l'annexe sont cohérentes avec (3b)). **La version publiée dans *Physica D* n'a pas été lue** : si elle diffère de la préimpression, la doc est à corriger.
- **Un état et des entrées qui ne paniquent pas** : `Neige::essayer_new` et `essayer_depuis_cases` rendent un `Result` ; `pas` ne panique jamais sur un état permis (la masse totale est `< 2^56` au départ, donc aucune somme de sept masses ne dépasse `i64`, pour les 10^10 premiers pas).

### La masse, et sa dérive (mesurée)

La doc fixe la borne : à chaque pas `|M' - M|` est au plus `3 C / 7` unités du dernier chiffre (`C` : les cases non rattachées), et les sous-pas 2 à 4 sont exacts. **Les tests ne fixent aucune tolérance empirique** : la borne est déduite de la formule de l'arrondi (V-G1-MASSE-01), et la conservation est
exacte quand les vapeurs sont des multiples de 7 (V-G1-MASSE-02). **Mesuré** (`mesure-derive.txt` : `fig9` à `ρ = 0,5` et `0,4`, `fig13m`, `jeuC` ; rayons 20 et 60 ; 500 et 2000 pas) : la dérive totale va de `-2340` à `+9828` unités (une unité vaut `2^-32 ≈ 2,3e-10`), soit de `-0,9e-3` à `+1,7e-3` unité par case et par pas ; la borne est de 165 à 2640 fois plus grande
(par exemple 8 millions d'unités pour `R = 60` et 2000 pas, contre 5580 mesurées). La dérive est **de signe variable à petite échelle et le plus souvent positive** : de l'ordre de `+1e-3` unité par case et par pas ; sur un cristal de 20 000 pas à `R = 400` elle est de `+7,7e6` unités, soit `7e-9` de la masse totale. Négligeable.

### La précision de la virgule fixe (mesurée)

Q31.32 a un pas de `2,3e-10`. Les petits réglages (`γ = 5e-5`, `μ = 0,07`, `κ = 0,003` pour la figure 9 ; `γ = 1e-5` et `κ = 1e-4` pour la figure 13 au milieu) sont lus exactement par `lire_decimal` (`γ = 0,00005` donne `214748` brut : une erreur relative de `1,7e-6`). **Le prototype est comparé à une
référence en précision large** (`g1_ref.py precision` : les mêmes équations en `decimal`, 70 chiffres, avec les réglages exacts ou arrondis à Q31.32), `R = 22`, 400 pas, trois jeux (`ρ = 0,5` et `0,4` de la figure 9, la figure 13 au milieu) : **l'écart de chaque masse (`b`, `c`, `d`) à la précision large reste sous 13 unités du dernier chiffre (`3e-9`)** pendant les 400 pas
(l'écart dû à l'arrondi des réglages à Q31.32 est d'au plus 10 unités, celui de l'arithmétique d'au plus 13), et **les ensembles de cases rattachées sont identiques à chaque pas, pour les trois jeux** : les décisions de rattachement ne basculent pas. L'écart croît lentement avec le temps (de 1 à 5 unités vers 50 pas, de 5 à 13 unités à 400 pas). **Q31.32 suffit à ces échelles** ; ce qu'il en est à 10^5 pas
(l'écart ne se mesure pas aussi loin avec `decimal` : trop lent) n'est pas établi, et un cristal peut finir par prendre un autre chemin si une comparaison de seuil bascule (les dynamiques sont chaotiques) : si cela compte pour G2, passer à 128 bits est une décision du pilote ; **il n'est pas fait ici**.

**Mesure du juge, plus longue** (un autre outil, `juge_precision.py` : les mêmes équations en entiers à 128 bits après la virgule, réglages exacts) : 2000 pas, `R = 22` à 30, sept jeux (les deux de la figure 9, le milieu de la figure 13, les deux lectures de la droite de la figure 13, `JEU_D`, la ligne « 10 » de l'annexe) : **les ensembles de cases rattachées restent identiques à chaque pas pour les sept jeux**, et l'écart d'une masse atteint 24 à 505 unités du dernier chiffre à 2000 pas (au plus `1,2e-7` ; 10 à 22 unités à 400 pas) : il croît plus vite que le temps (les arrondis s'accumulent sur les cases de bord), et rien n'est mesuré au-delà.

### Le coût (mesuré)

Natif, profil `dev` à `opt-level = 3` (le même temps que `release`, à 10 % près), depuis l'état de départ (`ρ = 0,5`, figure 9). Environ **22 à 30 ns par case et par pas** (quatre sous-pas de boucles simples sur des tableaux ; pas de parallélisme ; **remesuré par le juge sur le même PC : 19 à 40 ns, et jusqu'à 67 ns sur une mesure à `R = 200`** : 5,7 s pour 5000 pas à `R = 100`, 24 s à `R = 200` ; prendre 1 s par 1000 pas à `R = 100` et 5 à 8 s à `R = 200`) :

| rayon | cases | 500 pas | 5000 pas | ms par pas |
|---|---|---|---|---|
| 20 | 1 261 | 0,014 s | 0,12 à 0,14 s | 0,028 |
| 50 | 7 651 | 0,09 s | 0,82 s | 0,17 à 0,19 |
| 100 | 30 301 | 0,41 s | 3,5 à 3,9 s | 0,71 à 0,85 |
| 200 | 120 601 | 1,5 à 1,8 s | 15,2 à 15,9 s | 3,0 à 3,2 |

**Un rayon de 100 coûte donc 0,8 s par 1 000 pas** (3,2 s pour un rayon de 200). **Un cristal de la taille de l'article** (un diamètre de 400 à 600 cases, 10 000 à 100 000 pas) : mesuré une fois, `R = 400` (481 201 cases), `ρ = 0,5`, 20 000 pas : **239 s** (24,8 ns par case et par pas ; le cristal a alors le rayon 351 et 101 371 cases : il approche du mur, un domaine plus grand serait
nécessaire) ; 100 000 pas, **extrapolés et non mesurés**, coûteraient de l'ordre de 20 minutes. Mémoire : de l'ordre de 60 octets par case (`R = 400` : 30 Mo ; `R = 2047` : 750 Mo). **La création** de la neige est négligeable (50 ms à `R = 400`). Les tailles des tests (de 3 à 30 de rayon, de 25 à 4000 pas) tiennent les 41 épreuves de G1 en moins de 3 secondes.

### Le rendu net (`src/rendu_hex.rs`)

Le constat d'André : les aperçus étaient pixelisés parce que le dépôt dessinait **un pixel par case puis agrandissait** (`image-rendering: pixelated`, `rendu.rs`, `apercus/*.png`). Ce module dessine la case pour ce qu'elle est :

- **chaque case est un hexagone à sa vraie géométrie** : à plat en haut (côtés haut et bas horizontaux), d'écart 1 entre voisines, le bras `(1, 0)` de la grille vers le haut (les figures de l'article ont un bras vertical). `√3` n'est jamais calculé : l'inverse `1/√3` est `1239850262 / 2147483647` (écart relatif `2,6e-10`, invisible), ce qui rend tout le calcul entier ;
- **une couleur continue tirée de la masse cristalline `c`** (`Neige::valeurs_du_cristal` : `c` des cases rattachées, 0 ailleurs) sur **un dégradé de bleus clair puis foncé**, **fond blanc** (la case sans glace est exactement `(255, 255, 255)`), cinq arrêts en entiers. **Pourquoi `c`** : c'est le champ que l'article dessine (« les gris dans le cristal représentent `c` »), il est déjà dans l'état,
  et il montre l'épaisseur de la glace (les plaques minces remplies après coup, `c` près de 0,1, sont claires ; les branches, `c` de 1 à 1,45, sont plus foncées). **Ce que le modèle offre de mieux** : « l'instant où la case a gelé » montrerait l'ordre d'apparition (les « ribs » de l'article) mais n'est pas dans l'état ; le garder coûte un tableau d'entiers de plus (4 octets par case, hors de l'écriture canonique et de l'empreinte) :
  **une décision pour G3**, pas faite ici ;
- **haute résolution et lissage** : `sur × sur` échantillons par pixel (4 dans les exemples : 16 niveaux de couverture au bord d'une case), la case d'un échantillon par un arrondi cubique **en entiers** (`i64`), la couleur du pixel la moyenne entière : les octets sont les mêmes sous Linux et sous Windows, l'image d'un état symétrique est **exactement** symétrique (miroirs et rotation de 180°), aucun agrandissement au plus proche voisin.

**Le coût** (natif, mesuré, `mesure-rendu.txt`) : environ **15 à 18 ns par échantillon**, quel que soit le rayon : une image de `1200 × 1200` pixels à `sur = 4` coûte **0,35 s** (R = 100) à 0,41 s (R = 200) ; `1600 × 1600` : 0,64 à 0,75 s ; `2400 × 2400` : 1,4 s ; `sur = 8` coûte quatre fois plus (2,4 à 2,6 s à `1600`). L'écriture du PNG de `ecrire_png` (**non compressé**) ajoute 0,03 à 0,1 s et
fait `3 * cote²` octets : 4,3 Mo à 1200 px, 7,7 Mo à 1600, 17 Mo à 2400 : **les aperçus du dépôt pèsent donc des mégaoctets** (un PNG compressé serait une autre étape).
**Les aperçus** (`examples/apercus_etape_g1.rs`, hors ligne ; réglages de la figure 9, relus à la source) : `apercus/etape-g1-cristal.png` (`ρ = 0,46`, 8 000 pas, rayon 100, 1200 px : 12 s de simulation + 0,4 s de dessin) et `apercus/etape-g1-planche.png` (`ρ` de 0,40 à 0,50 par 0,01 : **onze cristaux**, rayon cible 100 dans un
domaine de 150, 300 px chacun, 1800 × 600 px : **de 240 à 320 s de simulation** (4 à 5 minutes, selon que la machine est occupée) + 0,3 s de dessin ; `ρ = 0,40` et `0,41` n'atteignent pas le rayon cible et s'arrêtent à 40 000 pas, aux rayons 88 et 97). **On voit** la suite de la figure 9 : la plaque à secteurs (`ρ = 0,40`), l'étoile à plaques, puis des dendrites à branches latérales de plus en plus fines. **Ce qui n'est pas là** : la finesse du repère
(onze cristaux de 400 à 600 cases), le contraste de teinte du repère (ici `c` ne varie que de 1,0 à 1,45 sur les branches), et la ressemblance n'est pas une preuve.

### Ce qui est prouvé et comment

41 tests, tous étiquetés `juste` ou `régression`, avec leur ligne `Ne couvre pas` : le naïf du test contre la bibliothèque (états sans symétrie, 40 graines ; neuf trajectoires ; toutes les branches du rattachement exercées), le premier pas, le mur, les règles une à une, le rattachement synchrone et `θ` strict **calculés à la main**, huit trajectoires du code Python indépendant, la symétrie des douze images à chaque pas
et l'équivariance, la masse (borne et exactitude), des régimes, la directive et le binaire, et le rendu (un hexagone calculé à la main, un naïf de géométrie exacte, le dégradé, la symétrie de l'image, une empreinte de référence). **Fautes volontaires** : **58 des 59 fautes volontaires** (une moyenne sur six cases, la division par 7 vers le bas, une diffusion en place, l'ordre des sous-pas, chaque seuil `>` pour `>=` et `<=` pour `<`, `β`, `α` et `θ` échangés, `κ` pour `1 - κ`, la fonte de `c` au lieu de `b`, `γ` oublié, le mur à 0, une voisine oubliée, un rattachement lu après mise à jour, une case de bord comptée comme cristal, un germe décalé, de la masse créée ou perdue, et quatorze fautes du rendu : une grille décalée, des hexagones tournés, un dégradé inversé, un fond noir, une couverture comptée deux fois…) **font échouer au moins un test** ; la 59e (`rendu_sans_plafond`) est un mutant **équivalent** : le plafond de `t` est déjà porté par `t >= 1024`. Le tableau est dans `essai-g1-outils/table-fautes.md`.

### Les valeurs publiées, dites telles quelles

**Aucune valeur de forme n'est un test** : les figures 9 et 13 de l'article n'ont aucun nombre (des images et des réglages). Le jeu de la figure 9 (`β = 1,3`, `α = 0,08`, `θ = 0,025`, `κ = 0,003`, `μ = 0,07`, `γ = 0,00005`, `ρ` de 0,40 à 0,50) est une entrée relue à la source, sans contradiction ; le jeu du milieu de la figure 13 aussi. **Les deux autres cristaux de la figure 13 ont un jeu que le texte et l'annexe donnent
différemment** (`α` et `θ` échangés et `γ` d'un facteur 10 à gauche ; `γ = .00006` ou `.0006` à droite) : ils ne sont pas employés comme ceux de l'article. À `R = 400`, 20 000 pas, `ρ = 0,5`, le prototype donne une dendrite à branches latérales (vue, pas testée) ; à petit rayon (`R = 30`, 500 pas) le cristal est un astérisque à six pointes (V-G1-REG-07, régression).

### Références

- J. Gravner et D. Griffeath, « Modeling snow crystal growth II: A mesoscopic lattice map with plausible dynamics », *Physica D* 237 (2008) 385-404, https://doi.org/10.1016/j.physd.2007.09.008 : **lue** dans sa préimpression (« Revised version, September 2007 », `mk.bcgsc.ca/snowflakes/paper/modeling-snow-crystal-growth-2.pdf`), section 5 (équations (1) à (5)), section 6, annexe 8 (réglages des figures), figures 9 et 13 vues ; **non lue** : la version publiée, les figures 10 à 12 et 14 à 17.
- J. Gravner et D. Griffeath, « Modeling snow-crystal growth: A three-dimensional mesoscopic approach », *Phys. Rev. E* 79, 011601 (2009), arXiv:0711.4020 : lue seulement pour la diffusion (mêmes poids `1/7`, « reflecting boundary conditions at the crystal's edge ») et la définition du bord du cristal ; ses règles d'attachement sont celles de la 3D.
- C. A. Reiter, *Chaos, Solitons & Fractals* 23 (2005) 1111-1119 : le modèle de l'étape 3, dont ce module reprend la grille, la virgule fixe et la preuve de symétrie.

### Ce que cette étape n'établit pas

- **Que la neige ressemble à celle de l'article, ou à une vraie neige** : aucune forme publiée n'est testée, et l'article ne donne aucun nombre pour ses figures. Le prototype montre à grande taille une dendrite à branches latérales et, de `ρ = 0,40` à `0,50`, la suite plaque, étoile, dendrite de la figure 9 : c'est vu, pas démontré, et le détail n'est pas celui de la figure (la finesse des branches de l'article
  à 400 à 600 cases n'est pas montrée par la planche).
- **Ni la forme selon la température** (le diagramme de Nakaya : les auteurs disent eux-mêmes ne pas l'expliquer ; passer de (température, sursaturation) aux sept réglages est une question de recherche ouverte), **ni le bruit** (`σ`), **ni la 3D**.
- **Que le mur soit celui de l'article** (périodique) : le domaine fermé s'épuise (« Le mur est réfléchissant ») ; des cristaux qui touchent le mur ne sont pas ceux de l'article.
- **Que les versions publiée et préimprimée soient les mêmes** : la version de *Physica D* n'a pas été lue (ni par l'écrivain ni par le juge : l'éditeur refuse l'accès ; la copie du site de l'auteur, `math.ucdavis.edu/~gravner/papers/h2l.pdf`, est le même fichier que la préimpression lue, octet pour octet) ; les contradictions relevées (V-G1-03) ne sont pas arbitrées par la publication.
- **La précision de Q31.32 au-delà de 2000 pas** (`R = 22` à 30) ni à 10^5 pas : rien n'est mesuré (l'écart croît plus vite que le temps : une comparaison de seuil peut basculer à un pas lointain) ; la dérive de masse est mesurée jusqu'à 20 000 pas.
- **Le coût dans un navigateur** : jamais mesuré (ni Node ni `wasm32` sur ce PC) ; le natif donne 0,8 s par 1 000 pas à `R = 100`. **Le module web** : `neige` et `rendu_hex` ne sont appelés par aucune exportation de `vitrine.rs` (vérifié par lecture) ; ajouter `etape-g1` à `default` change pourtant les octets de `docs/givre.wasm` (la section des noms seulement, mesuré sur la VM : règle 12 d'`AGENTS.md`), que la vague du code reconstruit.
- **Que les images soient belles** : le rendu est jugé par des calculs exacts et une empreinte, jamais par l'œil ; la page (le canvas, la fin de `pixelated` pour la neige) est l'étape G3.

## Étape V7 : le rendu de la page (les flocons de Reiter en hexagones lisses)

Le constat d'André : la page montrait des images « pixelisées ». Pour les flocons de Reiter (modèles 1 à 3, rayons 30, 40 et 60), elle dessinait **un pixel par case** (une image de 61 à 121 pixels) et le navigateur l'agrandissait
(`image-rendering: pixelated`) en gros carrés. G1 a écrit le moteur qui dessine chaque case comme un **hexagone lisse** (`rendu_hex::rendre` : hexagones à leur vraie géométrie, dégradé de bleus clair puis foncé sur fond blanc,
suréchantillonnage en entiers, aucun pixel agrandi) ; **V7 le met dans la page**, pour les modèles sur la grille hexagonale. La DLA et le claquage (grille carrée) gardent leurs cases-carrés et leur `pixelated` : un pixel par case y est leur
vraie géométrie (André décidera s'il veut les adoucir : une autre étape). Les fichiers : `src/vitrine.rs` (quatre exportations de plus : **vingt-sept** en tout), `docs/index.html`, `docs/app.js`, `docs/lecture.js`, `docs/givre.wasm` (reconstruit
après l'activation de `etape-v7`), `MODELE.md`, `references/etape-v7-valeurs.md` (les valeurs attendues des tests, chacune avec sa source). **Rien de ce qui existe ne change** : les vingt-trois exportations d'avant et l'image d'un pixel par
case qu'elles rendent (les tests de l'étape 5 la lisent), l'état, l'empreinte affichée, les scénarios. **La règle des nombres ne change pas** (`+ - * /` sur des entiers ; `rendre` est tout en entiers : les octets sont les mêmes sous Linux
et sous Windows).

### Ce que la page montre

Deux canvas : `#vitrine` (un pixel par case : la DLA et le claquage ; **`image-rendering: pixelated` n'est écrit que pour lui**) et `#vitrine-nette` (les flocons : `image-rendering: auto`, un fond blanc, une légende qui dit « hexagone »).
La page demande **toujours** l'image nette (`lireImageNette` de `lecture.js`) : le module répond par un code, et le code 1 (« pas de grille hexagonale ») fait retomber la page sur l'image d'un pixel par case et son canvas carré. **La page ne sait donc pas
quels modèles sont hexagonaux** : la neige (G3) y entrera sans qu'on touche à la page. L'animation dessine à chaque image une **image légère** (384 pixels de côté, 2 × 2 échantillons par pixel) et, **quand le scénario s'arrête, une image plus fine**
(1024 pixels, 3 × 3 : réduite à l'écran par le navigateur, jamais agrandie en carrés). Le volet caché d'un navigateur fige `requestAnimationFrame` (c'est connu : l'animation attend qu'on le rouvre).

### Le champ dessiné (la décision de l'écrivain, à relire)

`rendre` veut une valeur entière par case, dans l'ordre canonique de `Hexagone`. **Pour un flocon de Reiter, la masse d'eau des cases gelées** (`Flocon::masses`, la seule grandeur du modèle : il n'y a ni température ni épaisseur) : une case gelée gagne `γ` à chaque
pas et ne reçoit rien d'autre (ses voisines sont réceptives : leur masse qui diffuse est 0), sa masse **suit donc son âge**, à ce qu'elle avait en trop en gelant près (le dégradé montre le cristal qui s'épaissit : le germe est foncé, la pointe d'une
branche est claire ; mais une case qui gèle peut dépasser 1 : dans la dendrite six cases gelées après lui atteignent ou dépassent la masse du germe, 1,8, et saturent comme lui : **la couleur est une masse, pas une date**, et la légende de la page le dit ainsi).
La doc de `src/vitrine.rs` fixe au bit près : `E = γ.brut × durée` (ce que le germe gagne pendant tout le scénario : `85 899 346 000`, `51 539 607 600`, `3 435 973 600` pour la plaque, l'étoile, la dendrite), `valeur = 0` pour une case de masse
inférieure à 1 (le fond, la vapeur, les cases de bord : **blanc** même pour `β = 0,45`), `valeur = 3 (brut − UN) + E` pour une case gelée, `plein = 4 E` : le dégradé `t = 256 + 768 (brut − UN) / E`, plafonné à 1024. **Une case tout juste gelée est au premier
arrêt (`t = 256`, un bleu clair visible sur le blanc), le germe à la fin du scénario au dernier (`t = 1024`, le bleu foncé)** ; la vue est le carré de `2 R + 1` cases centré sur la case `(0, 0)`.
**Ce qui a été essayé et refusé** (images regardées, `essai-v7-outils/champ-*.png`) : la masse brute avec `plein` = la masse du germe à la fin (`1 + γ T`) : le bord d'une plaque (`1 / 21` du plein) est presque blanc, le cristal « fond » dans le fond ;
une masse brute avec un `plein` fixe (2, 3 ou 5) : saturée pour la plaque et l'étoile, trop pâle pour la dendrite (masses de 1 à 1,84) ; la racine carrée de `masse − 1` (`math::racine_entiere`) avec un plancher : des bleus plus soutenus, mais deux
nombres de plus à fixer sans gain de lecture ; **un plancher d'un quart de l'échelle** (celui retenu : `3 (brut − UN) + E` revient à `t = 256` pour une case tout juste gelée) garde la loi affine (un calcul exact à la main, V-V7-11) et un bord visible pour les trois modèles.
**L'instant du gel** (le champ que G1 jugeait le plus parlant) n'est pas dans l'état : le garder demanderait un tableau de plus hors de l'empreinte ; **ce n'est pas fait**, et pour Reiter la masse en tient lieu (l'âge de la case, à `γ` près).
**Contre le repère d'André** (onze neiges de Gravner et Griffeath) : on retrouve des hexagones lisses, un dégradé de bleus et un fond blanc ; on ne retrouve ni la finesse (les cristaux de Reiter à 30, 40 et 60 de rayon n'ont que 1 000 à 2 300 cases :
une case fait 8 à 17 pixels sur une image de 1024) ni la variété de teintes (un seul champ, une seule loi).

### Les quatre exportations, les bornes et les codes

`givre_nette_preparer(cote, sur)` prépare l'image du modèle en cours dans un **tampon** et rend un code ; `givre_nette_largeur`, `givre_nette_hauteur` et `givre_nette_pixel(x, y)` (`0x00RRGGBB`, comme `givre_pixel`) le lisent sans rien recalculer. **Le tampon
appartient à l'état qu'il montre** : `givre_demarrer` qui réussit l'efface, `givre_avancer` l'efface dès qu'il joue une unité, une préparation qui échoue l'efface (jamais l'image d'un autre état). Les codes, vérifiés dans cet ordre : 2 rien n'est démarré, 1 le
modèle n'est pas sur la grille hexagonale (DLA, claquage), 3 `cote` impair ou hors de 2 à 2048, 4 `sur` hors de 1 à 4, 5 `cote × sur` au-delà de 4096, 6 `rendre` a refusé (jamais, avec ces bornes). **Aucune panique** (un code, y compris pour `u32::MAX`).
**Les bornes viennent du coût** : 4096 échantillons de côté (16,8 millions d'échantillons) coûtent 0,24 à 0,30 s en natif au calme, 0,7 s chargé, et une image de 2048 × 2048 pèse 12 Mio ; au-delà (`(2048, 4)` : 0,94 à 0,99 s) une préparation figerait la page
(D3 : « aucun appel au-delà d'une seconde environ »). **Le brief suggérait** « 15 à 18 ns par échantillon, 40 ms à 800 × 800 à `sur = 2` » (G1) : **mesuré ici** 15 à 18 ns par échantillon dès `sur >= 2` (**30 ns à `sur = 1`** : le coût d'un pixel s'ajoute) ;
800 × 800 à `sur = 2` coûte **44 à 47 ms** (pas 40) au calme.

### Le coût (mesuré) et ce que la page en fait

Natif, profil `dev` à `opt-level = 3` (`essai-v7-outils/v7_mesures.rs`, 7 essais, un poste de 12 fils **partagé : le même calcul coûte jusqu'à 1,5 fois plus quand il est chargé** ; les chiffres sont ceux du calme) : le rayon du flocon (30, 40, 60) ne change pas le coût.

| côté, `sur` | échantillons | temps natif |
|---|---|---|
| 256, 2 | 262 144 | 4,7 ms |
| **384, 2 (l'animation de la page)** | 589 824 | **10,3 à 10,8 ms** (17 à 27 ms chargé) |
| 512, 2 | 1 048 576 | 18,4 à 18,8 ms |
| 800, 2 | 2 560 000 | 44,7 à 46,6 ms |
| **1024, 3 (la fin de la page)** | 9 437 184 | **143 à 162 ms** |
| 1024, 4 et 2048, 2 (la borne) | 16 777 216 | 241 à 268 ms et 290 à 304 ms |
| 2048, 4 (hors borne) | 67 108 864 | 0,94 à 0,99 s |

La lecture pixel par pixel coûte en plus 1,8 ms pour 384² pixels et 13 ms pour 1024² (natif : 12 ns par appel). **Une image d'animation coûte donc de l'ordre de 12 ms en natif** (préparer et lire). Le juge a refait la mesure sur son poste (7 essais, la vitrine
elle-même) : `(384, 2)` 13 à 14 ms et 2,5 ms de lecture, `(1024, 3)` 190 ms et 18 ms, `(2048, 2)` 360 ms et 71 ms, 20 à 24 ns par échantillon, les trois flocons confondus ; un `givre_avancer(5)` coûte 0,4 à 1,8 ms : le rendu domine.
**Le module WebAssembly sous Node** (mesuré par le pilote sur la VM de preuve, Linux, Node 20, le module reconstruit avec le prototype) : **26 ms pour préparer l'image d'animation `(384, 2)`, 28 ms avec la lecture** ; **375 ms pour celle de la fin `(1024, 3)`,
388 ms avec la lecture** ; 770 ms pour la borne `(2048, 2)`. La borne de départ (une image d'animation sous 50 ms, la dernière sous 1 s) est donc tenue sous Node : une trentaine d'images par seconde au plus pour un flocon, et un arrêt de 0,4 s, une seule fois,
à la fin du scénario. Un appareil trois à cinq fois plus lent (un téléphone) ferait 7 à 12 images par seconde et une à deux secondes pour la dernière image : l'animation ralentit, elle ne fige pas (une image par `requestAnimationFrame`) ; **ce n'est pas mesuré**.
La page garde `BUDGET_IMAGE_MS` du claquage pour ses modèles carrés : l'image nette est un calcul court, non découpé.

### Ce qui est prouvé et comment

Tous les tests sont **exacts** (aucune tolérance). La vitrine : l'image est, octet pour octet, `rendre` appliqué au champ de la doc (recalculé dans le test d'après un `Flocon` construit en face), pour plusieurs états et réglages ; les bornes (4096 inclus) et les codes dans
leur ordre ; une préparation ne change rien à l'état ni à l'image d'un pixel par case ; le tampon appartient à l'état ; les modèles carrés n'ont pas d'image nette ; jamais de panique. Le rendu : un calcul à la main (le germe : quatre pixels `(241, 246, 251)`,
puis `(198, 219, 239)`, `(130, 185, 220)`, `(70, 144, 198)`, `(8, 48, 107)` à quatre instants), la couleur case par case le long d'un bras, et quatre empreintes de référence **recoupées par un second code Python** écrit d'après la doc seule (`régression`). Le module
(Node, une fois reconstruit) : les mêmes vérités lues par `lecture.js`. **Avant tout démarrage** (le code 2, le tampon vide) : en natif aussi, par un fichier d'un seul test (`tests/etape_v7_neuf.rs` : un processus neuf). Le champ d'un bras de la plaque à 400 pas
se calcule **à la main** (`β = 0` : rien ne diffuse, l'anneau `q` gèle au pas `20 q`, sa masse est `(420 − 20 q) γ`) et le test l'exige case par case. La page : `image-rendering: pixelated` n'est pas sur le canvas des hexagones (les règles CSS sont lues : une liste blanche de sélecteurs), `app.js` dessine l'image nette sur son canvas, les
réglages sont dans les bornes, et `lireImageNette` / `imagesCachees` sont éprouvés sous Node sur un **faux module non carré qui change d'état**. Les tests des étapes 5 et Z1 qui comptent les exportations sont **amendés** (sans `etape-v7` : l'ancien ensemble ou les
vingt-sept ; avec : les vingt-sept seulement).

### Références

- G1 (`src/rendu_hex.rs`, ci-dessus) : le moteur de rendu, sa géométrie, son dégradé et ses bornes ; aucune référence nouvelle n'est lue pour V7 (le rendu n'est pas une mesure).
- C. A. Reiter, *Chaos, Solitons & Fractals* 23 (2005) 1111-1119 : le modèle de l'étape 3 (la masse d'eau, les cases gelées, `γ`), déjà cité.

### Ce que cette étape n'établit pas

- **Que la page marche dans un navigateur** : seule sa forme (`index.html`, `app.js`, `lecture.js` lus comme du texte), `lecture.js` sous Node avec un faux module, et le module sous Node une fois reconstruit. Que le canvas des hexagones ne soit pas agrandi en carrés
  par le navigateur (`image-rendering: auto` sur un canvas réduit ou agrandi de moins de 2) **se voit, il ne se prouve pas** : un coup d'œil humain avec le vrai module reste nécessaire. L'animation (sa fluidité) ne se voit qu'avec le module reconstruit.
- **Le temps dans un navigateur, sur un appareil lent** : le module est mesuré sous Node sur la VM (voir « Le coût »), pas dans un navigateur ni sur un téléphone.
- **Que l'image soit belle, ou fidèle au repère** : le rendu est jugé par des calculs exacts, jamais par l'œil ; le champ (la masse d'eau des cases gelées) est un choix de l'écrivain, lisible mais pas celui de l'instant du gel ; les teintes sont moins variées que celles du repère.
- **Un champ lu dans un autre ordre de cases qui soit une symétrie de l'hexagone** (les douze symétries) : l'état d'un flocon est exactement symétrique, l'image est la même, aucun test ne peut voir cette faute-là. **Une largeur et une hauteur échangées** : l'image est carrée.
- **Un `>>> 0` oublié sur un pixel** (la couleur `0x00RRGGBB` est positive : la faute ne change rien) ; **le plancher et le plafond de `E`** (1 et `2^58` : aucun des trois modèles ne les atteint, aucun test ne peut les voir) ; **le code 6** (inatteignable).
- **Ce que les tests de forme de la page laissent passer** : un script qui poserait `pixelated` par une chaîne fabriquée en morceaux, un style venu d'ailleurs que des `<style>` de la page ; les tests lisent du texte, pas un navigateur.
- **Les modèles carrés lissés** (la DLA, le claquage), **la neige** (G3) ; **Firefox, Safari, un téléphone** : jamais vus.
