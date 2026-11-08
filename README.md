# Givre Lab

Un simulateur libre de croissance cristalline, en Rust, orienté recherche : agrégation limitée par diffusion,
croissance de surface, flocons de neige à facettes, attachement et détachement thermiques. Chaque mécanisme est refait
en nombres d'après son article (une dimension fractale, un exposant de croissance, une classe de formes), avec un
rendu qu'on prend plaisir à regarder : le givre pousse en dendrites et en facettes.

**Où il en est** : la V0 est faite. Les six étapes, de l'agrégat de Witten et Sander à la page web, sont fusionnées,
chacune avec sa preuve. [FEUILLE-DE-ROUTE.md](FEUILLE-DE-ROUTE.md) dit le plan ; [MODELE.md](MODELE.md) dit, pour chaque
modèle, ses équations, sa référence, ce que ses tests établissent et ce qu'ils n'établissent pas (il y en a, et ils sont
écrits : une valeur publiée qu'on n'atteint pas s'y dit telle quelle).

## Essayer

Lancer un scénario (l'empreinte de l'état, les mesures en CSV et les aperçus vont dans le dossier de sortie) :

```text
cargo run --release -- lancer scenarios/dla.txt --graine 1 --sortie sortie
```

La vitrine est la page `docs/index.html` : la DLA et les flocons de Reiter, calculés en WebAssembly (`docs/givre.wasm`,
construit depuis `src/vitrine.rs`) et dessinés sur un canvas, avec l'empreinte affichée (celle d'une graine y est la même
qu'en natif). Pour la voir en local, servir le dossier `docs/` par un serveur statique, par exemple
`python -m http.server --directory docs`, puis ouvrir `http://localhost:8000`. Elle se publie sur GitHub Pages quand
l'humain qui mène le dépôt l'active (dossier `/docs` de la branche `main`) ; ensuite chaque fusion la republie.

## Ce qui le tient

- Rust sans dépendance ; même scénario et même graine, même état et mêmes nombres, sous Linux comme sous Windows.
- La preuve est l'état simulé et des nombres, chacun avec sa référence publiée ; les images sont pour l'œil.
- Les tests d'une étape s'écrivent avant son code, et un juge les éprouve avant qu'ils soient gelés.
- Les règles du dépôt sont dans [AGENTS.md](AGENTS.md) ; chaque modèle, ses équations et ses limites, dans
  [MODELE.md](MODELE.md).

## Comment il se construit

Givre Lab est le second banc d'essai de [Lydite](https://github.com/aoricloud/lydite), un orchestrateur d'agents : ses
mains travaillent sur les issues, chacune sur une branche `lydite/…`, et GitHub fusionne une PR quand sa CI est verte
sous Linux et sous Windows. L'humain qui mène le dépôt en garde les réglages et regarde après coup. Le premier banc est
[Mycelium Lab](https://github.com/aoricloud/mycelium-lab).

## Licence

MIT, voir [LICENSE](LICENSE).
