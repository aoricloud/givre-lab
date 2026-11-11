# Aperçus de l'étape 0

- `etape-0-graine-1.png` : la graine seule, une image de 128 × 96 pixels de bruit tiré par PCG32 avec la graine 1 (flux 54).
- `etape-0-graine-2.png` : la graine seule, une image de 128 × 96 pixels de bruit tiré par PCG32 avec la graine 2 (flux 54).

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues sont dans `references/etape-0-valeurs.md` et les
empreintes dans `references/*.empreinte` ; embellir un aperçu ne change aucune référence.

## Aperçus de l'étape 2s

- `etape-2s-fronts.png` : les quatre modèles de croissance de surface côte à côte, dans l'ordre dépôt aléatoire, relaxation, balistique et Eden, sur L = 128 colonnes et 96 couches avec la graine 1, chaque rangée colorée selon la couche où le front l'a atteinte.
- `etape-2s-rugosite.png` : les courbes de rugosité w(t) des quatre modèles en échelle log-log (L = 256, instants 1 à 256 par puissances de 2, w² moyenné sur 8 graines), un panneau et une couleur par modèle, dans le même ordre (bleu pour le dépôt, orange pour la relaxation, vert pour le balistique, violet pour Eden).

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.

## Aperçus de l'étape 1

- `etape-1-amas.png` : l'amas de DLA de la graine 1 (4000 sites, réseau de demi-côté 200), chaque site coloré par son ordre d'arrivée, du blanc pour le germe au rose pour le dernier site.
- `etape-1-loglog.png` : la courbe log-log de la masse dans le rayon autour du germe contre le rayon (rayons 2 à R), avec deux axes, leurs repères et la droite de pente 1,71 en repère pointillé.

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.

## Aperçus de l'étape 3

- `etape-3-planche.png` : neuf flocons de Reiter (α = 1, hexagone de rayon 60, 250 pas) en grille 3 × 3, β = 0,3, 0,45 et 0,6 en colonnes et γ = 0,001, 0,01 et 0,1 en lignes, chacun l'aperçu de `Flocon::apercu`, séparés par un filet.
- `etape-3-plaque.png` : l'aperçu, écrit tel quel, du scénario `scenarios/reiter-plaque.txt` (hexagone de rayon 30, α = 1, β = 0, γ = 0,05, 400 pas).

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.

## Aperçus de l'étape 4

- `etape-4-faces.png` : cinq faces de 64 × 64 colonnes à Δμ = 0 (J/kT = 2 ; 1,5 ; 1 ; 0,75 ; 0,5), après 300 balayages depuis une face plate (graine 1), chacune l'aperçu min-max de `Face::apercu`, séparées par un filet : de la face lisse, uniforme, à la face rugueuse.
- `etape-4-rugosite.png` : la rugosité d'équilibre `w` contre `J/kT`, de 0,25 à 2 par pas de 1/16, pour L = 16 et L = 32 (une couleur par taille), en échelle logarithmique en `w`, avec les deux axes, la grille et leurs repères, sans texte ; chaque point est `w = √<w²>` après 300 balayages de transitoire puis 300 balayages moyennés, sur les graines 1 à 8.

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.

## Aperçus de l'étape 2d

- `etape-2d-bruit.png` : les amas de réduction du bruit m = 1, 4 et 16 côte à côte (graine 1, 3000 sites, réseau de demi-côté 250), chaque site coloré par son ordre d'arrivée et les panneaux séparés par un filet : les bras se rangent sur les axes quand m croît.
- `etape-2d-collage.png` : le collage partiel p = 1, 0,5 et 0,1 côte à côte (mêmes graine, taille et réseau), chaque site coloré par son ordre d'arrivée et les panneaux séparés par un filet : les branches s'épaississent quand p baisse.

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.


## Aperçus de l'étape D1

- `etape-d1-planche.png` : le claquage diélectrique à `η = 0`, `1` et `2` côte à côte (graine 1, 400 sites, domaine de demi-côté 75, même échelle pour les trois panneaux, chaque site coloré par son ordre d'arrivée) : un disque plein (le modèle d'Eden), un buisson, un éclair plus fin et plus étendu (rayons 16, 27 et 42). Produite par `cargo run --example apercus_etape_d1`.

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.

