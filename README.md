# Perhabs
A package of cognitive (incl visual, auditory) exercises to support recovery from brain injury. Takes inspiration from Cognitive FX and other mTBI-therapies.

Perhabs is especially useful for people wanting to exercise their brains by themselves. Some exercises rely on having someone read you the next cue. When you are by yourself, Perhabs can do this for you!

## Note
This is not a substitute for proper diagnosis and prescription by a health care professional. The exercises in this package are provided as-is. Consult your therapist/physician to determine the selection and intensity of exercises that will benefit you.

## Included exercises
### Cognitive
- Working memory:
    - Calculate a series of numbers based on random input
        (Example: start frpm 0, add 8, remove 3, complete the sequence)
    - Recall and reorder numbers or sentences
- Visuospatial skills:
    - Drawing an image on its side or mirrored

### Visual
- Convergence/divergence
- Saccades
- Visual recall

## Various supporting modules
- Timer (not available in web version)
    Runs for a random amount of time (configurable) and then says "switch!". Runs out after a given amount of time.
- Clock
- Metronome (web support is flaky)

## Technical
Perhabs is written in Rust and relies on [egui](https://crates.io/crates/egui) ([demo](https://egui.rs)) for its UI. I'm standing on the shoulders of giants all the way down.
