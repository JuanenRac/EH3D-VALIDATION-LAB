<p align="center">
  <img src="images/EH3D_VALIDATION_LAB_BANNER.svg" alt="EH3D-VALIDATION-LAB banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-VALIDATION-LAB

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | 🇩🇪 <b>Deutsch</b> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### EH3D-VALIDATION-LAB: die Testumgebung der PHOENIX-Familie: Manifeste von Referenzdateien mit redundanten Kopien, klassifizierte Fehler und ein Tor, das verhindert, dass eine gemessene Rate sinkt. Früher Code.

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust%20%2B%20Python-blue.svg" alt="Rust and Python">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-63-2EA043.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.2-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Ehrlichkeitscheck - was heute tatsächlich läuft:** eine kleine Rust-Bibliothek mit Kommandozeile, die das Prüfsummen-Manifest eines Referenzsatzes schreibt und gegen redundante Kopien prüft, Ergebnisdateien liest, in denen jeder Fehler eine Klasse tragen muss, und einen Lauf über ein Tor mit einer Ausgangsbasis vergleicht. Sie hat 60 eigene Tests, die auf einem Rechner bestehen. Sie wurde nur mit Daten benutzt, die ihre eigenen Tests erzeugen: nie mit einem echten Korpus oder einem echten Referenzsatz. Sie liest keine Entwurfsdatei, startet kein Referenzwerkzeug und vergleicht noch keine Ausgabe eines Entwurfswerkzeugs. Der Fallreduzierer, die Invariantentests und der Vergleich mit Referenzdateien existieren nicht.

---

> **Stand von Sicherheit und IT-Sicherheit:** dieses Repository steht am Anfang und ist kein Sicherheitssystem (Safety). Nichts hier Geschriebenes ersetzt die Grenzen, Verriegelungen und den Not-Aus der Maschine und ihrer Steuerung.

## 📋 Auf einen Blick

| | |
|---|---|
| Version | `0.0.2` |
| Reifegrad | Gerüst |
| Tests | 63 |
| Technologie-Stack | Rust, Python 3.10+ |
| Auf echter Hardware | Noch nicht auf Hardware ausgeführt |

