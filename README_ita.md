<p align="center">
  <img src="images/EH3D_VALIDATION_LAB_BANNER.svg" alt="EH3D-VALIDATION-LAB banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-VALIDATION-LAB

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | 🇮🇹 <b>Italiano</b> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### EH3D-VALIDATION-LAB: l'ambiente di test della famiglia PHOENIX: manifesti di file di riferimento con copie ridondanti, errori classificati e un cancello che impedisce a un tasso misurato di scendere. Codice iniziale.

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust%20%2B%20Python-blue.svg" alt="Rust and Python">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-44-2EA043.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.1-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Verifica di onestà - cosa funziona oggi:** una piccola libreria Rust con riga di comando che scrive e verifica il manifesto delle somme di controllo di un insieme di riferimento rispetto a copie ridondanti, legge file di risultati in cui ogni errore deve avere una classe e confronta un'esecuzione con una base di partenza tramite un cancello. Ha 41 test propri e passano su un computer. È stata usata solo con dati creati dai suoi stessi test: mai su un corpus reale né su un vero insieme di riferimento. Non legge alcun file di progetto, non esegue alcuno strumento di riferimento e non confronta ancora l'output di alcuno strumento di progettazione. Il riduttore di casi, i test degli invarianti e il confronto con i file di riferimento non esistono.

---

> **Stato di sicurezza funzionale e informatica:** questo repository è ai primi passi e non è un sistema di sicurezza funzionale. Nulla di quanto scritto qui sostituisce i limiti, gli interblocchi e l'arresto di emergenza della macchina e del suo controllore.

## 📋 In breve

| | |
|---|---|
| Versione | `0.0.1` |
| Maturità | Impalcatura |
| Test | 44 |
| Stack tecnologico | Rust, Python 3.10+ |
| Su hardware reale | Non ancora eseguito su hardware |

