//! Givre Lab : les simulations de croissance cristalline, leurs mesures et leur rendu. Le binaire `givre` les lance ;
//! `FEUILLE-DE-ROUTE.md` dit quelle étape remplit quel module. Les modules sont déclarés d'avance, ici, pour que deux
//! vagues n'éditent pas ce fichier ensemble.

pub mod alea;
pub mod cinetique;
pub mod dla;
pub mod dla_variantes;
pub mod empreinte;
pub mod flocon;
pub mod image;
pub mod lancer;
pub mod math;
pub mod mesures;
pub mod rendu;
pub mod reseau;
pub mod scenario;
pub mod surface;
pub mod vitrine;