**Springe zu:** [Überblick](#-überblick) · [Architektur](#-architektur) · [Schnellstart](#-schnellstart) · [Tests](#-tests) · [Struktur des Repositorys](#-struktur-des-repositorys) · [Verwandte Projekte](#-verwandte-projekte)

## 🚦 Projektstatus

Jede Zeile ist ein Teil dieses Repositorys, und die Ampel sagt, wie weit er heute ist. Sie beruht auf dem, was das Repository zeigen kann (Tests, Builds, Läufe), nicht auf dem, was geplant ist.

🟢 Fertig und überprüft · 🟡 Funktioniert, mit bekannten Grenzen · 🟠 Teilweise fertig oder noch nicht überprüft · 🔴 Nicht gemacht oder funktioniert nicht

| Bereich | Stand | Was das hier bedeutet |
|---|---|---|
| Repository-Prüfungen | 🟡 | Die Repository-Prüfung, die Prüfung der Bilder und die 3 Tests der Basis laufen in der CI und bestehen. |
| Tests | 🟡 | Die 60 Tests des Rust-Codes und die 3 der Basis bestehen auf einem Rechner. Sie verwenden nur Daten, die die Tests selbst erzeugen; keiner lief auf einem echten Korpus. |
| Echte Hardware | 🟠 | Auf keiner Maschine und keiner Platine ausgeführt: nichts hier wurde auf Hardware ausprobiert. |
| Geplante Funktion | 🟠 | Geschrieben: Manifeste von Referenzsätzen, Prüfung der Kopien, klassifizierte Ergebnisse und das Tor. Nicht geschrieben: der Vergleich mit Referenzdateien, der Fallreduzierer und die Invariantentests. |
| Anforderungen | 🟠 | 4 von 6 Anforderungen sind im Prototypstand, 2 sind nur geplant und keine ist validiert. |

## 🎯 Überblick

* **Was es gibt:** eine Bibliothek und eine Kommandozeile, ohne Fenster. Sie schreiben und prüfen das `SHA256SUMS`-Manifest eines Referenzsatzes gegen redundante Kopien, lesen Ergebnisdateien, in denen jeder Fehler eine von drei Klassen trägt, und vergleichen einen Lauf mit einer Ausgangsbasis: Die gemessene Rate jeder Stufe darf nicht sinken. Berichte zeigen nur Hashes, Stufen, Klassen und Zahlen.
* **Geplant:** der Vergleich mit Referenzdateien, der Fallreduzierer, benannte Tests für die Invarianten des Entwurfsmodells und, wo es passt, eine Art Server-Aufgabe. Nichts davon existiert bisher.

## 🧩 Rolle in EH3D

EH3D-VALIDATION-LAB gehört zur PHOENIX-Familie der EH3D Fabrication Suite. Was es tun wird, was es noch nicht tun wird, seine Anforderungen und seine Beziehungen zu anderen Repositories stehen im Ordner `docs/`.

## 🏛️ Architektur

<p align="center">
  <img src="images/architecture.svg" alt="Architektur von EH3D-VALIDATION-LAB: wer es nutzt, was es enthält und wovon es abhängt" width="100%">
</p>

Wie die Teile dieses Repositorys zusammenpassen: wer es nutzt, was es enthält und wovon es abhängt. Das Bild wird aus `docs/architecture.json` gezeichnet und gegen den Code geprüft.

## 🚀 Schnellstart

Für die Prüfungen brauchst du Python 3.10 oder neuer, für den Code eine Rust-Toolchain.

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

Der Code hat 60 Tests (`cargo test`), die Basis hat 3 (`python -I -m unittest discover -s tools -p "test_*.py"`); alle bestehen auf einem Rechner. Die CI führt die Repository-Prüfung, die Prüfung der Bilder, die Tests der Basis und für den Rust-Code die Formatprüfung, clippy mit Warnungen als Fehlern und die Tests unter Linux und Windows aus.

## 🏗️ Build und Veröffentlichung

Die Version steht in der Datei `VERSION` und in `eh3d.project.json`, und jede Version hat ihren Eintrag in [CHANGELOG.md](CHANGELOG.md). `cargo build --release` baut die Bibliothek und das Programm `eh3d-validation-lab`; Releases werden auf GitHub veröffentlicht.

## 📂 Struktur des Repositorys

<p align="center">
  <img src="images/repository-structure.svg" alt="Struktur des Repositorys EH3D-VALIDATION-LAB" width="100%">
</p>

<details>
<summary>Textversion des Baums</summary>

```text
EH3D-VALIDATION-LAB/
├── docs/
│   ├── architecture.json structure.json         die Daten der beiden Bilder
│   ├── SCOPE.md REQUIREMENTS.md                 Umfang und Anforderungen
│   ├── DEPENDENCIES.md THREAT_MODEL.md          Beziehungen zu anderen Repositories und Bedrohungsmodell
│   ├── DECISIONS.md STATUS.md                   hier geltende Entscheidungen und ehrlicher Status
│   ├── NOT_YET.md RULES.md                      Ideen für später und Regeln zum Schreiben von Code
│   └── adr/                                     die angenommenen Entscheidungsprotokolle
├── AGENTS.md                                    Regeln für alle, die das Repository ändern
├── deny.toml                                    Lizenz- und Hinweisrichtlinie der Abhängigkeiten
├── src/                                         die Bibliothek und die Kommandozeile
├── config/                                      die Einstellungsdatei (jeder abgestimmte Wert)
├── Cargo.toml Cargo.lock                        das Rust-Paket und seine festgelegten Abhängigkeiten
├── rust-toolchain.toml                          die festgelegte Rust-Toolchain
├── tools/                                       die Repository-Prüfung, ihre Tests und die Bild-Werkzeuge
├── images/                                      Banner und die Bilder von Architektur und Repository-Struktur
├── .github/                                     CI- und Dashboard-Workflows, Vorlagen für Issues und Pull Requests
├── eh3d.project.json                            Manifest: Version, Rolle, Reifegrad und Build-Befehl
├── VERSION                                      die Version des Repositorys
├── CHANGELOG.md                                 was sich in jeder Version geändert hat
├── SECURITY.md SUPPORT.md                       Schwachstellen melden und Fragen stellen
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   Mitwirken, Verhaltenskodex und Lizenz
```

</details>

## 🗺️ Roadmap und bekannte Grenzen

**Bekannte Grenzen heute**

* Früher Code: Er lief nur mit Daten seiner eigenen Tests, nie mit einem echten Korpus oder einem echten Referenzsatz.
* Er prüft, dass Kopien dieselben Bytes enthalten; sie auf verschiedene Datenträger zu legen, ist Sache des Eigentümers.
* Der Vergleich mit Referenzdateien, der Fallreduzierer und die Invariantentests sind nicht geschrieben.

**Als Nächstes**

* Den Vergleich mit Referenzdateien und die Invariantentests schreiben, sobald die Verträge angenommen sind, von denen dieses Projekt abhängt (siehe `docs/DEPENDENCIES.md`).

## 🌍 Sprachen

README und Texte gibt es auf Englisch, Spanisch, Französisch, Italienisch, Deutsch, Chinesisch, Japanisch und europäischem Portugiesisch.

## 🔗 Verwandte Projekte

* [EH3D-DOCS](https://github.com/JuanenRac/EH3D-DOCS): die Karte der Suite, ihre Architektur, die Konventionen und die gemeinsamen Prüfungen
* Geplante Beziehungen, noch nicht gebaut ([EH3D-PHOENIX-PCB](https://github.com/JuanenRac/EH3D-PHOENIX-PCB), [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON), [EH3D-LAB-BRIDGE](https://github.com/JuanenRac/EH3D-LAB-BRIDGE)): siehe [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)

## 📚 Dokumentation

* [Die Konventionen der Suite](https://github.com/JuanenRac/EH3D-DOCS/blob/main/docs/CONVENTIONS.md)
* [Änderungsprotokoll](CHANGELOG.md)
* [Umfang, Anforderungen, Abhängigkeiten, Bedrohungsmodell und Entscheidungen](docs/SCOPE.md)

## 🤝 Mitwirken und Community

* Führe vor einem Pull Request `python -I tools/ci_validate.py` und die Tests der Basis aus: alles muss bestehen. Siehe [CONTRIBUTING.md](CONTRIBUTING.md) und den [Verhaltenskodex](CODE_OF_CONDUCT.md).
* Das README gibt es in acht Sprachen: Eine Textänderung wird in allen acht Dateien gemacht, und die beiden Bilder werden aus `docs/architecture.json` und `docs/structure.json` mit `tools/make_architecture.py` und `tools/make_structure.py` gezeichnet.
* Melde eine Schwachstelle privat: [SECURITY.md](SECURITY.md). Fragen, Ideen und Meldungen: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LIZENZ

Software: GPL-3.0-or-later. Siehe [LICENSE](LICENSE). Die Hardware-Entwürfe der Suite verwenden die CERN OHL-S.