**Vai a:** [Panoramica](#-panoramica) · [Architettura](#-architettura) · [Avvio rapido](#-avvio-rapido) · [Test](#-test) · [Struttura del repository](#-struttura-del-repository) · [Progetti correlati](#-progetti-correlati)

## 🚦 Stato del progetto

Ogni riga è una parte di questo repository e il semaforo dice a che punto è oggi. È scritta in base a ciò che il repository può mostrare (test, compilazioni, esecuzioni), non a ciò che è previsto.

🟢 Fatto e verificato · 🟡 Funziona, con limiti noti · 🟠 Fatto a metà o non ancora verificato · 🔴 Non fatto o non funzionante

| Area | Stato | Cosa significa qui |
|---|---|---|
| Controlli del repository | 🟡 | Il controllo del repository, quello delle immagini e i 3 test della base girano nella CI e passano. |
| Test | 🟡 | I 41 test del codice Rust e i 3 della base passano su un computer. Usano solo dati creati dai test stessi; nessuno è stato eseguito su un corpus reale. |
| Hardware reale | 🟠 | Non eseguito su nessuna macchina né scheda: nulla qui è stato provato su hardware. |
| Funzione pianificata | 🟠 | Scritto: manifesti degli insiemi di riferimento, verifica delle copie, risultati classificati e il cancello. Non scritto: il confronto con i file di riferimento, il riduttore di casi e i test degli invarianti. |
| Requisiti | 🟠 | 4 requisiti su 6 sono a livello di prototipo, 2 sono solo pianificati e nessuno è convalidato. |

## 🎯 Panoramica

* **Cosa esiste:** una libreria e una riga di comando, senza finestra. Scrivono e verificano il manifesto `SHA256SUMS` di un insieme di riferimento rispetto a copie ridondanti, leggono file di risultati in cui ogni errore ha una di tre classi e confrontano un'esecuzione con una base di partenza: il tasso misurato di ogni livello non deve scendere. I rapporti mostrano solo hash, livelli, classi e conteggi.
* **Pianificato:** il confronto con i file di riferimento, il riduttore di casi, i test con nome per gli invarianti del modello di progetto e, quando serve, un tipo di attività del server. Nulla di tutto questo esiste ancora.

## 🧩 Ruolo in EH3D

EH3D-VALIDATION-LAB fa parte della famiglia PHOENIX della EH3D Fabrication Suite. Cosa farà, cosa non farà ancora, i suoi requisiti e le sue relazioni con altri repository sono scritti nella cartella `docs/`.

## 🏛️ Architettura

<p align="center">
  <img src="images/architecture.svg" alt="Architettura di EH3D-VALIDATION-LAB: chi lo usa, che cosa contiene e da che cosa dipende" width="100%">
</p>

Come si incastrano le parti di questo repository: chi lo usa, che cosa contiene e da che cosa dipende. L'immagine è disegnata da `docs/architecture.json` e verificata rispetto al codice.

## 🚀 Avvio rapido

Servono Python 3.10 o successivo per i controlli e una toolchain Rust per il codice.

```bash
cargo build --release
cargo test
cargo run -- manifest <golden-folder>
python -I tools/ci_validate.py
python -I tools/make_architecture.py --check
python -I tools/make_structure.py --check --readmes
python -I -m unittest discover -s tools -p "test_*.py"
```

## 🧪 Test

Il codice ha 41 test (`cargo test`) e la base ne ha 3 (`python -I -m unittest discover -s tools -p "test_*.py"`); tutti passano su un computer. La CI esegue il controllo del repository, quello delle immagini, i test della base e, per il codice Rust, il controllo del formato, clippy con gli avvisi trattati come errori e i test su Linux e Windows.

## 🏗️ Compilazione e rilascio

La versione sta nel file `VERSION` e in `eh3d.project.json`, e ogni versione ha la sua voce in [CHANGELOG.md](CHANGELOG.md). `cargo build --release` compila la libreria e il programma `eh3d-validation-lab`; le versioni vengono pubblicate su GitHub.

## 📂 Struttura del repository

<p align="center">
  <img src="images/repository-structure.svg" alt="Struttura del repository EH3D-VALIDATION-LAB" width="100%">
</p>

<details>
<summary>Versione testuale dell'albero</summary>

```text
EH3D-VALIDATION-LAB/
├── docs/
│   ├── architecture.json structure.json         i dati delle due immagini
│   ├── SCOPE.md REQUIREMENTS.md                 l'ambito e i requisiti
│   ├── DEPENDENCIES.md THREAT_MODEL.md          relazioni con altri repository e modello delle minacce
│   ├── DECISIONS.md STATUS.md                   decisioni applicabili qui e stato onesto
│   ├── NOT_YET.md RULES.md                      idee per dopo e regole per scrivere il codice
│   └── adr/                                     i registri delle decisioni accettate
├── AGENTS.md                                    regole per chi modifica il repository
├── deny.toml                                    politica di licenze e avvisi delle dipendenze
├── src/                                         la libreria e la riga di comando
├── config/                                      il file delle impostazioni (ogni valore regolato)
├── Cargo.toml Cargo.lock                        il pacchetto Rust e le sue dipendenze bloccate
├── tools/                                       il controllo del repository, i suoi test e gli strumenti delle immagini
├── images/                                      banner e le immagini di Architettura e Struttura del repository
├── .github/                                     workflow di CI e della dashboard, modelli di issue e pull request
├── eh3d.project.json                            manifesto: versione, ruolo, maturità e comando di compilazione
├── VERSION                                      la versione del repository
├── CHANGELOG.md                                 cosa è cambiato in ogni versione
├── SECURITY.md SUPPORT.md                       come segnalare una vulnerabilità e dove chiedere
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   come contribuire, codice di condotta e licenza
```

</details>

## 🗺️ Roadmap e limiti noti

**Limiti noti oggi**

* Codice iniziale: è stato eseguito solo con dati creati dai suoi stessi test, mai su un corpus reale né su un vero insieme di riferimento.
* Verifica che le copie contengano gli stessi byte; metterle su supporti diversi spetta al proprietario.
* Il confronto con i file di riferimento, il riduttore di casi e i test degli invarianti non sono scritti.

**Prossimi passi**

* Scrivere il confronto con i file di riferimento e i test degli invarianti quando saranno accettati i contratti da cui dipende questo progetto (vedi `docs/DEPENDENCIES.md`).

## 🌍 Lingue

Il README e i testi esistono in inglese, spagnolo, francese, italiano, tedesco, cinese, giapponese e portoghese europeo.

## 🔗 Progetti correlati

* [EH3D-DOCS](https://github.com/JuanenRac/EH3D-DOCS): la mappa della suite, la sua architettura, le convenzioni e i controlli condivisi
* Relazioni pianificate, non ancora costruite ([EH3D-PHOENIX-PCB](https://github.com/JuanenRac/EH3D-PHOENIX-PCB), [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON), [EH3D-LAB-BRIDGE](https://github.com/JuanenRac/EH3D-LAB-BRIDGE)): vedi [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)

## 📚 Documentazione

* [Le convenzioni della suite](https://github.com/JuanenRac/EH3D-DOCS/blob/main/docs/CONVENTIONS.md)
* [Registro delle modifiche](CHANGELOG.md)
* [Ambito, requisiti, dipendenze, modello delle minacce e decisioni](docs/SCOPE.md)

## 🤝 Contribuire e comunità

* Prima di una pull request esegui `python -I tools/ci_validate.py` e i test della base: tutto deve passare. Vedi [CONTRIBUTING.md](CONTRIBUTING.md) e il [codice di condotta](CODE_OF_CONDUCT.md).
* Il README esiste in otto lingue: una modifica al testo si fa in tutti e otto i file, e le due immagini sono disegnate da `docs/architecture.json` e `docs/structure.json` con `tools/make_architecture.py` e `tools/make_structure.py`.
* Segnala una vulnerabilità in privato: [SECURITY.md](SECURITY.md). Domande, idee e segnalazioni: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTORE

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENZA

Software: GPL-3.0-or-later. Vedi [LICENSE](LICENSE). I progetti hardware della suite usano la CERN OHL-S.
