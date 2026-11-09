# Givre Lab

A free simulator of crystal growth, in Rust, built for research: diffusion-limited aggregation, surface growth,
faceted snow crystals, dielectric breakdown. Each mechanism is checked against a published number where one exists (a
fractal dimension, a growth exponent, a class of shapes), and the gap is written down.

**▶ Run it in your browser: https://aoricloud.github.io/givre-lab/** (the aggregate, Reiter's snow crystals and a
measure of the correlation dimension, computed in WebAssembly).

> [!NOTE]
> **This repository is written by AI agents, under [Lydite](https://github.com/aoricloud/lydite).** A human writes the
> direction and keeps the settings; the agents write the code from plans, and Lydite replays the proof before any pull
> request. Each of its pull requests carries a receipt (cost, agents, retries); the recent ones carry the line
> `Reviewed and replayed on commit …` (or `Replayed on commit …, not reviewed` when no task was critical), and GitHub
> only merges a pull request whose head is that commit, once the CI is green on Linux and Windows. See
> [#29](https://github.com/aoricloud/givre-lab/pull/29): ten agents, one retry, $0.80, and what the review could not
> see replayed, said in the open.
> As of 9 November 2026: 27 pull requests by Lydite merged in two days, 153 agents, 6 retries, and $8.17 in all for
> the agents (estimated; they run on a low-cost model, DeepSeek; the session that writes the plans is not counted).

## Where it stands

The first version is merged: six steps, from Witten and Sander's aggregate to the web page, each with its proof. Since
then: the circular boundary of Reiter's crystals; the correlation dimension, shown beside the value published for zinc
electrodeposits (compatible, which does not prove that zinc follows the model); and the dielectric breakdown model with
its curve D(η), where the bench establishes the order and the decrease, not the published values. Next, decided on 8
November: the page and a slider for η (the breakdown model), then Gravner and Griffeath's snow in 2D, then phase-field
metal in 2D ([FEUILLE-DE-ROUTE.md](FEUILLE-DE-ROUTE.md)).

[MODELE.md](MODELE.md) says, for each model, its equations, its reference, what its tests establish and what they do
not. A published value that is not reached is written as such: at 4000 sites the mass-radius dimension of the aggregate
is 1.66, 0.04 under the published 1.71.

## What holds it

- Rust with no dependency; the same scenario and the same seed give the same state and the same numbers, on Linux and
  on Windows.
- The proof is the simulated state and numbers, each with its published reference; images are for the eye.
- The tests of a step are written before its code, in a pull request of their own (for example
  [#28](https://github.com/aoricloud/givre-lab/pull/28), then [#29](https://github.com/aoricloud/givre-lab/pull/29)).
- The rules of the repository are in [AGENTS.md](AGENTS.md).

## Try it

```text
cargo run --release -- lancer scenarios/dla.txt --graine 1 --sortie sortie
```

The fingerprint of the state, the measures (CSV) and the previews go to the output folder. The page shows the same
fingerprint for the same seed.

The model notes, the rules, the roadmap and the page are in French. The first bench is
[Mycelium Lab](https://github.com/aoricloud/mycelium-lab).

## License

MIT, see [LICENSE](LICENSE).
