<p align="center">
  <img src="images/EH3D_VALIDATION_LAB_BANNER.svg" alt="EH3D-VALIDATION-LAB banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-VALIDATION-LAB

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | 🇫🇷 <b>Français</b> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### EH3D-VALIDATION-LAB : l'environnement de test de la famille PHOENIX : manifestes de fichiers de référence avec copies redondantes, échecs classés et une barrière qui empêche un taux mesuré de baisser. Code de départ.

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust%20%2B%20Python-blue.svg" alt="Rust and Python">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-63-2EA043.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.2-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Vérification d'honnêteté - ce qui tourne aujourd'hui :** une petite bibliothèque Rust avec une ligne de commande qui écrit et vérifie le manifeste de sommes de contrôle d'un jeu de référence face à des copies redondantes, lit des fichiers de résultats où chaque échec doit porter une classe, et compare une exécution à une base de référence au moyen d'une barrière. Elle a 60 tests propres, qui réussissent sur un ordinateur. Elle n'a servi que sur des données fabriquées par ses propres tests : jamais sur un vrai corpus ni sur un vrai jeu de référence. Elle ne lit aucun fichier de conception, n'exécute aucun outil de référence et ne compare pas encore la sortie d'un outil de conception. Le réducteur de cas, les tests d'invariants et la comparaison avec les fichiers de référence n'existent pas.

---

> **État de la sûreté et de la sécurité :** ce dépôt en est à ses premiers pas et n'est pas un système de sûreté. Rien de ce qui est écrit ici ne remplace les limites, les verrouillages et l'arrêt d'urgence de la machine et de son contrôleur.

## 📋 En un coup d'œil

| | |
|---|---|
| Version | `0.0.2` |
| Maturité | Échafaudage |
| Tests | 63 |
| Pile technique | Rust, Python 3.10+ |
| Sur du matériel réel | Pas encore exécuté sur du matériel |

