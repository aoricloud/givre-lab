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
venue de ses voisines non réceptives) ; la
couronne extérieure est remise à β à chaque pas ; l'état initial vaut β partout, sauf le germe à 1. Les règles, l'ordre et
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

### Ce qui est mesuré, et l'écart à la valeur publiée

**Valeur publiée non atteinte, cause établie** : la table de Reiter (région de rayon 100) est reproduite exactement par ce modèle
sur un hexagone de rayon 100 pour 160 des 219 cases non plafonnées (toutes celles où β ≥ 0,55 ou γ ≥ 0,1). Pour β ≤ 0,5 et γ ≤ 0,05 ce
modèle met de 0,1 à 4,9 % de pas en plus : le script de l'auteur fixe à β les cases à distance euclidienne d'au moins 100 sin 60° ≈ 86,6
du germe (un cercle), alors que la couronne de l'hexagone est plus loin des pointes ; avec la géométrie de l'auteur et les mêmes
règles, un modèle écrit à part retrouve les 240 cases. Le bord circulaire n'est pas dans la bibliothèque (un bord en flottants, ou
en carrés de distances entiers, serait un changement de conception) ; le test de la table ne porte que sur des cases où les deux
géométries donnent le même nombre.

### Ce que cette étape n'établit pas

- La fidélité à la table pour les 59 cases des dendrites lentes : établie seulement par un outil à part, avec un bord circulaire
  que la bibliothèque n'a pas ; aucun test gelé ne la porte.
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
