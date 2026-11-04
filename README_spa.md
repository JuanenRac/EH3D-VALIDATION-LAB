<p align="center">
  <img src="images/EH3D_VALIDATION_LAB_BANNER.svg" alt="EH3D-VALIDATION-LAB banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-VALIDATION-LAB

<p align="center"><a href="README.md">🇺🇸 English</a> | 🇪🇸 <b>Español</b> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### EH3D-VALIDATION-LAB: el entorno de pruebas de la familia PHOENIX: manifiestos de archivos dorados con copias redundantes, fallos clasificados y una puerta que impide que baje una tasa medida. Código inicial.

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust%20%2B%20Python-blue.svg" alt="Rust and Python">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-44-2EA043.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.1-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Comprobación de honestidad - lo que se ejecuta hoy:** una pequeña biblioteca de Rust con línea de comandos que escribe y verifica el manifiesto de sumas de control de un juego dorado contra copias redundantes, lee ficheros de resultados en los que todo fallo debe llevar una clase, y compara una ejecución con una línea base mediante una puerta. Tiene 41 pruebas propias y pasan en un ordenador. Solo se ha usado con datos creados por sus propias pruebas: nunca con un corpus real ni con un juego dorado real. No lee ningún archivo de diseño, no ejecuta ninguna herramienta de referencia y todavía no compara la salida de ninguna herramienta de diseño. El reductor de casos, las pruebas de invariantes y la comparación con archivos dorados no existen.

---

> **Estado de seguridad (safety) y de seguridad informática:** este repositorio está en sus primeros pasos y no es un sistema de seguridad (safety). Nada de lo escrito aquí sustituye los límites, los enclavamientos y la parada de emergencia de la máquina y de su controlador.

## 📋 De un vistazo

| | |
|---|---|
| Versión | `0.0.1` |
| Madurez | Andamiaje |
| Pruebas | 44 |
| Pila tecnológica | Rust, Python 3.10+ |
| En hardware real | Aún no se ha ejecutado en hardware |

