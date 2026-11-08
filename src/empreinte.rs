//! L'empreinte de l'état : SHA-256 (FIPS 180-4), écrit dans le dépôt, sans dépendance (étape 0). L'empreinte d'un état
//! simulé est le SHA-256 de son écriture canonique (voir `lancer` pour celle de l'étape 0).
//!
//! # L'algorithme (FIPS 180-4, section 6.2)
//!
//! Le message (une suite d'octets de `L` bits) est complété (« bourrage », section 5.1.1) : un bit 1 (l'octet `0x80`),
//! puis autant de bits 0 qu'il faut pour que la longueur totale soit congrue à 448 modulo 512 bits, puis `L` écrit sur
//! 64 bits en grand-boutiste. Le résultat est découpé en blocs de 512 bits (64 octets). Les mots de 32 bits sont lus en
//! grand-boutiste. Le hachage part de `H = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
//! 0x1f83d9ab, 0x5be0cd19]` (les 32 premiers bits des parties fractionnaires des racines carrées des huit premiers
//! nombres premiers) et traite chaque bloc avec les 64 constantes `K[0..64]` (les 32 premiers bits des parties
//! fractionnaires des racines cubiques des 64 premiers nombres premiers, de `0x428a2f98` à `0xc67178f2`) :
//!
//! - échéancier : `W[t]` = le mot `t` du bloc pour `t < 16`, et pour `16 <= t < 64`
//!   `W[t] = σ1(W[t-2]) + W[t-7] + σ0(W[t-15]) + W[t-16]` (modulo 2^32), avec
//!   `σ0(x) = rotd(x,7) ^ rotd(x,18) ^ (x >> 3)` et `σ1(x) = rotd(x,17) ^ rotd(x,19) ^ (x >> 10)` ;
//! - 64 tours à partir de `(a, b, c, d, e, f, g, h) = H` : `T1 = h + Σ1(e) + Ch(e,f,g) + K[t] + W[t]`,
//!   `T2 = Σ0(a) + Maj(a,b,c)`, puis `h = g`, `g = f`, `f = e`, `e = d + T1`, `d = c`, `c = b`, `b = a`, `a = T1 + T2`
//!   (tout modulo 2^32), avec `Σ0(x) = rotd(x,2) ^ rotd(x,13) ^ rotd(x,22)`, `Σ1(x) = rotd(x,6) ^ rotd(x,11) ^
//!   rotd(x,25)`, `Ch(x,y,z) = (x & y) ^ (!x & z)`, `Maj(x,y,z) = (x & y) ^ (x & z) ^ (y & z)` (`rotd` : rotation
//!   vers la droite) ;
//! - fin du bloc : chaque `H[i]` reçoit son registre (`H[0] += a`, …, `H[7] += h`, modulo 2^32).
//!
//! L'empreinte est la concaténation des huit mots de `H` en grand-boutiste : 32 octets.
//!
//! # Les vecteurs de test
//!
//! Lus dans les exemples du NIST (« SHA-256 example », SHA256.pdf et SHA2_Additional.pdf) et dans FIPS 180-2,
//! annexe B : voir le fichier de tests de l'étape 0 pour la liste.

/// Les 64 constantes `K` (FIPS 180-4, section 4.2.2) : les 32 premiers bits des parties fractionnaires des racines
/// cubiques des 64 premiers nombres premiers.
const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// Le SHA-256 de `donnees` : 32 octets (voir la documentation du module).
pub fn sha256(donnees: &[u8]) -> [u8; 32] {
    /// `rotd(x, 7) ^ rotd(x, 18) ^ (x >> 3)` (FIPS 180-4, section 4.1.2).
    fn sigma0(x: u32) -> u32 {
        x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
    }

    /// `rotd(x, 17) ^ rotd(x, 19) ^ (x >> 10)` (FIPS 180-4, section 4.1.2).
    fn sigma1(x: u32) -> u32 {
        x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
    }

    /// `rotd(x, 2) ^ rotd(x, 13) ^ rotd(x, 22)` (FIPS 180-4, section 4.1.2).
    fn grand_sigma0(x: u32) -> u32 {
        x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
    }

    /// `rotd(x, 6) ^ rotd(x, 11) ^ rotd(x, 25)` (FIPS 180-4, section 4.1.2).
    fn grand_sigma1(x: u32) -> u32 {
        x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
    }

    /// `(x & y) ^ (!x & z)` (FIPS 180-4, section 4.1.2).
    fn choix(x: u32, y: u32, z: u32) -> u32 {
        (x & y) ^ (!x & z)
    }

    /// `(x & y) ^ (x & z) ^ (y & z)` (FIPS 180-4, section 4.1.2).
    fn majorite(x: u32, y: u32, z: u32) -> u32 {
        (x & y) ^ (x & z) ^ (y & z)
    }

    let mut h: [u32; 8] =
        [0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_f53a, 0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19];

    // Bourrage (section 5.1.1) : un bit 1, des zéros jusqu'à 56 modulo 64, puis la longueur en bits sur 64 bits
    // grand-boutistes.
    let longueur_bits = (donnees.len() as u64).wrapping_mul(8);
    let mut message = Vec::with_capacity(donnees.len() + 72);
    message.extend_from_slice(donnees);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&longueur_bits.to_be_bytes());

    for bloc in message.as_chunks::<64>().0 {
        // Échéancier des 64 mots (section 6.2.2, étape 1).
        let mut w = [0_u32; 64];
        for (t, mot) in bloc.as_chunks::<4>().0.iter().enumerate() {
            w[t] = u32::from_be_bytes(*mot);
        }
        for t in 16..64 {
            w[t] = sigma1(w[t - 2]).wrapping_add(w[t - 7]).wrapping_add(sigma0(w[t - 15])).wrapping_add(w[t - 16]);
        }

        // Les 64 tours (section 6.2.2, étape 3).
        let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
        let (mut e, mut f, mut g, mut hh) = (h[4], h[5], h[6], h[7]);
        for (&k, &mot) in K.iter().zip(w.iter()) {
            let t1 = hh.wrapping_add(grand_sigma1(e)).wrapping_add(choix(e, f, g)).wrapping_add(k).wrapping_add(mot);
            let t2 = grand_sigma0(a).wrapping_add(majorite(a, b, c));
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut sortie = [0_u8; 32];
    for (i, mot) in h.iter().enumerate() {
        sortie[4 * i..4 * i + 4].copy_from_slice(&mot.to_be_bytes());
    }
    sortie
}

/// Le SHA-256 de `donnees` écrit en 64 chiffres hexadécimaux **minuscules** (deux par octet, du premier octet au
/// dernier, sans séparateur ni préfixe).
pub fn sha256_hex(donnees: &[u8]) -> String {
    const CHIFFRES: &[u8; 16] = b"0123456789abcdef";
    let empreinte = sha256(donnees);
    let mut texte = String::with_capacity(64);
    for octet in empreinte {
        texte.push(CHIFFRES[(octet >> 4) as usize] as char);
        texte.push(CHIFFRES[(octet & 0x0F) as usize] as char);
    }
    texte
}
