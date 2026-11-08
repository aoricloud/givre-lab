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
