# Givre Lab

Un simulateur libre de croissance cristalline, en Rust, orienté recherche : agrégation limitée par diffusion,
croissance de surface, flocons de neige à facettes, attachement et détachement thermiques. Chaque mécanisme est refait
en nombres d'après son article (une dimension fractale, un exposant de croissance, une classe de formes), avec un
rendu qu'on prend plaisir à regarder : le givre pousse en dendrites et en facettes.

**Où il en est** : rien ne tourne encore. La V0, six étapes de l'agrégat de Witten et Sander à la page web, est décrite
dans [FEUILLE-DE-ROUTE.md](FEUILLE-DE-ROUTE.md).

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
