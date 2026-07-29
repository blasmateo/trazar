#!/usr/bin/env bash
#
# build.sh - Empaqueta exportar_docx.py en un binario autocontenido con
# PyInstaller (one-file). El binario incluye el intérprete Python y python-docx,
# por lo que NO requiere Python instalado en la máquina destino.
#
# El binario se deposita en <destino>/_scripts/exportar-docx, junto al
# ejecutable Rust, siguiendo la convención de distribución de Trazar.
#
# Uso:
#   scripts/exportar-docx/build.sh [destino]
#       Si no se especifica destino, se instala tanto en target/debug como en
#       target/release para cubrir ambos perfiles de compilación.
#       Destinos comunes:
#         target/release   (perfil release)
#         target/debug     (perfil debug, desarrollo)
#
set -euo pipefail

# Raíz del proyecto (este script vive en <raíz>/scripts/exportar-docx/)
RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC="$RAIZ/scripts/exportar-docx/exportar_docx.py"
VENV="$RAIZ/.venv"

# --- Validaciones previas -------------------------------------------------
if [ ! -f "$SRC" ]; then
    echo "✗ No se encontró el script: $SRC" >&2
    exit 1
fi

if [ ! -x "$VENV/bin/python" ]; then
    echo "✗ No se encontró el venv: $VENV/bin/python" >&2
    echo "  Cree el entorno con: python3 -m venv .venv && .venv/bin/pip install -r scripts/exportar-docx/requirements.txt" >&2
    exit 1
fi

# Verificar pyinstaller y python-docx en el venv
if ! "$VENV/bin/python" -c 'import PyInstaller' 2>/dev/null; then
    echo "ℹ Instalando pyinstaller en el venv..." >&2
    "$VENV/bin/pip" install pyinstaller >&2
fi
if ! "$VENV/bin/python" -c 'import docx' 2>/dev/null; then
    echo "✗ python-docx no está instalado en el venv." >&2
    echo "  Instale con: .venv/bin/pip install -r scripts/exportar-docx/requirements.txt" >&2
    exit 1
fi

_empaquetar() {
    local DEST="$1"
    local SCRIPTS_OUT="$DEST/_scripts"

    echo "ℹ Empaquetando exportar-docx → $SCRIPTS_OUT ..."
    mkdir -p "$SCRIPTS_OUT"

    # Compilar el binario. --onefile genera un único ejecutable autocontenido.
    # --clean ignora caché de compilaciones previas. --noconfirm no pregunta.
    # workpath/distpath controlan dónde se ponen artefactos temporales y salida.
    "$VENV/bin/python" -m PyInstaller \
        --onefile \
        --name exportar-docx \
        --distpath "$SCRIPTS_OUT" \
        --workpath "$RAIZ/build/pyinstaller" \
        --specpath "$RAIZ/build/pyinstaller" \
        --clean \
        --noconfirm \
        "$SRC" >&2

    # En Linux el binario queda en <SCRIPTS_OUT>/exportar-docx; en Windows .exe
    local BIN="$SCRIPTS_OUT/exportar-docx"
    if [ ! -x "$BIN" ] && [ -x "$BIN.exe" ]; then
        BIN="$BIN.exe"
    fi

    if [ ! -x "$BIN" ]; then
        echo "✗ PyInstaller no generó el binario esperado: $BIN" >&2
        exit 1
    fi

    echo "✓ Binario empaquetado: $BIN"
}

# --- Empaquetado ----------------------------------------------------------
if [ $# -ge 1 ]; then
    # Destino explícito: un solo perfil
    _empaquetar "$1"
else
    # Sin argumento: instalar en ambos perfiles (debug + release)
    echo "ℹ Instalando en target/debug y target/release (para cubrir ambos perfiles de compilación)..."
    _empaquetar "$RAIZ/target/debug"
    _empaquetar "$RAIZ/target/release"
fi

# Limpiar caché de pyinstaller (única carpeta compartida)
rm -rf "$RAIZ/build/pyinstaller"

echo ""
echo "✓ Distribuya <destino>/trazar junto con <destino>/_scripts/exportar-docx"