**Ir a:** [Resumen](#-resumen) · [Arquitectura](#-arquitectura) · [Inicio rápido](#-inicio-rápido) · [Pruebas](#-pruebas) · [Estructura del repositorio](#-estructura-del-repositorio) · [Proyectos relacionados](#-proyectos-relacionados)

## 🚦 Estado del proyecto

Cada línea es una parte de este repositorio y la luz dice hasta dónde llega hoy. Se escribe con lo que el repositorio puede demostrar (pruebas, compilaciones, ejecuciones), no con lo previsto.

🟢 Hecho y verificado · 🟡 Funciona, con límites conocidos · 🟠 Hecho a medias o sin verificar · 🔴 Sin hacer o sin funcionar

| Área | Estado | Qué significa aquí |
|---|---|---|
| Comprobaciones del repositorio | 🟡 | La comprobación del repositorio, la de las imágenes y las 3 pruebas de la base se ejecutan en el CI y pasan. |
| Pruebas | 🟡 | Las 41 pruebas del código Rust y las 3 de la base pasan en un ordenador. Usan solo datos creados por las propias pruebas; ninguna se ha ejecutado con un corpus real. |
| Hardware real | 🟠 | No se ha ejecutado en ninguna máquina ni placa: nada de aquí se ha probado en hardware. |
| Función planificada | 🟠 | Escrito: manifiestos de juegos dorados, verificación de copias, resultados clasificados y la puerta. Sin escribir: la comparación con archivos dorados, el reductor de casos y las pruebas de invariantes. |
| Requisitos | 🟠 | 4 de 6 requisitos están en prototipo, 2 solo están planificados y ninguno está validado. |

## 🎯 Resumen

* **Lo que existe:** una biblioteca y una línea de comandos, sin ventana. Escriben y verifican el manifiesto `SHA256SUMS` de un juego dorado contra copias redundantes, leen ficheros de resultados donde todo fallo lleva una de tres clases, y comparan una ejecución con una línea base: la tasa medida de cada nivel no puede bajar. Los informes muestran solo hashes, niveles, clases y recuentos.
* **Planificado:** la comparación con archivos dorados, el reductor de casos, las pruebas con nombre de los invariantes del modelo de diseño y, cuando proceda, un tipo de tarea del servidor. Nada de esto existe todavía.

## 🧩 Papel en EH3D

EH3D-VALIDATION-LAB forma parte de la familia PHOENIX de la EH3D Fabrication Suite. Lo que hará, lo que todavía no hará, sus requisitos y sus relaciones con otros repositorios están escritos en la carpeta `docs/`.

## 🏛️ Arquitectura

<p align="center">
  <img src="images/architecture.svg" alt="Arquitectura de EH3D-VALIDATION-LAB: quién lo usa, qué contiene y de qué depende" width="100%">
</p>

Cómo encajan las partes de este repositorio: quién lo usa, qué contiene y de qué depende. La imagen se dibuja desde `docs/architecture.json` y se comprueba contra el código.

## 🚀 Inicio rápido

Necesitas Python 3.10 o superior para las comprobaciones y una cadena de herramientas de Rust para el código.

```bash
cargo build --release
cargo test
cargo run -- manifest <golden-folder>
python -I tools/ci_validate.py
python -I tools/make_architecture.py --check
python -I tools/make_structure.py --check --readmes
python -I -m unittest discover -s tools -p "test_*.py"
```

## 🧪 Pruebas

El código tiene 41 pruebas (`cargo test`) y la base tiene 3 (`python -I -m unittest discover -s tools -p "test_*.py"`); todas pasan en un ordenador. El CI ejecuta la comprobación del repositorio, la de las imágenes, las pruebas de la base y, para el código Rust, la comprobación de formato, clippy con las advertencias como errores y las pruebas en Linux y Windows.

## 🏗️ Compilación y publicación

La versión vive en el fichero `VERSION` y en `eh3d.project.json`, y cada versión tiene su entrada en [CHANGELOG.md](CHANGELOG.md). `cargo build --release` compila la biblioteca y el programa `eh3d-validation-lab`; las versiones se publican en GitHub.

## 📂 Estructura del repositorio

<p align="center">
  <img src="images/repository-structure.svg" alt="Estructura del repositorio EH3D-VALIDATION-LAB" width="100%">
</p>

<details>
<summary>Versión en texto del árbol</summary>

```text
EH3D-VALIDATION-LAB/
├── docs/
│   ├── architecture.json structure.json         los datos de las dos imágenes
│   ├── SCOPE.md REQUIREMENTS.md                 su alcance y sus requisitos
│   ├── DEPENDENCIES.md THREAT_MODEL.md          relaciones con otros repositorios y modelo de amenazas
│   ├── DECISIONS.md STATUS.md                   decisiones que se aplican aquí y estado honesto
│   ├── NOT_YET.md RULES.md                      ideas para más adelante y reglas para escribir código
│   └── adr/                                     los registros de decisiones aceptadas
├── AGENTS.md                                    reglas para quien cambia el repositorio
├── deny.toml                                    política de licencias y avisos de las dependencias
├── src/                                         la biblioteca y la línea de comandos
├── config/                                      el fichero de ajustes (todo valor afinado)
├── Cargo.toml Cargo.lock                        el paquete de Rust y sus dependencias fijadas
├── tools/                                       la comprobación del repositorio, sus pruebas y las herramientas de las imágenes
├── images/                                      banner y las imágenes de Arquitectura y Estructura del repositorio
├── .github/                                     flujos de CI y del panel, plantillas de issues y pull requests
├── eh3d.project.json                            manifiesto: versión, rol, madurez y comando de compilación
├── VERSION                                      la versión del repositorio
├── CHANGELOG.md                                 qué cambió en cada versión
├── SECURITY.md SUPPORT.md                       cómo informar de una vulnerabilidad y dónde preguntar
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   cómo contribuir, el código de conducta y la licencia
```

</details>

## 🗺️ Hoja de ruta y límites conocidos

**Límites conocidos hoy**

* Código inicial: solo se ha ejecutado con datos creados por sus propias pruebas, nunca con un corpus real ni con un juego dorado real.
* Comprueba que las copias contienen los mismos bytes; ponerlas en soportes distintos corresponde al propietario.
* La comparación con archivos dorados, el reductor de casos y las pruebas de invariantes no están escritos.

**Siguiente**

* Escribir la comparación con archivos dorados y las pruebas de invariantes cuando se acepten los contratos de los que depende este proyecto (véase `docs/DEPENDENCIES.md`).

## 🌍 Idiomas

El README y los textos existen en inglés, español, francés, italiano, alemán, chino, japonés y portugués de Portugal.

## 🔗 Proyectos relacionados

* [EH3D-DOCS](https://github.com/JuanenRac/EH3D-DOCS): el mapa de la suite, su arquitectura, las convenciones y las comprobaciones compartidas
* Relaciones planificadas, aún sin construir ([EH3D-PHOENIX-PCB](https://github.com/JuanenRac/EH3D-PHOENIX-PCB), [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON), [EH3D-LAB-BRIDGE](https://github.com/JuanenRac/EH3D-LAB-BRIDGE)): véase [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)

## 📚 Documentación

* [Las convenciones de la suite](https://github.com/JuanenRac/EH3D-DOCS/blob/main/docs/CONVENTIONS.md)
* [Registro de cambios](CHANGELOG.md)
* [Alcance, requisitos, dependencias, modelo de amenazas y decisiones](docs/SCOPE.md)

## 🤝 Contribuir y comunidad

* Antes de una pull request ejecuta `python -I tools/ci_validate.py` y las pruebas de la base: todo debe pasar. Consulta [CONTRIBUTING.md](CONTRIBUTING.md) y el [código de conducta](CODE_OF_CONDUCT.md).
* El README existe en ocho idiomas: un cambio en el texto se hace en los ocho archivos, y las dos imágenes se dibujan a partir de `docs/architecture.json` y `docs/structure.json` con `tools/make_architecture.py` y `tools/make_structure.py`.
* Informa de una vulnerabilidad en privado: [SECURITY.md](SECURITY.md). Preguntas, ideas e informes: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENCIA

Software: GPL-3.0-or-later. Véase [LICENSE](LICENSE). Los diseños de hardware de la suite usan CERN OHL-S.
