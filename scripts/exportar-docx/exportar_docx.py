#!/usr/bin/env python3
"""
exportar_docx.py - Herramienta externa de Trazar para exportar
metricas de asistencias (JSON) a documentos .docx.

Uso:
    exportar_docx.py --json <RUTA_JSON> --salida <RUTA_DOCX> --modo [lista|tabla]
"""

import argparse
import json
import sys
from pathlib import Path

from docx import Document
from docx.enum.text import WD_ALIGN_PARAGRAPH


def main():
    parser = argparse.ArgumentParser(
        description="Exportar metricas de asistencias a .docx"
    )
    parser.add_argument("--json", required=True, help="Ruta del archivo JSON de metricas")
    parser.add_argument("--salida", required=True, help="Ruta de salida del archivo .docx")
    parser.add_argument(
        "--modo",
        required=True,
        choices=["lista", "tabla"],
        help="Modo de exportacion: lista (resumen) o tabla (detallado)",
    )
    args = parser.parse_args()

    ruta_json = Path(args.json)
    if not ruta_json.exists():
        print(f"No existe el archivo JSON: {ruta_json}", file=sys.stderr)
        sys.exit(1)

    try:
        with open(ruta_json, "r", encoding="utf-8-sig") as f:
            datos = json.load(f)
    except json.JSONDecodeError as e:
        print(f"Error al parsear JSON: {e}", file=sys.stderr)
        sys.exit(1)

    # La ruta de salida debe ser un archivo .docx, no un directorio.
    salida = Path(args.salida)
    if salida.is_dir():
        print(
            f"La ruta de salida es un directorio, no un archivo: {salida}\n"
            f"  Indique el nombre del documento, por ejemplo: "
            f"{salida / 'asistencias.docx'}",
            file=sys.stderr,
        )
        sys.exit(1)

    try:
        if args.modo == "lista":
            exportar_lista(datos, args.salida)
        else:
            exportar_tabla(datos, args.salida)
    except Exception as e:
        print(
            f"No se pudo crear el documento: {e}\n"
            f"  Verifique que la ruta de salida sea un archivo .docx valido "
            f"y que tenga permisos de escritura.",
            file=sys.stderr,
        )
        sys.exit(1)

    print(f"\u2713 Documento exportado: {args.salida}")


def _cabecera(doc, datos):
    """Escribe la cabecera comun del documento."""
    curso = datos.get("curso", "Curso")
    total_clases = datos.get("totalClases", 0)
    fecha = datos.get("fechaActualizacion", "")

    titulo = doc.add_heading(f"Metricas de Asistencias - {curso}", level=1)
    titulo.alignment = WD_ALIGN_PARAGRAPH.CENTER

    p = doc.add_paragraph()
    p.add_run("Total de clases: ").bold = True
    p.add_run(f"{total_clases}")

    if fecha:
        p2 = doc.add_paragraph()
        p2.add_run("Fecha de actualizacion: ").bold = True
        p2.add_run(fecha)

    doc.add_paragraph()


def exportar_lista(datos, salida):
    """Exporta el modo lista (resumen por cursante, sin detalle)."""
    doc = Document()
    _cabecera(doc, datos)

    cursantes = datos.get("cursantes", {})
    if not cursantes:
        doc.add_paragraph("No hay cursantes para mostrar.")
    else:
        doc.add_heading(f"Cursantes ({len(cursantes)})", level=2)

        tabla = doc.add_table(rows=1, cols=4)
        tabla.style = "Light Grid Accent 1"
        hdr = tabla.rows[0].cells
        hdr[0].text = "Cursante"
        hdr[1].text = "Presente"
        hdr[2].text = "Ausente"
        hdr[3].text = "% Asistencia"

        for nombre in sorted(cursantes.keys()):
            info = cursantes[nombre]
            presente = int(info.get("presente", 0))
            ausente = int(info.get("ausente", 0))
            total = presente + ausente
            porcentaje = round((presente / total * 100)) if total > 0 else 0

            fila = tabla.add_row().cells
            fila[0].text = nombre
            fila[1].text = str(presente)
            fila[2].text = str(ausente)
            fila[3].text = f"{porcentaje}%"

    doc.save(salida)


def exportar_tabla(datos, salida):
    """Exporta el modo tabla (detalle por cursante con asignatura/clase/asiste)."""
    doc = Document()
    _cabecera(doc, datos)

    cursantes = datos.get("cursantes", {})
    if not cursantes:
        doc.add_paragraph("No hay cursantes para mostrar.")
    else:
        for nombre in sorted(cursantes.keys()):
            info = cursantes[nombre]
            presente = int(info.get("presente", 0))
            ausente = int(info.get("ausente", 0))
            total = presente + ausente
            porcentaje = round((presente / total * 100)) if total > 0 else 0

            doc.add_heading(f"{nombre}", level=2)
            p = doc.add_paragraph()
            p.add_run(
                f"Total: {total} | Presente: {presente} | "
                f"Ausente: {ausente} | % Asistencia: {porcentaje}%"
            )

            detalle = info.get("detalle", [])
            if detalle:
                tabla = doc.add_table(rows=1, cols=3)
                tabla.style = "Light Grid Accent 1"
                hdr = tabla.rows[0].cells
                hdr[0].text = "Asignatura"
                hdr[1].text = "Clase"
                hdr[2].text = "Asiste"

                def ordenar(asis):
                    asig = asis.get("asignatura", "")
                    id_str = asig.split("-")[0] if "-" in asig else asig
                    try:
                        id_num = int(id_str)
                    except ValueError:
                        id_num = 999999
                    return (id_num, int(asis.get("clase", 0)))

                for asis in sorted(detalle, key=ordenar):
                    asig = asis.get("asignatura", "")
                    asig_mostrar = asig if asig else "eventos-academicos"
                    clase = int(asis.get("clase", 0))
                    presente = bool(asis.get("presente", False))
                    estado = "si" if presente else "no"

                    fila = tabla.add_row().cells
                    fila[0].text = asig_mostrar
                    fila[1].text = str(clase)
                    fila[2].text = estado

            doc.add_paragraph()  # separacion entre cursantes

    doc.save(salida)


if __name__ == "__main__":
    main()
