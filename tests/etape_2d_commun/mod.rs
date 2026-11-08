//! Aides communes aux tests de l'étape 2d : un délai pour les simulations. Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est
//! gelé avec les tests (comme `tests/common/mod.rs` à l'étape 0).
#![allow(dead_code)]

use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

/// Joue `corps` dans un fil séparé et ÉCHOUE au bout de `secondes` : une simulation fautive qui boucle sans fin (un compteur remis à zéro à chaque
/// abandon, un marcheur qui n'est jamais abandonné, un collage impossible) fait alors échouer le test avec un message, au lieu de bloquer le test
/// (et la CI) jusqu'au délai de la CI. Si `corps` panique, la panique est relancée telle quelle (son message est celui du test). Le fil d'une simulation
/// qui boucle reste en vie jusqu'à la fin du processus de test.
pub fn avec_delai<F: FnOnce() + Send + 'static>(secondes: u64, corps: F) {
    let (envoi, reception) = channel();
    let fil = std::thread::spawn(move || {
        corps();
        let _ = envoi.send(());
    });
    match reception.recv_timeout(Duration::from_secs(secondes)) {
        Ok(()) => {}
        Err(RecvTimeoutError::Timeout) => {
            panic!(
                "délai de {secondes} s dépassé : la simulation ne finit pas (boucle sans fin : compteur remis à zéro, marcheur jamais abandonné ?)"
            )
        }
        Err(RecvTimeoutError::Disconnected) => {
            if let Err(panique) = fil.join() {
                std::panic::resume_unwind(panique);
            }
        }
    }
}
