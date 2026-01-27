# BUSMASTER C++ Dependencies Reference

This document catalogs all dependencies from the original BUSMASTER C++ implementation as referenced in `source/Readme.txt`. This serves as a reference for understanding the original architecture and for potential Rust equivalents.

## Original Repository

- **GitHub**: https://github.com/rbei-etas/busmaster
- **Website**: https://rbei-etas.github.io/busmaster/
- **Releases**: https://github.com/rbei-etas/busmaster/releases

## Build Tools

### 6.1 MinGW / GCC
- **Purpose**: C/C++ compiler for Windows
- **Original URL**: http://sourceforge.net/projects/mingw/files/MinGW/
- **Current Source**: https://sourceforge.net/projects/mingw-w64/
- **Latest Version**: mingw-w64-v11.0.0
- **License**: GPL

**Rust Equivalent**: Not needed - Rust has its own compiler (rustc)

### 6.2 Bison
- **Purpose**: Parser generator (YACC-compatible)
- **Original URL**: http://downloads.sourceforge.net/gnuwin32/bison-2.4.1-src-setup.exe
- **Current Source**: https://www.gnu.org/software/bison/
- **License**: GPL-3.0

**Rust Equivalent**: 
- `nom` - Parser combinator library
- `pest` - PEG parser generator
- `lalrpop` - LALR(1) parser generator

### 6.3 Flex
- **Purpose**: Lexical analyzer generator
- **Original URL**: http://flex.sourceforge.net/
- **Current Source**: https://github.com/westes/flex
- **License**: BSD

**Rust Equivalent**:
- `logos` - Fast lexer generator
- `nom` - Can handle lexing as well

### 6.4 Libxml2
- **Purpose**: XML parsing library
- **Original URL**: ftp://xmlsoft.org/libxml2/win32/
- **Current Source**: https://gitlab.gnome.org/GNOME/libxml2
- **GitHub Mirror**: https://github.com/GNOME/libxml2
- **License**: MIT

**Rust Equivalent**:
- `quick-xml` - Fast XML parser
- `roxmltree` - Read-only XML tree
- `xml-rs` - XML parser

### 6.5 GetText
- **Purpose**: Internationalization (i18n) library
- **Original URL**: http://www.gnu.org/software/gettext/
- **FTP**: http://ftp.gnu.org/gnu/gettext/
- **Windows**: http://gnuwin32.sourceforge.net/packages/gettext.htm
- **License**: GPL/LGPL

**Rust Equivalent**:
- `fluent` - Mozilla's i18n system
- `gettext-rs` - Rust bindings to gettext
- `rust-i18n` - Simple i18n macro

### 6.6 Oxygen Icon Set
- **Purpose**: UI icons
- **URL**: http://www.oxygen-icons.org/
- **License**: LGPL/CC-BY-SA

**Rust Equivalent**: Same icons can be used, or alternatives like:
- `egui` built-in icons
- `iced` icons
- Custom SVG icons

## CodeProject Components

License: http://www.codeproject.com/info/cpol10.aspx (Code Project Open License)

### 7.1 DM Graph
- **Purpose**: 2D Graph ActiveX control
- **URL**: http://www.codeproject.com/Articles/310494/2D-Graph-ActiveX-control-in-Cplusplus-with-ATL-no
- **Description**: ATL-based 2D graphing without MFC dependency

**Rust Equivalent**:
- `plotters` - Rust plotting library
- `egui_plot` - Plotting for egui
- `charts-rs` - Chart rendering

### 7.2 Header Control
- **Purpose**: Custom header control for lists
- **URL**: http://www.sothink.com

**Rust Equivalent**: Built into UI frameworks (egui, iced, ratatui)

### 7.3 MFC Tree Control
- **Purpose**: Tree view control
- **URL**: http://doc.mimec.org/articles/mfc/mctree/index.html

**Rust Equivalent**: Built into UI frameworks

### 7.4 Qt 5.3.0
- **Purpose**: Cross-platform UI framework
- **Original URL**: http://download.qt-project.org/archive/qt/5.3/5.3.0/
- **Current Source**: https://www.qt.io/download-open-source
- **Archive**: https://download.qt.io/archive/qt/
- **License**: LGPL-3.0

**Rust Equivalent**:
- `egui` - Immediate mode GUI (our choice for MVP)
- `iced` - Elm-inspired GUI
- `slint` - Qt-like declarative UI
- `tauri` - Web-based desktop apps

### 7.5 FreeType Library
- **Purpose**: Font rendering
- **URL**: http://www.freetype.org/
- **License**: FTL (BSD-style) or GPL-2.0

**Rust Equivalent**:
- `rusttype` - Font rendering
- `fontdue` - Fast font rasterization
- Built into egui/iced

### 7.6 ANTLR 3.5.2
- **Purpose**: Parser generator for language recognition
- **URL**: http://www.antlr3.org/download.html
- **Current Version**: 3.5.3
- **License**: BSD-3-Clause

**Rust Equivalent**:
- `pest` - PEG parser (our choice for DBC parsing)
- `nom` - Parser combinators
- `tree-sitter` - Incremental parsing

## Hardware SDKs (Not Open Source)

These are vendor-specific SDKs that require procurement:

### PEAK Systems
- **Hardware**: PCAN-USB adapters
- **SDK**: PCAN-Basic API
- **Website**: https://www.peak-system.com/
- **macOS SDK**: Available on request

### ETAS GmbH
- **Hardware**: ES 581 and other devices
- **SDK**: HSP (Hardware Service Pack)
- **Website**: https://www.etas.com/

### Vector Informatik
- **Hardware**: VN series interfaces
- **SDK**: XL Driver Library
- **Website**: https://www.vector.com/

## Dependency Mapping Summary

| C++ Dependency | Purpose | Rust Equivalent | Status |
|----------------|---------|-----------------|--------|
| MinGW/GCC | Compiler | rustc | ✅ Built-in |
| Bison | Parser gen | pest/nom | ✅ Planned |
| Flex | Lexer gen | logos/nom | ✅ Planned |
| Libxml2 | XML parsing | quick-xml | ✅ Planned |
| GetText | i18n | fluent | ⏳ Phase 3 |
| Qt 5.3 | GUI | egui | ⏳ Phase 3 |
| DM Graph | Plotting | plotters | ⏳ Phase 3 |
| FreeType | Fonts | fontdue | ✅ Built into egui |
| ANTLR | Parsing | pest | ✅ Planned |

## Notes for Rust Implementation

1. **No ActiveX**: Rust doesn't use ActiveX/COM. All UI components will be native Rust.

2. **No MFC**: No Microsoft Foundation Classes dependency. Using cross-platform libraries.

3. **Parser Strategy**: Using `nom` for binary protocols (CAN frames) and `pest` for text formats (DBC files).

4. **UI Strategy**: 
   - MVP: CLI (`clap`) + TUI (`ratatui`)
   - Phase 3: GUI (`egui`)
   - Phase 3: Web UI (Yew/Leptos)

5. **XML Strategy**: Using `quick-xml` for ARXML parsing (Phase 3).

## References

- Original BUSMASTER Readme: `source/Readme.txt`
- BUSMASTER GitHub: https://github.com/rbei-etas/busmaster
- BUSMASTER Documents: https://github.com/rbei-etas/busmaster-documents