**Aller à  :** [Présentation](#-présentation) · [Architecture](#-architecture) · [Démarrage rapide](#-démarrage-rapide) · [Tests](#-tests) · [Structure du dépôt](#-structure-du-dépôt) · [Projets liés](#-projets-liés)

## 🚦 État du projet

Chaque ligne est une partie de ce dépôt et le feu dit où elle en est aujourd'hui. Elle est écrite d'après ce que le dépôt peut montrer (tests, compilations, exécutions), pas d'après ce qui est prévu.

🟢 Fait et vérifié · 🟡 Fonctionne, avec des limites connues · 🟠 À moitié fait ou pas encore vérifié · 🔴 Pas fait ou ne fonctionne pas

| Domaine | État | Ce que cela veut dire ici |
|---|---|---|
| Contrôles du dépôt | 🟡 | Le contrôle du dépôt, celui des images et les 3 tests de la base tournent dans la CI et réussissent. |
| Tests | 🟡 | Les 60 tests du code Rust et les 3 de la base réussissent sur un ordinateur. Ils n'utilisent que des données fabriquées par les tests eux-mêmes ; aucun n'a tourné sur un vrai corpus. |
| Matériel réel | 🟠 | Non exécuté sur une machine ni une carte : rien ici n'a été essayé sur du matériel. |
| Fonction prévue | 🟠 | Écrit : manifestes de jeux de référence, vérification des copies, résultats classés et la barrière. Pas écrit : la comparaison avec les fichiers de référence, le réducteur de cas et les tests d'invariants. |
| Exigences | 🟠 | 4 exigences sur 6 sont au stade de prototype, 2 sont seulement prévues et aucune n'est validée. |

## 🎯 Présentation

* **Ce qui existe :** une bibliothèque et une ligne de commande, sans fenêtre. Elles écrivent et vérifient le manifeste `SHA256SUMS` d'un jeu de référence face à des copies redondantes, lisent des fichiers de résultats où chaque échec porte l'une de trois classes, et comparent une exécution à une base de référence : le taux mesuré de chaque niveau ne doit pas baisser. Les rapports n'affichent que des empreintes, des niveaux, des classes et des comptes.
* **Prévu :** la comparaison avec les fichiers de référence, le réducteur de cas, les tests nommés des invariants du modèle de conception et, le cas échéant, un type de tâche du serveur. Rien de cela n'existe encore.

## 🧩 Rôle dans EH3D

EH3D-VALIDATION-LAB fait partie de la famille PHOENIX de l'EH3D Fabrication Suite. Ce qu'il fera, ce qu'il ne fera pas encore, ses exigences et ses relations avec d'autres dépôts sont décrits dans le dossier `docs/`.

## 🏛️ Architecture

<p align="center">
  <img src="images/architecture.svg" alt="Architecture de EH3D-VALIDATION-LAB : qui l'utilise, ce qu'il contient et de quoi il dépend" width="100%">
</p>

Comment les parties de ce dépôt s'articulent : qui l'utilise, ce qu'il contient et de quoi il dépend. L'image est dessinée à partir de `docs/architecture.json` et vérifiée par rapport au code.

## 🚀 Démarrage rapide

Il faut Python 3.10 ou plus récent pour les contrôles et une chaîne d'outils Rust pour le code.

```bash
cargo build --release
cargo test
cargo run -- manifest <golden-folder>
python -I tools/ci_validate.py
python -I tools/make_architecture.py --check
python -I tools/make_structure.py --check --readmes
python -I -m unittest discover -s tools -p "test_*.py"
```

## 🧪 Tests

Le code a 60 tests (`cargo test`) et la base en a 3 (`python -I -m unittest discover -s tools -p "test_*.py"`) ; tous réussissent sur un ordinateur. La CI exécute le contrôle du dépôt, celui des images, les tests de la base et, pour le code Rust, la vérification du format, clippy avec les avertissements traités comme des erreurs et les tests sous Linux et Windows.

## 🏗️ Compilation et publication

La version se trouve dans le fichier `VERSION` et dans `eh3d.project.json`, et chaque version a son entrée dans [CHANGELOG.md](CHANGELOG.md). `cargo build --release` compile la bibliothèque et le programme `eh3d-validation-lab` ; les versions sont publiées sur GitHub.

## 📂 Structure du dépôt

<p align="center">
  <img src="images/repository-structure.svg" alt="Structure du dépôt EH3D-VALIDATION-LAB" width="100%">
</p>

<details>
<summary>Version texte de l'arborescence</summary>

```text
EH3D-VALIDATION-LAB/
├── docs/
│   ├── architecture.json structure.json         les données des deux images
│   ├── SCOPE.md REQUIREMENTS.md                 son périmètre et ses exigences
│   ├── DEPENDENCIES.md THREAT_MODEL.md          relations avec d'autres dépôts et modèle de menaces
│   ├── DECISIONS.md STATUS.md                   décisions applicables ici et état honnête
│   ├── NOT_YET.md RULES.md                      idées pour plus tard et règles d'écriture du code
│   └── adr/                                     les décisions acceptées
├── AGENTS.md                                    règles pour quiconque modifie le dépôt
├── deny.toml                                    politique de licences et d'avis des dépendances
├── src/                                         la bibliothèque et la ligne de commande
├── config/                                      le fichier de réglages (chaque valeur ajustée)
├── Cargo.toml Cargo.lock                        le paquet Rust et ses dépendances figées
├── rust-toolchain.toml                          la toolchain Rust épinglée
├── tools/                                       le contrôle du dépôt, ses tests et les outils des images
├── images/                                      bannière et images d'Architecture et de Structure du dépôt
├── .github/                                     workflows de CI et du tableau de bord, modèles d'issues et de pull requests
├── eh3d.project.json                            manifeste : version, rôle, maturité et commande de compilation
├── VERSION                                      la version du dépôt
├── CHANGELOG.md                                 ce qui a changé dans chaque version
├── SECURITY.md SUPPORT.md                       signaler une vulnérabilité et poser des questions
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   contribuer, code de conduite et licence
```

</details>

## 🗺️ Feuille de route et limites connues

**Limites connues aujourd'hui**

* Code de départ : il n'a tourné que sur des données fabriquées par ses propres tests, jamais sur un vrai corpus ni sur un vrai jeu de référence.
* Il vérifie que les copies contiennent les mêmes octets ; les placer sur des supports différents revient au propriétaire.
* La comparaison avec les fichiers de référence, le réducteur de cas et les tests d'invariants ne sont pas écrits.

**Prochaine étape**

* Écrire la comparaison avec les fichiers de référence et les tests d'invariants une fois acceptés les contrats dont ce projet dépend (voir `docs/DEPENDENCIES.md`).

## 🌍 Langues

Le README et les textes existent en anglais, espagnol, français, italien, allemand, chinois, japonais et portugais européen.

## 🔗 Projets liés

* [EH3D-DOCS](https://github.com/JuanenRac/EH3D-DOCS) : la carte de la suite, son architecture, les conventions et les contrôles partagés
* Relations prévues, pas encore construites ([EH3D-PHOENIX-PCB](https://github.com/JuanenRac/EH3D-PHOENIX-PCB), [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON), [EH3D-LAB-BRIDGE](https://github.com/JuanenRac/EH3D-LAB-BRIDGE)) : voir [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)

## 📚 Documentation

* [Les conventions de la suite](https://github.com/JuanenRac/EH3D-DOCS/blob/main/docs/CONVENTIONS.md)
* [Journal des modifications](CHANGELOG.md)
* [Périmètre, exigences, dépendances, modèle de menaces et décisions](docs/SCOPE.md)

## 🤝 Contribuer et communauté

* Avant une pull request, exécutez `python -I tools/ci_validate.py` et les tests de la base : tout doit réussir. Voir [CONTRIBUTING.md](CONTRIBUTING.md) et le [code de conduite](CODE_OF_CONDUCT.md).
* Le README existe en huit langues : une modification du texte se fait dans les huit fichiers, et les deux images sont dessinées à partir de `docs/architecture.json` et `docs/structure.json` par `tools/make_architecture.py` et `tools/make_structure.py`.
* Signalez une vulnérabilité en privé : [SECURITY.md](SECURITY.md). Questions, idées et rapports : electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTEUR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENCE

Logiciel : GPL-3.0-or-later. Voir [LICENSE](LICENSE). Les conceptions matérielles de la suite utilisent la CERN OHL-S.