## Aperçus de l'étape D2

- `etape-d2-courbe.png` : la courbe `D(η)` du claquage diélectrique à 100 sites (`η = 0, 1, 2, 3`, graines 1 à 50), 640 × 320 pixels. Les carrés bleus sont la dimension moyenne mesurée par la corrélation sur la fenêtre des petits amas ; chacun porte sa barre d'erreur (l'erreur-type de la moyenne des 50 graines), mais à cette échelle (200 pixels par unité de `D`) la barre fait de 1 à 5 pixels de haut et se cache sous le carré de 5 × 5. Les carrés rouges, décalés vers la droite, sont les repères publiés : le rappel de la DLA à 1,71 pour `η = 1` et Hastings (2001) à 1,433 et 1,263 pour `η = 2` et `3`. Le décor sert à lire `D` : une grille grise tous les 0,25 de `D`, de 1,0 (ligne du bas) à 2,0 (ligne du haut), un axe vertical, un axe horizontal (à `D = 0,8`) et un trait de graduation sous chaque colonne de `η` ; la colonne de `η` est `40 + 70 η`. **Le banc n'est pas « en accord » avec le publié** : ses amas sont petits (100 sites contre 10 000 à 15 000 particules chez Hastings), sa méthode n'est pas la sienne et le choix de la fenêtre déplace `D` de ±0,05 ; le banc est au-dessus de Hastings pour `η = 2` et `3` et au-dessous du rappel de la DLA pour `η = 1` ; ce n'est pas un accord. Seuls l'ordre et la baisse de `D(η)` sont établis.
- `etape-d2-courbe.csv` : les mêmes quatre points, `eta,taille,graines,D,erreur,publie,ecart` (`ecart = D - publie`, `publie` et `ecart` vides pour `η = 0`, qui n'a pas de repère). Produits par `cargo run --example courbe_d2` (arguments facultatifs : le dossier, la taille, le nombre de graines).

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles.


## Aperçus de l'étape G1

Dessinés par `rendu_hex` (hexagones à leur vraie géométrie, un dégradé de bleus sur fond blanc d'après la masse cristalline `c`, suréchantillonnage 4 × 4 en entiers : plus aucun pixel n'est un agrandissement), calculés hors ligne par `cargo run --release --example apercus_etape_g1`
(environ cinq minutes pour la planche : le temps est celui de la simulation, le dessin coûte moins d'une seconde ; `cristal` ou `planche` en second argument n'en fait qu'une). Les réglages sont ceux de la figure 9 de Gravner et Griffeath (annexe 8, ligne « 9 » : `β = 1,3`, `α = 0,08`, `θ = 0,025`, `κ = 0,003`, `μ = 0,07`, `γ = 0,00005`).

- `etape-g1-cristal.png` : un cristal à `ρ = 0,46` (8 000 pas, rayon 100, 12 781 cases rattachées, domaine de rayon 150), 1000 × 1000 pixels : un bras vers le haut, six branches, des branches latérales ; la teinte suit `c` (le bleu le plus foncé est atteint dès `c = 1,5`, les cases les plus épaisses).
- `etape-g1-planche.png` : six cristaux, `ρ` de 0,40 à 0,50 par 0,02 (la figure 9 montre 0,40, 0,41, 0,42, 0,44, 0,46 et 0,50), chacun grandi jusqu'au rayon 100 (ou 40 000 pas : `ρ = 0,40` n'y arrive pas, rayon 88), 400 × 400 pixels par panneau, trois colonnes sur deux rangées, 1200 × 800 : de la plaque à secteurs aux dendrites à branches latérales.

Un aperçu est pour l'œil, jamais une preuve : les valeurs attendues et les empreintes sont dans `references/`, et
embellir une image ne change aucune d'entre elles. Ces PNG ne sont pas compressés (`3 * largeur * hauteur` octets : 3,0 Mo et 2,9 Mo).
